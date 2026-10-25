# Nokia TA-1278 固件提取与交互逆向

针对自有 Nokia TA-1278 / Nokia 215 4G（2020）的实机采集与离线分析。系统版本 **17.00.17.02**。完整 NAND 已在本地双读校验；仓库发布经过范围筛选的原始系统组件、分析脚本、资源及研究报告。

**资源主体已解码，完整交互尚未恢复。** 已确认、公开资料推断和当前无法恢复的内容分别标注，不能把本项目当作可启动原厂系统的模拟器或通用刷机包。

## 阅读入口

- [研究概览与设备证据](nokia-ta1278-research/README.md)
- [逆向成果与未恢复项](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [交互实现依据](nokia-ta1278-research/analysis/ui/interaction-spec.md)
- [固件结构](nokia-ta1278-research/analysis/firmware-structure.md)、[来源](nokia-ta1278-research/sources.md)
- [素材目录](nokia-ta1278-research/analysis/ui/catalog.html)、[字体预览](nokia-ta1278-research/analysis/ui/fonts/font-preview.html)（克隆后用浏览器打开 HTML）
- [发布范围与复现](PUBLICATION.md)、[第三方声明](THIRD_PARTY_NOTICES.md)

## 已发布原始固件

`nokia-ta1278-research/firmware/original/` 中包含 boot0、boot1、kernel、user、mmi 五个系统组件和一份 BootROM。`user.bin` 是系统应用代码镜像，不是用户数据分区。文件未改写，每份大小与 SHA-256 见 [系统发布清单](nokia-ta1278-research/firmware/public-system-manifest.json)。

整片 NAND/OOB、NV、完整逻辑分区备份、采集工作目录及第三方 donor 固件未纳入仓库。报告中出现这些本地证据路径，并不意味着对应数据已发布。

## 离线准备

已有 Python 3.9+、Pillow、fontTools、Node.js 和 Xcode Command Line Tools 的 macOS 上：

```sh
cd nokia-ta1278-research
shasum -a 256 -c PUBLIC_SHA256SUMS
python3 tools/prepare_offline_workspace.py
```

完整重跑还需要从固定版本取回 35 个公开参考文本，详见 [复现步骤](PUBLICATION.md#复现步骤)。不需要连接手机。脚本不会安装软件或执行固件；采集工具是独立研究工具，不属于上述离线准备流程。
