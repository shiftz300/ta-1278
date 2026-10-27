# Desktop builds and releases

The winit/wgpu desktop host targets macOS arm64/Intel, Windows x64 and Linux x64. These share input, rendering, local messages, contacts, settings, Snake, browser/JavaScript, folder access and MP3/WAV playback.

macOS keeps its Swift folder/media helper. Windows uses a native folder dialog and stores its profile under `%LOCALAPPDATA%/TA1278 Replica` (falling back to `%APPDATA%`). Linux uses an XDG portal/Zenity dialog and `$XDG_DATA_HOME/ta1278-replica` or `~/.local/share/ta1278-replica`. `--storage-dir PATH` bypasses the dialog on every platform. Image/video decoding and storage-capacity reporting currently remain macOS-only; other hosts report these services unavailable.

Linux needs ALSA, X11 (including libxkbcommon-x11) or Wayland, and a Vulkan/OpenGL driver. Build prerequisites on Ubuntu 24.04:

```sh
sudo apt-get install pkg-config libasound2-dev
# X11 startup and the folder portal also need their runtime libraries:
sudo apt-get install libxkbcommon-x11-0 libdbus-1-3
```

D-Bus, Wayland and XKB use dynamic loading; their development headers and libudev-dev are not build requirements. A desktop portal backend or Zenity is needed for the folder dialog. No portal is needed when using `--storage-dir`. Windows builds need the MSVC C++ tools for the native JS/audio dependencies. macOS builds need Apple Command Line Tools. Window portability alone does not supply multimedia codecs or a phone modem backend.

The HTML parser disables its upstream command-line support and unused parse-error collection. wgpu enables native backends and WGSL input without WebGPU/browser features. The Rust image and playback crates enable only PNG and MP3/WAV/PCM codecs respectively; MIDI and macOS system media decoding use separate paths. Remaining direct dependencies serve active features. fontTools (font generation) and Pillow (screenshot comparisons) are optional local developer tools; neither is installed in CI or shipped with the app.

## CI

`CI` runs on every branch push and through Run workflow. A shared check validates the version, formatting and resource hashes once. Fonts and their coverage report are generated locally and checked in; resource checks use the Python standard library without fontTools or conversion. Native runners check release tools, Clippy, workspace tests, the headless library and an optimized desktop build. Release-tool tests remain on every OS to catch path and newline differences. Linux additionally presents the synthetic window scenarios through Xvfb and Mesa software graphics. Ignored audio/public-network checks are not enabled in CI. Windows and macOS runner builds are not a substitute for manual UI acceptance.

Archives and their SHA-256 files are retained as workflow artifacts for 14 days. Already compressed archives are uploaded without another compression pass. Cargo caches use platform, Rust version, lockfile and source hashes; fallback keys reuse dependencies. Caches exclude packaged app bundles and archives. GitHub Actions dependencies are fixed to inspected commit SHAs. Build jobs have read-only repository permission; only tagging and publication jobs receive the writes they need. No personal access token is required.

## Change the version

```sh
python3 scripts/version.py bump patch
# Also accepts minor, major, or a larger X.Y.Z.
python3 scripts/version.py check
cargo test --workspace --locked
```

Review and commit the changed Cargo.toml and Cargo.lock, then push to `main`. After all four native builds pass, CI creates the missing annotated `vX.Y.Z` tag. A newer main commit supersedes stale tagging jobs. Existing version tags never move; ordinary same-version pushes only run CI.

CI explicitly dispatches `Release` at the new tag and passes its run ID: a tag pushed with `GITHUB_TOKEN` does not itself start another push-triggered workflow. Automatic releases reuse those four CI archives without rebuilding. Before download, Release verifies the source repository, CI workflow, event, main branch, exact tagged commit and success of all four native builds and shared checks. The final CI tagging job may still be finishing when Release starts. Every archive's SHA-256 is also checked before publication. [Cross-run artifact download](https://github.com/actions/download-artifact/blob/v7/README.md#download-artifacts-from-other-workflow-runs-or-repositories).

A manually pushed `v*` tag, or a manual Release with no CI run ID, runs the full native build matrix as a fallback. Both routes require the version tag to match the exact checked-out commit. The first successful CI run can tag the existing untagged 0.2.2 version.

Release uploads macOS arm64/Intel ZIPs, a Windows x64 ZIP, a Linux x64 tar.gz and SHA256SUMS. It uploads into a draft and publishes only after all uploads succeed. A failed draft may be resumed; a published release is retained without replacing its assets. macOS packages are ad-hoc signed, not notarized; Windows packages have no code-signing certificate.

To retry publication, choose `Release` → Run workflow and select the matching `vX.Y.Z` tag. Supply its successful CI run ID to reuse unexpired artifacts; leave it blank to rebuild. Invalid run IDs fail rather than publishing unverified files. Selecting a branch works only when its HEAD is exactly that version tag. Run `CI` manually on `main` to retry a failed tag/dispatch step.

## Package locally

```sh
python3 scripts/package.py
# Or: --target aarch64-apple-darwin / x86_64-apple-darwin /
#     x86_64-pc-windows-msvc / x86_64-unknown-linux-gnu
```

Use the native OS/toolchain for each target. Archives go in `target/dist/`; no user profile, selected media folder, research archive or raw firmware is included. `scripts/package-macos.sh` remains the short route to `target/TA1278.app`.

Release-tool integrity checks: `python3 scripts/tests.py`. They use temporary Cargo workspaces, a local bare Git remote and mocked GitHub metadata to check lockfile synchronization, annotated tags, repeated tagging, same-version changes, stale CI commits and rejection of unsafe artifact sources.
