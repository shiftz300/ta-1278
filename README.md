# Nokia TA-1278

[简体中文](README.zh-CN.md)

Firmware research for the **Nokia 215 4G (2020), TA-1278**, version **17.00.17.02**.

The goal is to recreate its interface, mainly messaging, the browser, and music playback. Most assets are decoded. App behavior is partly mapped; unknown details and design choices are marked. This is not a working phone emulator.

## Contents

- [App behavior and reference models](nokia-ta1278-research/analysis/ui/apps/README.md)
- [Findings and open questions](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [Fonts](nokia-ta1278-research/analysis/ui/fonts/font-preview.html) and [assets](nokia-ta1278-research/analysis/ui/catalog.html) — open locally after cloning
- [Firmware structure](nokia-ta1278-research/analysis/firmware-structure.md) and [sources](nokia-ta1278-research/sources.md)

Most research notes are in Chinese.

## Firmware

Includes original `boot0`, `boot1`, `kernel`, `user`, `mmi`, and BootROM files. Here, `user.bin` contains system app code. Private storage and full NAND/NV dumps are excluded. See the [file list and SHA-256 hashes](nokia-ta1278-research/firmware/public-system-manifest.json).

## Setup

Requires macOS, Python 3.9+, Pillow, fontTools, Node.js, and Xcode Command Line Tools.

```sh
cd nokia-ta1278-research
shasum -a 256 -c PUBLIC_SHA256SUMS
python3 tools/prepare_offline_workspace.py
```

See [rebuild steps](PUBLICATION.md) and [third-party notices](THIRD_PARTY_NOTICES.md). No phone connection is needed for offline analysis.
