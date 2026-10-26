# 第三方来源与权利说明

本仓库包含原厂系统镜像及从中提取的字体、媒体、词库和代码片段。发布这些研究样本不代表拥有其中第三方内容的版权，也没有将原厂内容重新授予开源许可证。

`tools/decode_abm.py` 的 ABM 解码部分基于 Crawlerop/chinamobiletools 的 sprd_abm.py 改编（固定提交 249b0156f146c85c4f6b7a03bf84820d17b7feb2）。上游 MIT 版权与许可全文见 [chinamobiletools-MIT.txt](third-party-licenses/chinamobiletools-MIT.txt)。

W19 MMI/T9 框架参考仅发布 URL、固定版本和文本哈希；不将整个参考仓库认定为开放许可，不再分发这些参考源码。取回的文件应保留其原有版权声明。其他资料、加载器和开源项目的来源及适用边界见 [sources.md](nokia-ta1278-research/sources.md)。

仓库尚未为其余原创代码或文档指定统一开源许可证。

桌面程序新增 Noto CJK / Symbols 2 / Emoji（SIL OFL）、Rime 简体拼音词库（Apache 2.0）和 FrequencyWords 英文词频数据（CC BY-SA 4.0，不是其代码的 MIT 许可）。来源固定版本、SHA-256、作者与完整许可见 [assets/README.md](assets/README.md) 和相邻许可文件。新九键算法是复刻程序实现，不是原机 T9 的恢复结果。

- Rime stroke dictionary: rime/rime-stroke, revision `1e8fff9b9494ddec23b0cbc526bcfd8171a6fd48`. Original text, authors and LGPL-3.0 license retained under `assets/input/`. Used as replacement five-stroke data, not original Nokia input code. Runtime ranking also uses the existing pinyin frequencies.

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
