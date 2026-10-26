# App size and build changes

The packaging script copied a **debug** executable into `TA1278.app`. It now builds with `--release`, optimization level 2, ThinLTO, one codegen unit, stripped symbols and abort-on-panic. Debug builds remain available for development.

| Item | Bytes | MiB |
|---|---:|---:|
| Previous debug executable | 71,440,816 | 68.13 |
| Final release executable, before bundle signing | 37,693,920 | 35.95 |
| Final executable inside the signed app | 37,474,880 | 35.74 |

The signed executable is **47.5% smaller** than the previous debug executable. Code signing rewrites the executable signature, so pre-sign and bundled file sizes/hashes differ. The whole bundle is 37,574,452 bytes (35.83 MiB). Exact hashes and the before/after frame comparison are in [the manifest](evidence/build-size.json).

The largest embedded data are intentional: 19,653,088 bytes of fallback fonts, 5,476,096 bytes of original fonts and 5,285,295 bytes of input dictionaries (**29.0 MiB combined**). They retain the requested character coverage. No original firmware image, private capture, research report or Cargo build cache is bundled. `target/` was about 2.5 GiB before the release build; it is a developer cache, not the distributable app.

## Implementation checks

- Text rendering borrowed cached glyph bitmaps instead of cloning each bitmap for every character on every frame.
- Icon rendering now samples the source image directly with the same nearest-neighbor coordinates. It no longer creates a resized image on every draw.
- New game sprites use source rectangles at native scale, without temporary resized images.
- Font/dictionary coverage is unchanged. The audio follow-up adds Rodio/CPAL/Symphonia with only MP3 and PCM WAV enabled. The simple HTML adapter remains; no full browser engine was added.

The optimization-only comparison had **42 identical framebuffer scenes out of 43**. The only difference was the home clock advancing one minute. The later Snake, alarm, SMS and Home footer changes intentionally alter their images. This is visual regression evidence, not a measured CPU, memory or battery improvement. Fonts still parse at startup and the display loop still checks updates at about 30 Hz; those remain possible future performance work.

The earlier signed baseline passed 105 default Rust tests (one additional audio hardware test is run explicitly for WAV and MP3), warnings-denied Clippy, resource checks and a real Metal window run: **100 saved scenes / 101 presented frames**. These are CPU framebuffers uploaded to wgpu, not GPU readbacks. The app has not passed complete handset acceptance.

Build: `sh scripts/package-macos.sh`. Open `target/TA1278.app`.

## Local media completion build, 2026-10-26

The final signed app is **37,932,658 bytes (36.18 MiB)**: executable 37,708,208 bytes, native helper 124,656 bytes. This is 358,206 bytes more than the previous signed bundle. The helper uses system frameworks; FFmpeg and SDK/build caches are not bundled.

Scaled-image drawing samples the source without per-frame resize allocations. Same-volume moves use rename. Playlist paths use a library index, and song/queue/playlist lists borrow labels for visible rows instead of cloning the entire library every frame. The 114-scene comparison after this music change had **113 identical scenes**; only the Home wall clock changed. No CPU/frame-time improvement is claimed without measurement.

119 default tests, 114 scenes/115 Metal presentations, native WAV/MP3/video checks and signature checks are recorded in [the current manifest](evidence/local-completion-validation.json). Native comparisons remain partial.

## Original sound-bank build, 2026-10-26

The signed bundle is **40,942,291 bytes (39.05 MiB)** after embedding the 2,586,984-byte original SF2, native MIDI inputs and independent synthesis code. It includes the small pinned synthesizer/parser source archives and licenses. Decoded PCM, system GM/DLS banks, FFmpeg, full firmware images and build caches are not bundled. MIDI synthesis no longer requires the Apple media helper; that helper remains for image/video decoding. Exact per-file hashes and 127-test/118-scene checks are in [native-audio-validation.json](evidence/native-audio-validation.json). The original 36.18 MiB measurement above remains the earlier build, not the current one.
