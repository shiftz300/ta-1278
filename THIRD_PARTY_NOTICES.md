# 第三方来源与权利说明

本仓库保留程序运行所需的原机字体、图标、媒体、词库和 T9 代码片段；完整固件、分析和采集工具已迁至独立研究归档。这些资源的权利仍属于原权利人，不会因迁移或运行库封装而获得新的开源许可。

仓库尚未为其余原创代码指定统一开源许可证。资源来源和校验见 [assets/README.md](assets/README.md) 与相邻清单。

桌面程序新增 Noto CJK / Symbols 2 / Emoji（SIL OFL）、Rime 简体拼音词库（Apache 2.0）和 FrequencyWords 英文词频数据（CC BY-SA 4.0，不是其代码的 MIT 许可）。来源固定版本、SHA-256、作者与完整许可见 [assets/README.md](assets/README.md) 和相邻许可文件。上述公共词库仅供可选替代预测器使用；桌面程序已切换到 `assets/input/t9/` 中的原机例程与字库，通过本项目的有界 ARM 解释器运行。原机 T9 代码、字符表及字库属于固件衍生资料，不适用 Rime、FrequencyWords 或本项目自编代码的许可；权利归其原权利人，见资源清单与固件发布边界。

- Rime stroke dictionary: rime/rime-stroke, revision `1e8fff9b9494ddec23b0cbc526bcfd8171a6fd48`. Original text, authors and LGPL-3.0 license retained under `assets/input/`. Used as replacement five-stroke data, not original Nokia input code. Runtime ranking also uses the existing pinyin frequencies.

## Browser scripts

- rquickjs, rquickjs-core and rquickjs-sys 0.14.0 (MIT), unmodified, [upstream](https://github.com/DelSkayn/rquickjs). The bundled QuickJS-NG engine has its own MIT copyright notice. Both texts are retained in `third-party-licenses/resources/rquickjs-MIT.txt` and `quickjs-ng-MIT.txt`, and included in the app package. The adapter evaluates source only; it does not load untrusted QuickJS bytecode.

- RFD 0.17.2 (MIT), unmodified, [upstream](https://github.com/PolyMeilex/rfd). Used for Windows/Linux native folder selection; macOS retains its existing helper. License retained in `third-party-licenses/resources/rfd-MIT.txt` and packaged notices.

## Resource compression

- miniz_oxide 0.8.9 (MIT), unmodified. Build-time packing and bounded runtime decompression use the already pinned dependency. Its copyright and license are retained in `third-party-licenses/resources/miniz_oxide-MIT.txt` and bundled with the app. Compression does not change the fonts, dictionaries or original sound bank and does not change their licenses.

## Audio output

- Rodio 0.21.1 (MIT OR Apache-2.0), [upstream source](https://github.com/RustAudio/rodio/tree/v0.21.1). MIT text retained in `third-party-licenses/audio/rodio-MIT.txt`.
- CPAL 0.16.0 (Apache-2.0), [upstream source](https://github.com/RustAudio/cpal/tree/v0.16.0). Text retained in `third-party-licenses/audio/cpal-Apache-2.0.txt`.
- Symphonia 0.5.5 and its core/metadata/MP3/PCM/RIFF components (MPL-2.0), unmodified. [Corresponding source](https://github.com/pdeljanov/Symphonia/tree/v0.5.5) and exact crates/checksums in Cargo.lock. License text retained in `third-party-licenses/audio/symphonia-MPL-2.0.txt`.

These audio components are host implementations, not recovered Nokia code. Other transitive dependency versions and source checksums are recorded in Cargo.lock.

## MIDI synthesis

- OxiSynth, oxisynth-chorus and oxisynth-reverb 0.1.0 (LGPL-2.1), unmodified, [upstream](https://github.com/PolyMeilex/OxiSynth). The exact registry source archives and SHA-256 manifest are retained in `third-party-licenses/audio/sources/` and packaged under `Contents/Resources/Notices/audio/sources/`. The LGPL text is `oxisynth-LGPL-2.1.txt`.
- soundfont 0.1.0 (MIT), unmodified, from the same upstream repository. License, exact source archive and checksum are retained alongside OxiSynth.
- midly 0.5.3 (Unlicense), unmodified, [upstream](https://github.com/negamartin/midly). Declared license text, exact source archive and checksum are retained alongside OxiSynth.

The application source in this checkout can be modified and rebuilt to replace these LGPL libraries; reverse engineering to debug modifications to these libraries is permitted. Unpack an archive, modify the library, add a local `[patch.crates-io]` path in the root Cargo.toml and rebuild with `cargo build --release -p phone-desktop` (refresh Cargo.lock when changing the source). `scripts/package-macos.sh` then packages and ad-hoc signs the replacement executable. Keep the matching application source and these notices with any distribution.

`assets/audio/gm1_A21.sf2` is original firmware data, copyright 2010 SPRDCorporation according to its metadata. The original MIDI and MP3 resources are also firmware data. None is relicensed under the synthesizer's LGPL or the parser's license. Recovered notes/samples do not imply recovery of the proprietary DSP implementation.
## Owner-selected artwork

`assets/ui/owner-wallpaper.png` contains the handset owner's selected background. Its original artist and license are not established. It is not represented as Nokia factory artwork or original project artwork. Provenance and pixel-decoding details are in `assets/ui/owner-wallpaper.json`.
