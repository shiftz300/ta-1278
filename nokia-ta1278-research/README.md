# Nokia TA-1278 固件研究

最新应用交付：[短信、浏览器与音乐的复刻合同](analysis/ui/apps/README.md)。含原生菜单/事件证据、三份应用规格及 37 项检查；未恢复细节明确作为复刻策略或后端接口。

仓库发布说明见 [PUBLICATION.md](../PUBLICATION.md)：用户后续授权上传原始系统固件，已纳入五个系统组件和 BootROM；整片 NAND/NV/用户存储仍只在本地保存。下文历史调查中的“未上传”描述当时状态。公开文件校验使用 `PUBLIC_SHA256SUMS`，完整本地清单为 `SHA256SUMS`。

**完整物理地址范围已采集：128 MiB NAND 主区 + 4 MiB OOB/spare，两遍整片读取的 SHA-256 一致。** 覆盖全部 65,536 页；使用原生控制器读取、ECC 标志关闭，未作外接编程器交叉验证。

已经从实机读出 boot0、boot1、kernel、user 应用镜像和 MMI 资源，全部双读 SHA-256 一致。内核软件版本字符串为 **17.00.17.02**。

当前扩展备份：20 个逻辑分区/启动区双读一致，共 123,113,472 bytes；状态 `all_mapped_parts_verified`。详见 [逻辑备份清单](analysis/logical-backup.md)。

最新离线交付：[完成情况与未恢复项](analysis/ui/reverse-engineering-status.md)、[交互实现依据](analysis/ui/interaction-spec.md)、[资源目录](analysis/ui/catalog.html)、[字体预览与下载](analysis/ui/fonts/font-preview.html)。三份 SFON 的 **22,349 个字形全部解码**；**2,824 / 2,826 图像条目**可预览（剩余两个为无像素的动态名称记录，49 个 SABM 通道仍暂定）；73 个媒体文件、55 个主题颜色、479 个 DATA 条目已导出。已提取 T9 7.04.04.01 的内置词库块，并重建两份英文拼接文件及一份简体容器。**完整交互仍未恢复**，繁体词库、动态菜单、焦点、短信完整计数和状态转换等逐项标明证据、卡点及验证条件。

## 已确认结果

- 不装电池，按住左软键接 USB：1782:4d00，SPRD3。开源 RAM FDL1 返回芯片 ID 0x98180001，32 KiB BootROM 双读一致。
- 从社区 Nokia 215 4G PAC 提取的 FDL2 在本机实测可读取系统；这验证了本次读取用途，不能证明 donor 固件适合刷入。包内标签指向 NQ2 / TA-1272 RU 线索，不冒充 TA-1278 官方包。
- 原厂系统采用 DHTB 组件；MMI 使用 SRES/TRES 资源容器。已解析 2,820 个图像条目、75 个音频条目、44 个字符串块及字体区。
- 已解码 2,228 个 ABM，另提取 509 个压缩 BMP、20 GIF、2 标准 BMP，49 个 raw SABM 暂定通道预览，以及五个动画容器的 43 帧、11 个 SJPG。此前 395 个失败项已修复末尾单字节读取；[菜单注册表图标](analysis/ui/mainmenu-contact-sheet.png) 已目视检查，不代表实际菜单排布。
- 提取 keymap、pinmap、初始化段；LCD 配置 ID 为 0x9305。user 镜像包含 Smart9Key、MMISMS、编辑控件和 Opera Mini 字符串；存在不等于 UI 可达或行为已还原。

| 实机组件 | 字节数 | SHA-256 |
|---|---:|---|
| boot0 | 31,284 | `aefd78512018752d3cca0046c19c192221f55c3e8308737db5c6976b8e53e001` |
| boot1 | 63,972 | `63aa1fc89d279c43119353607982ce2ee3d695bfc7486fc11d01553f25eaab9b` |
| kernel | 13,154,212 | `b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303` |
| user | 13,312,500 | `8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca` |
| mmi | 11,835,196 | `14c573027f375ea6b5f8ead2c6c432f2773f373faaa3c8a33261d0de9737886c` |

## 交付文件

- [设备识别报告](device-report.md)、[调查日志](research-log.md)、[来源与可信度](sources.md)
- [固件结构分析](analysis/firmware-structure.md)、[BootROM 结构](analysis/bootrom-structure.md)
- [获取方法与剩余条件](analysis/acquisition-plan.md)、[协议记录](analysis/protocol-report.md)
- [交互复刻证据](findings.md)、[逻辑备份清单](analysis/logical-backup.md)
- [物理 NAND 采集与结构](analysis/nand-report.md)
- [资源偏移索引](analysis/system/resource-index.json)、[按键矩阵](analysis/system/keymap.json)
- [逆向交付与限制](analysis/ui/reverse-engineering-status.md)、[交互实现依据](analysis/ui/interaction-spec.md)、[公开行为基线](analysis/ui/official-behavior-baseline.md)
- [资源浏览器](analysis/ui/catalog.html)、[验证记录](analysis/ui/validation.json)、[主题颜色](analysis/ui/color-resources.json)、[T9 资源](analysis/ui/t9-resource-analysis.json)

## 文件与操作边界

原件位于 firmware/original/；工作副本位于 analysis/work/，派生文件位于 analysis/extracted/。完整分区备份可能含私人数据，仅本地保存，不在报告展示。没有上传设备数据，也未执行 NAND program、erase、重分区、解锁或刷入 donor 系统。

用户已明确撤销最初的读写限制并授权调试。主机使用已有 clang、libusb、Python 和 xcrun llvm-objdump，没有安装驱动或未知 EXE。自编译调试扩展已在 RAM 实测运行，NAND 几何、NV 边界和物理页抽样均通过验证。

离线复现：`python3 tools/analyze_system.py`、`python3 tools/decode_abm.py`。实机流程必须区分 BootROM、custom FDL1、原厂 FDL2、调试 FDL2 四种状态，不能互换命令。具体采集参数保存在 acquisition.json；状态与完整性以清单为准。

早期 UI 提取流程：`tools/extract_ui_resources.py` → `tools/decode_abm.py` → `tools/analyze_ui_code.py` → `tools/build_ui_catalog.py` → `tools/validate_ui_outputs.py`，均用 Python 运行。代码分析使用已有 Xcode LLVM，目录/图片验证使用已有 Pillow 与 Node；无需联网或安装新工具。

最新完整离线流水线：`python3 tools/rebuild_offline_ui.py`。14 步均通过；格式说明、当前可恢复范围与无法确认项见 [最终离线交付](analysis/ui/reverse-engineering-status.md)。
