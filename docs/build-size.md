# App size and build changes

The first optimization pass replaced a **debug** executable in `TA1278.app` with a release build using optimization level 2 and ThinLTO. The current size-focused build is measured at the end of this document. Debug builds remain available for development.

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

## Current lossless packing and font deduplication, 2026-10-26

The signed 0.2.1 bundle after the macOS Quit fix is **20,731,395 bytes (19.77 MiB)**, down from 40,991,847 bytes (39.09 MiB) at the start of this pass: **49.4% smaller**. The initial packing build was 20,714,979 bytes; the lifecycle fix and version metadata add 16,416 bytes including signing/alignment changes. [Release hashes](evidence/release-0.2.1.json).

| Component | Stored MiB |
|---|---:|
| Supplemental fonts | 6.47 |
| Original fonts | 4.21 |
| Prediction dictionaries | 1.79 |
| Original sound bank | 2.36 |
| Machine code and static dependencies | 2.77 |
| Other executable data and metadata | 1.80 |
| Native media helper | 0.12 |
| Licenses, source notices and bundle metadata | 0.26 |

These categories sum to the whole bundle. Machine code comes from Mach-O `__text`/`__stubs`; embedded compressed assets are located byte-for-byte in the executable. Other executable data includes small PNG/MIDI/MP3 resources, tables, strings, signing/alignment metadata and other constants, not just application code. [Exact bytes, hashes and method](evidence/packed-build-size.json).

Changes:

- Original fonts, input dictionaries and SF2 are compressed without content loss during Cargo builds. The shared build helper verifies every decompressed byte. Runtime inflation is bounded by the recorded original length. Dictionaries initialize once when first used; font/SF2 temporary byte buffers are released after parsing.
- Supplemental fonts omit characters already covered by earlier fonts and unused OpenType shaping features. Full original downloaded fonts remain in the source checkout; renamed derived fonts and their provenance are under `assets/fonts/subsets/`. No character coverage was removed: the union remains **49,332 codepoints**. All **31,768 visible fallback characters** match full-font metrics and bitmaps at 18/20/24 px. This is not new complex-script or emoji-sequence shaping support.
- Release uses `opt-level="z"`, full LTO, one codegen unit, symbol stripping and abort-on-panic. Compared profiles: packed opt2/ThinLTO executable 31,905,904 bytes, size-s/full LTO 30,103,280 bytes, size-z/full LTO before font subsetting 29,092,848 bytes. Intermediate builds differ slightly in application fixes; these are measured builds, not a precise isolated compiler benchmark.
- Normal builds need Rust and the existing macOS SDK, not Python. Only regenerating font subsets requires developer fontTools. No new runtime system library, external unpacker or downloaded executable was added. The miniz_oxide version was already in Cargo.lock.

The 118-frame comparisons against the initial build differ only at the live Home clock. Original `snake-over` MIDI renders to the same WAV bytes before/after compression and compiler changes. The warmed standalone render path took roughly 23–27 ms at opt2/s and 33–37 ms at z in a small three-run sample; it includes process startup and is not a full playback/frame-time benchmark. This tradeoff was retained for the smaller package. No general CPU or memory improvement is claimed. Compressed dictionaries retain their decompressed source in memory after first use.

Reproduce with `scripts/package-macos.sh`, `python3 scripts/report-package-size.py`, and the explicit `fallback_subsets_preserve_every_visible_glyph` ignored test. Packing validation and limitations are in [the size checks](evidence/size-audit-validation.json); the later lifecycle build is recorded in [restart checks](evidence/restart-validation.json). Earlier measurements above are historical. The shared `build/pack.rs` source is explicitly allowed by the repository ignore rules so a new checkout retains the build helper.
