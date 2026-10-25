# 第三方来源与权利说明

本仓库包含原厂系统镜像及从中提取的字体、媒体、词库和代码片段。发布这些研究样本不代表拥有其中第三方内容的版权，也没有将原厂内容重新授予开源许可证。

`tools/decode_abm.py` 的 ABM 解码部分基于 Crawlerop/chinamobiletools 的 sprd_abm.py 改编（固定提交 249b0156f146c85c4f6b7a03bf84820d17b7feb2）。上游 MIT 版权与许可全文见 [chinamobiletools-MIT.txt](third-party-licenses/chinamobiletools-MIT.txt)。

W19 MMI/T9 框架参考仅发布 URL、固定版本和文本哈希；不将整个参考仓库认定为开放许可，不再分发这些参考源码。取回的文件应保留其原有版权声明。其他资料、加载器和开源项目的来源及适用边界见 [sources.md](nokia-ta1278-research/sources.md)。

仓库尚未为其余原创代码或文档指定统一开源许可证。

桌面程序新增 Noto CJK / Symbols 2 / Emoji（SIL OFL）、Rime 简体拼音词库（Apache 2.0）和 FrequencyWords 英文词频数据（CC BY-SA 4.0，不是其代码的 MIT 许可）。来源固定版本、SHA-256、作者与完整许可见 [assets/README.md](assets/README.md) 和相邻许可文件。新九键算法是复刻程序实现，不是原机 T9 的恢复结果。

- Rime stroke dictionary: rime/rime-stroke, revision `1e8fff9b9494ddec23b0cbc526bcfd8171a6fd48`. Original text, authors and LGPL-3.0 license retained under `assets/input/`. Used as replacement five-stroke data, not original Nokia input code. Runtime ranking also uses the existing pinyin frequencies.
