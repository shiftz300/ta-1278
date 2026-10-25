# App size and build changes

The packaging script copied a **debug** executable into `TA1278.app`. It now builds with `--release`, optimization level 2, ThinLTO, one codegen unit, stripped symbols and abort-on-panic. Debug builds remain available for development.

| Item | Bytes | MiB |
|---|---:|---:|
| Previous debug executable | 71,440,816 | 68.13 |
| Final release executable, before bundle signing | 37,594,144 | 35.85 |
| Final executable inside the signed app | 37,375,680 | 35.64 |

The signed executable is **47.7% smaller** than the previous debug executable. Code signing rewrites the executable signature, so pre-sign and bundled file sizes/hashes differ. The whole bundle is 37,475,252 bytes (35.74 MiB). Exact hashes and the before/after frame comparison are in [the manifest](evidence/build-size.json).

The largest embedded data are intentional: 19,653,088 bytes of fallback fonts, 5,476,096 bytes of original fonts and 5,285,295 bytes of input dictionaries (**29.0 MiB combined**). They retain the requested character coverage. No original firmware image, private capture, research report or Cargo build cache is bundled. `target/` was about 2.5 GiB before the release build; it is a developer cache, not the distributable app.

## Implementation checks

- Text rendering borrowed cached glyph bitmaps instead of cloning each bitmap for every character on every frame.
- Icon rendering now samples the source image directly with the same nearest-neighbor coordinates. It no longer creates a resized image on every draw.
- New game sprites use source rectangles at native scale, without temporary resized images.
- Font/dictionary coverage is unchanged. The audio follow-up adds Rodio/CPAL/Symphonia with only MP3 and PCM WAV enabled. The simple HTML adapter remains; no full browser engine was added.

The optimization-only comparison had **42 identical framebuffer scenes out of 43**. The only difference was the home clock advancing one minute. The later Snake, alarm, SMS and Home footer changes intentionally alter their images. This is visual regression evidence, not a measured CPU, memory or battery improvement. Fonts still parse at startup and the display loop still checks updates at about 30 Hz; those remain possible future performance work.

The current signed app passed 88 default Rust tests (one additional audio hardware test is run explicitly for WAV and MP3), warnings-denied Clippy, resource checks and a real Metal window run: **67 saved scenes / 68 presented frames**. These are CPU framebuffers uploaded to wgpu, not GPU readbacks. The app has not passed complete handset acceptance.

Build: `sh scripts/package-macos.sh`. Open `target/TA1278.app`.
