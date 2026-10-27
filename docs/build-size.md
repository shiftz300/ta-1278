# Build size

Release builds use `opt-level = "z"`, fat LTO, one codegen unit, stripped symbols and abort-on-panic. Debug artifacts and `target/` are development outputs, not the distributable app.

The main payloads are original and fallback fonts, machine code and linked libraries, the sound bank, T9 code/dictionaries, icons and small audio resources. Fonts, T9 data and the sound bank are losslessly compressed at build time with roundtrip checks. Optional independent dictionaries remain in the source tree but are removed from the standard release executable by dead-code elimination.

```sh
sh scripts/package-macos.sh
python3 scripts/report-package-size.py
```

The report is written to `target/packed-build-size.json`. It measures the actual signed bundle, locates packed resource bytes, verifies decompression and separates Mach-O code sections from other data. Bundle metadata, helper code and license/source notices are included. It requires the built macOS bundle and its matching Cargo build outputs. This is a size measurement, not a performance benchmark.

Full firmware, research reports, raw captures and Cargo caches are not packaged. Historical measurements and comparisons live in the separate research archive.

## 2026-10-28 resource reduction

Measured on macOS arm64 with the same release profile/toolchain, against the
preceding Shell build before these resource changes. Logical sizes include the
ad-hoc signatures, native helper and notices. ZIP uses Deflate level 9 and fixed
entry timestamps for the comparison.

| Output | Before | After | Change |
|---|---:|---:|---:|
| Signed app bundle | 19,887,702 B (18.9664 MiB) | 18,552,789 B (17.6933 MiB) | -1,334,913 B (-6.71%) |
| Signed executable | 19,480,784 B | 18,151,088 B | -1,329,696 B |
| ZIP | 17,321,807 B (16.5194 MiB) | 15,980,882 B (15.2406 MiB) | -1,340,925 B (-7.74%) |

The Emoji-only fallback (876,394 packed bytes) and its unused source/subset/license
files are removed. Unsupported characters use the square glyph. Existing handset,
CJK and ordinary-symbol fonts are unchanged; 30,698 visible fallback characters
were compared at 18/20/24 px with identical metrics and pixels. This intentionally
removes 1,070 Emoji mappings; the saved remaining coverage is 48,262 codepoints.

SF2 sample PCM is second-differenced modulo 2^16 and split into low/high byte
planes before existing zlib compression. After bounded inflate the loader reverses
this filter before normal OxiSynth loading. Metadata, samples and all GM/drum
presets are retained. The original source SF2 is unchanged. Packed bank bytes fall
from 2,470,319 to 2,009,194; the new code and metadata are included in the measured
executable above. No dependencies were added.

Build-time roundtrips compare every SF2 byte; the measurement script independently
reconstructs and verifies it. Offline synthesis of the original and restored bank
produces bit-identical PCM. The workspace has 240 passing tests and seven opt-in
tests ignored. The exhaustive font parity test also passed explicitly. Across
282 CPU scenes, 279 PNGs match exactly; the other differences are the intentional
Emoji row and two live date/time regions (captures crossed midnight). All pixels
outside those regions match. No sound output was used.

Reproduce with the commands above, `python3 -S scripts/check-assets.py`, and
`cargo test --locked --workspace`. Run the explicit font comparison after font
changes: `cargo test --locked -p phone-desktop --no-default-features canvas::parity -- --ignored`.
Detailed local measurements are in `target/packed-build-size.json` and
`target/resource-size-comparison.json`; generated archives stay outside Git.

## MIDI player increment

After adding streaming local MIDI playback, the same optimized macOS arm64 bundle
is 18,552,853 bytes (17.6934 MiB), 64 bytes above the resource-optimized build
above. No dependencies or soundfont copies were added. This is a net bundle
comparison, including linker layout/signing; it does not measure the player code
in isolation. Reproduce with `sh scripts/package-macos.sh` and
`python3 scripts/report-package-size.py --out target/midi-build-size.json`.
The current workspace has 243 passing tests and seven opt-in tests ignored; MIDI
scan, PCM, seeking and EOF tests use no audio device.
