# Android host

Android uses the same Rust Runtime, winit/wgpu host and 240×320 phone display as
macOS/Windows/Linux. The default standalone backend is Offline. An embedded
backend can be local and in the same process; external world/state rules are not
added to the phone shell. See [runtime interfaces](runtime.md#local-backend-ownership).

## Controls

Android starts in immersive fullscreen: status and navigation bars are hidden.
An edge swipe can reveal them temporarily. Focus return restores immersive mode.
The phone display keeps its 240×320 aspect ratio and fills the window width on
portrait phones. The keypad fills the remaining height, with matching touch
bounds. No space is reserved for the hidden system bars. On short/wide windows,
the display is centered to leave at least 30% of the height for controls. The
window extends into cutout areas; a physical camera hole may cover pixels.

The layout follows the [handset user guide](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/keys-and-parts):
left/right selection, call/end, four-way navigation with center, 0–9, star and hash.
The button artwork is drawn locally; it is not a photograph of the handset.
The keypad follows the owner's handset photo: a single rounded direction pad,
a small center dot, white call/end legends and letters before 3/6/9. The pad is
about as wide as the middle numeric column, with a smaller center target. Its
four directions are equal-area diagonal sectors around the independent center.
Touch mapping and pressed highlights share the same rounded pad geometry.

Touch the buttons below the display. The display itself is not a touchscreen.
Press/release, the existing 500 ms long press, numeric entry and # input-mode
switching pass through the same Input state machine as keyboard input. Multiple
fingers have separate source IDs. Leaving a key cancels its held input until the
next touch. Resize/focus loss/suspend also cancel held keys.

Android gives one short [system haptic key press](https://developer.android.com/develop/ui/views/haptics/haptic-feedback),
subject to the user's system setting and hardware. It needs no vibration permission;
no continuous vibration or background feedback is requested. Set
`controls.haptics` to false in config to disable it. Desktop `--keypad` enables the
same visible controls; `--no-haptics` is also accepted. Window decorations are hidden
by default; desktop `--titlebar` restores them. Desktop has no motor adapter.

## Build

The initial APK targets Android 8/API 26 or later, ARM64. Packaging runs on macOS
or Linux using the official SDK/NDK and JDK, without Gradle or cargo-apk.

Install Rust's `aarch64-linux-android` target and these SDK packages:

```sh
rustup target add aarch64-linux-android
sdkmanager 'platforms;android-35' 'build-tools;35.0.0' 'ndk;28.2.13676358'
export ANDROID_HOME=/path/to/android/sdk
python3 scripts/android.py
# Also run Android Clippy:
python3 scripts/android.py --check
```

`JAVA_HOME` may select the JDK. Android QuickJS bindings use the dependency's
`bindgen` build feature: install libclang on Linux; macOS uses Xcode Command Line
Tools. Android-only JNI/bindgen requirements do not add desktop runtime packages.

Output: `target/android/TA1278-vVERSION-android-arm64.apk` plus SHA-256. It is an
optimized local development build with a locally generated key at
`target/android/debug.keystore`, not a store release. Keep that key to install
updates over a previous build. It and all build output are ignored by Git.
The script checks APK v2/v3 signatures and 16 KiB ZIP alignment. Native code is
linked for 16 KiB pages; no resources are downloaded when the app starts.
`--no-build` repackages an already built native library.

```sh
adb -s DEVICE install -r target/android/TA1278-v0.3.0-android-arm64.apk
adb -s DEVICE shell am start -n dev.shiftz.ta1278/.App
# Quiet UI testing, without changing the device volume or saved configuration:
adb -s DEVICE shell am start -n dev.shiftz.ta1278/.App --ez quiet true
```

The build script does not install to a device. The separate
[Android workflow](https://github.com/shiftz300/ta-1278/actions/workflows/android.yml)
runs on every branch push and manual dispatch. It checks Android Clippy, builds
the optimized ARM64 APK, verifies its signature/alignment, and retains the APK
and SHA-256 as artifacts for 14 days. SDK/NDK versions come from this script.
Desktop CI owns shared tests and version tagging. Release reuses a successful
same-commit Android CI artifact, or calls this Android workflow to build from the
version tag when no unexpired artifact is available. The APK and its SHA-256 are
published alongside desktop downloads and included in `SHA256SUMS`.
No emulator or physical-device tests run in CI.

CI generates a temporary development signing key for each run. Updating an existing
installation requires the same signing key; an APK from another run may require
uninstalling first, which removes app data. CI does not store or upload the local
signing key, and the key is excluded from its Cargo cache.

## Storage and lifecycle

Profile/config: the app's internal files directory, under `profile/`. Phone
storage: the app-specific external files directory, under `Phone/`, falling back
to internal `Phone/` if external storage is unavailable. This is still direct
folder mapping. Arbitrary Storage Access Framework document-tree URIs are not
filesystem paths and are not supported by this implementation. The app does not
ask for broad storage permissions. Uninstall can remove its profile/media.

On Android focus loss, held keys are canceled, audio pauses/stops and data is flushed.
On suspension, the native GPU surface is dropped. Resume recreates the surface. Media stays
paused until requested again. Android system Back maps to the right softkey;
the drawn End key retains the handset's Home behavior.

## Platform limits

- Phone UI, local records, browser, MP3/WAV/MIDI and native sound resources share
  their existing implementations. The Java Activity only supplies UI-thread
  haptic feedback, bounded system bitmap decoding and startup errors.
- Images use Android BitmapFactory on a worker thread (16 MiB input, 32 MP source
  bounds, sampled decode, output no larger than 240×240). No desktop helper is
  needed. Video decoding is not implemented; that capability is disabled.
- There is no real SMS/modem transport, call service, microphone recorder or FM
  hardware adapter. The existing backend interface remains available for a host.
- Virtual key geometry and long presses can be tested on desktop. Motor feel,
  vendor GPU behavior and mobile audio latency require physical-device testing.

## Verification (2026-10-28)

- Moto edge 70 max, Android 16/API 36, ARM64, Adreno Vulkan: install/cold launch,
  full-window layout, drawn-key input, internal # mode switching, numeric long press,
  Android Back, immersive system-bar hiding, background/resume, and cold relaunch
  checked on the physical device.
  The message editor content remained identical across background/resume. A local
  HTTP page executed JavaScript, opened a link and submitted a GET form. Files
  opened a PNG through the Android decoder. Browser history survived app update
  and cold launch. These were app-only test fixtures.
- The user confirmed that key feedback was short and its strength was suitable.
- macOS: 253 tests passed, 8 opt-in checks skipped; desktop and Android Clippy with
  warnings denied; headless build checked. Both default and keypad window modes
  previously presented 127 GPU frames; all 126 saved phone regions were pixel-identical.
  A later desktop smoke rerun passed all 127 frames after the compact SMS update.
  The split viewport was verified on Moto (1440×1920 display, 1440×1248
  controls), including center/bottom-row touch and background/resume.
- Android speaker output/audio latency, other Android devices and current
  Windows/Linux native windows are not accepted by these checks. Testing used
  the quiet launch option. Android's 16 KiB layout is checked in the package;
  this Moto itself uses 4 KiB pages.

The Android profile lock calls the OS `flock` directly because Rust 1.96's
[standard file-lock implementation excludes Android](https://github.com/rust-lang/rust/issues/148325).
It retains exclusive locking instead of skipping it.
