# Nokia TA-1278

[English](README.md)

**Nokia 215 4G（2020），TA-1278** 固件研究，版本 **17.00.17.02**。

目标是复刻手机界面，重点是短信、浏览器和音乐播放。大部分素材已解码，应用逻辑已部分恢复；未知细节和复刻设计均有标注。目前不是可运行的手机模拟器。

## 内容

- [应用逻辑与参考模型](nokia-ta1278-research/analysis/ui/apps/README.md)
- [研究结果与未解决问题](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [字体](nokia-ta1278-research/analysis/ui/fonts/font-preview.html)与[素材](nokia-ta1278-research/analysis/ui/catalog.html)：克隆后在本地打开
- [固件结构](nokia-ta1278-research/analysis/firmware-structure.md)与[来源](nokia-ta1278-research/sources.md)

## 固件

包含原始 `boot0`、`boot1`、`kernel`、`user`、`mmi` 和 BootROM。这里的 `user.bin` 是系统应用代码。私人存储和完整 NAND/NV 镜像不在仓库中。详见[文件与 SHA-256 清单](nokia-ta1278-research/firmware/public-system-manifest.json)。

## 准备

需要 macOS、Python 3.9+、Pillow、fontTools、Node.js 和 Xcode Command Line Tools。

```sh
cd nokia-ta1278-research
shasum -a 256 -c PUBLIC_SHA256SUMS
python3 tools/prepare_offline_workspace.py
```

详见[复现步骤](PUBLICATION.md)与[第三方声明](THIRD_PARTY_NOTICES.md)。离线分析无需连接手机。
