# 发布范围与复现

2026-10-25。仓库保持原研究目录结构，避免改动采集原件或使偏移索引失效。Git 采用白名单管理；私人采集文件和临时分析文件现已归档到工作区外的同级 `ta-1278-source/workspace/`，保留相对路径。公开系统原件仍在仓库。

## 包含

- 6 个实机读取的原始系统/BootROM 文件，总大小与逐文件 SHA 见 `firmware/public-system-manifest.json`。
- 设备、协议、采集和逆向报告，本项目 Python/C/汇编脚本。
- 仅从系统镜像派生的字体、图像、声音、文案、词库块、控件/事件索引和定向反汇编。
- 原始 kernel 的 keymap 和初始化数据切片，供离线分析使用；不包含个人 NV 或用户自学习词库。
- 外部框架固定提交、来源 URL 与保存文本 SHA 清单；公开参考源码需自行取回，不将其冒充本项目源码。

## 仅本地保留

完整物理 NAND/OOB、NV、逻辑分区备份、采集会话、USB/主机原始日志、donor PAC、第三方加载器、编译产物和重复工作副本。完整本地 `SHA256SUMS` 也不提交；仓库使用 `PUBLIC_SHA256SUMS` 校验发布文件。

大型字形轮廓 JSON、人工 ELF、GitHub 全仓树索引及含主机命令路径的生成日志可重建，不提交。报告指向被排除文件时，应按本节理解为本地证据或重建产物，而非漏传系统固件。

## 复现步骤

已验证环境：macOS arm64，Python 3.9，已有 Pillow/fontTools、Node.js、Xcode `xcrun llvm-objdump`。未承诺 Linux/Windows 的反汇编命令兼容性。依赖不会自动安装。

```sh
cd nokia-ta1278-research
shasum -a 256 -c PUBLIC_SHA256SUMS
python3 tools/prepare_offline_workspace.py
python3 tools/analyze_system.py
```

`prepare_offline_workspace.py` 校验发布清单后复制五个系统文件到既有脚本约定的工作目录；已有不同内容会拒绝覆盖。仅操作本地文件。

框架语义步骤需要 `tools/upstream-reference/mmi-framework/` 中的 63 个参考文本。清单 `provenance.json` 指向固定提交，版权保留。显式运行以下命令会联网下载这些文本并校验，不会编译或执行它们：

```sh
python3 tools/fetch_framework_reference.py
python3 tools/rebuild_offline_ui.py
python3 tools/rebuild_app_analysis.py
```

源文本编码/换行规范化后必须匹配此前记录的 SHA；无法匹配则停止，不能忽略校验。取回参考文件后，14 步流水线完全离线运行。它会重建轮廓 JSON、人工 ELF、资源目录及验证记录，某些报告已附带的派生文件可能随工具版本重新编码。

实机采集工具依赖另外准备的 libusb、上游工具和特定 RAM 加载器。它们保留本次实测地址与版本检查，不是可对任意 Nokia 运行的通用命令；详见协议报告。仓库复现不需要重新采集，也不自动接触手机。

## 发布检查

首次发布时（35 个参考文本版本），已从暂存文件单独导出临时目录，成功校验/准备六份原件、实际联网取回并验证 35 个固定参考文本，运行 analyze_system.py 和全部 14 步流水线。未借用被排除的工作副本或私人分区。结果见 [publication-validation.json](nokia-ta1278-research/analysis/ui/publication-validation.json)。

本轮新增应用逆向：本地已核验全部 63 个参考文本哈希，7 步应用流水线及 75 项检查通过；隔离重建的 79 个确定性输出一致。具体状态见 `analysis/ui/apps/rebuild-log.json` 和 `contract-validation.json`。历史发布验证记录保持原样，不冒充本轮重新联网下载记录。

隔离复现：`python3 tools/verify_app_reproduction.py`。它从 Git 可见文件创建临时副本，使用已核验的本地参考文本，准备公开系统原件并重建应用分析；不读取私人 NAND/NV，不联网。结果写入 `analysis/ui/apps/reproduction-validation.json`。

## 桌面程序阶段

Rust 源码在 `crates/`，运行及模块说明见根 README 和 `docs/architecture.md`。运行程序不需要 Python、LLVM、私人备份或参考源码。旧 `refresh_checksums.py` 用于采集阶段的完整本地树；归档后应分别验证仓库 `PUBLIC_SHA256SUMS` 与外部 `archive-manifest.json`，不要重写旧采集基线。

参考文本已归档。本机可运行 `python3 tools/restore_framework_reference.py` 从外部档案仅恢复清单中 63 个公开参考文本（逐文件核验）；新机器仍使用 `fetch_framework_reference.py`。两者均不执行这些参考代码。
