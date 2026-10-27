# Runtime resources

These files are part of the application. Building, testing and packaging do not need the separate research archive or a connected phone.

| Directory | Content |
|---|---|
| `fonts/device/` | Three fonts derived from the original handset firmware |
| `fonts/subsets/` | Noto fallback fonts used by the renderer |
| `icons/` | Only the handset PNG resources referenced by the application |
| `audio/` | Sound bank, sound bindings, Snake cues and selected tones |
| `input/t9/` | Original T9 ARM code slice, character tables and Chinese/English databases |
| `input/` | Public dictionaries and their licenses for the optional independent predictor |
| `ui/` | Runtime palettes, alarm/message tables and the owner-selected wallpaper |
| `snake/` | Runtime maps and timing tables |

`runtime.json` records the SHA-256 of each resource copied out of the research tree. Its `archive_source` fields are historical provenance paths, not build inputs. The copied bytes were checked against the originals. `sources.json` records the public font/dictionary downloads. Adjacent manifests retain source hashes and technical provenance; old firmware paths refer to the external archive.

## Shared control icons

The contact picker (`00455`), backspace (`01152`) and battery (`00542`) use the decoded handset PNGs at native size. Their resource IDs are `0x400001c8`, `0x40000481` and `0x4000021f`. Softkey images keep their padding at y=287; the battery image starts at (210, 4). The desktop battery is a fixed full-battery placeholder, not a hardware measurement.

SMS recipient focus uses the contact picker. Body focus uses the existing menu image (`01160`), as seen in the separate handset captures. The shared blitter applies PNG alpha once. Pencil (`01204`), radio markers (`01294`/`01295`) and Close (`01153`) also use native artwork. Pencil and radio alpha masks take the active theme color.

## Rights

Original handset fonts, icons, tones, the sound bank, maps and T9 resources retain their original rights. They are not relicensed under this project's code or the libraries used to render them. No full firmware image is needed or included in the current application tree. This separation does not remove images from earlier Git history.

The green owner-selected wallpaper is not claimed to be Nokia factory art. Its artist and license are unknown; its provenance is in `ui/owner-wallpaper.json`.

| Public resource | Source and license |
|---|---|
| Noto Sans CJK SC | Noto CJK, SIL OFL; `fonts/NotoSansCJK-LICENSE.txt` |
| Noto Sans Symbols 2 | Google Fonts, SIL OFL; `fonts/NotoSansSymbols2-OFL.txt` |
| Noto Emoji | Google Fonts, SIL OFL; `fonts/NotoEmoji-OFL.txt` |
| Simplified pinyin | Rime pinyin-simp, derived from Android Pinyin IME; Apache 2.0, adjacent license and authors |
| English frequencies | hermitdave / Dave, FrequencyWords, 2018 OpenSubtitles data; content CC BY-SA 4.0, not the source code's MIT license |
| Stroke dictionary | Rime stroke; LGPL-3.0, adjacent license and authors |

Public dictionary files retain their upstream text. `Builtin` is an optional independent predictor; the desktop uses the original T9 resources through a bounded ARM interpreter. Private user learning is saved in the app profile, not imported from handset dumps.

## Maintenance

Fonts are already converted and checked in. `python3 scripts/check-assets.py` verifies resource hashes and binds the saved coverage report to the exact font files. It uses only the Python standard library and does not modify resources. CI never converts fonts or installs fontTools. Normal Rust builds use checked-in resources and do not need Python or fontTools.

Only when replacing fonts, use local `fonttools==4.60.2` to regenerate subsets and coverage, then commit the resulting fonts and manifests:

```sh
python3 scripts/subset-fallback-fonts.py
python3 scripts/check-assets.py --fonts
```

Modified Noto subsets have distinct family names and retain OFL notices.

Build-time compression is lossless and checked by roundtrip decoding. T9 guest code is interpreted; it is not loaded as host machine code. Full resource extraction and disassembly tools live in the separate research archive.
