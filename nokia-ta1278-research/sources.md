# 来源与可信度登记

检索日期：2026-10-25，Asia/Shanghai。来源的型号级证据不自动等于本机版本匹配。未向来源网站上传任何固件、设备数据或唯一标识。

## 官方资料

| ID | 来源 URL | 适用范围 / 得到的证据 | 可信度与限制 |
|---|---|---|---|
| S01 | https://www.hmd.com/en_in/support/nokia-215-4g-user-guide/keys-and-parts | 明列 TA-1278 属于 Nokia 215 4G；方向/软键等外观说明 | 高；产品手册，非实机版本 |
| S02 | https://www.hmd.com/en_int/nokia-215-4g/specs | T117、Series 30+、64/128 MB、QVGA、Micro USB | 高；产品级规格，地区和硬件修订未锁定 |
| S02b | https://www.hmd.com/en_in/nokia-215-4g/specs | 印度官方页面也列 Model TA-1278 | 高；不能用型号直接认定中国地区 |
| S07 | https://www.hmd.com/en_in/support/nokia-215-4g-user-guide/keypad | 方向导航、字符选择、空格和大小写 | 高；本机语言与版本需确认 |
| S08 | https://www.hmd.com/en_in/support/nokia-215-4g-user-guide/messages | 新建短信、收件人与正文、插入符号/表情 | 高；无光标和计数实现细节 |
| S09 | https://www.hmd.com/en_in/support/nokia-215-4g-user-guide/browse-the-web | 存在浏览器章节，明确地区行为差异 | 高；不能断言本机浏览器引擎 |
| S10 | https://www.hmd.com/en_int/security-updates | 从产品手册“Software updates”链接跳转 | 官方；本次没有找到 TA-1278 完整镜像直链 |
| S11 | https://www.hmd.com/en_int/nokia-215-4g-2024/specs | 新版采用 T107 等不同规格 | 高；仅用于排除混淆，不适用于本机 |

## 开源作者原始资料与源码

| ID | URL | 版本/证据 | 结论与限制 |
|---|---|---|---|
| S03 | https://github.com/ilyakurdyukov/fpdoom | 查询时 main commit 296795c2cfa084b061a7529997a19e75f4f91e36；README blob e7643745ebe563670654b29ee53ca62141c3f423 | 测试表明确 TA-1278 / UMS9117 / GC9305 240×320 / 左软键；作者测试记录可信度较高，不是原厂采集认证 |
| S04 | https://github.com/ilyakurdyukov/spreadtrum_flash | commit 3d0582579f3521f35a4937b2e871312e4dd6c308 | 有 4G FDL+read_flash 流程；已编译并仅用于 RAM 加载与 BootROM 双读，未执行 NAND 写入/擦除 |
| S05 | https://github.com/ilyakurdyukov/spreadtrum_flash/blob/3d0582579f3521f35a4937b2e871312e4dd6c308/t117_fdl/README.md | blob 5c92ab7709634e6c3c752adc3e9e6311c33a71a7 | 自制加载器不支持 NAND 读写，仅 RAM/BootROM 功能，不能代替原厂 FDL |
| S06 | https://github.com/ilyakurdyukov/spreadtrum_flash/blob/3d0582579f3521f35a4937b2e871312e4dd6c308/fphelper_t117/README.md | blob d792933390570ae10388abbbb88cafd9e78b7ebe | 解析初始化/压缩段及 keymap/pinmap；无本次样本验证 |
| S12 | https://github.com/ilyakurdyukov/fpdoom/blob/296795c2cfa084b061a7529997a19e75f4f91e36/ums9117/README.md | blob c6ed5f67e9d6786fe8d905c50d87a3c55de0ee23 | 要求先得到 kernel 及原厂 FDL；文档 extract 示例与 helper 当前 README 的 unpack 术语不同，未来应以源码解析参数为准 |
| S13 | https://github.com/ilyakurdyukov/spreadtrum_flash/blob/3d0582579f3521f35a4937b2e871312e4dd6c308/unpac/unpac.c | blob 7241917cea6c05f4fb562fb7c029bd3abbf69bee | PAC 结构与 CRC 实现，可作离线解析依据；未编译/执行 |

源码快照在 tools/upstream-reference/，下载渠道为 GitHub connector 的文本读取。保存了许可证；初轮仅保存必要文本，后续另下载了已核对 SHA-256 的开源 RAM 加载器，见下方登记。每个本地文件的实际 SHA-256 见 SHA256SUMS；Git blob SHA 不等于 SHA-256，也不提供手机固件真实性证明。

## 固件候选与未采用理由

| 来源 / URL | 文件或版本线索 | 型号与可信度评估 | 本次处理 |
|---|---|---|---|
| https://stackoverflow.com/questions/67031568/how-to-modify-checksum-after-making-changes-in-nokia-215-4g-firmware | Nokia_215_4G_readed_test.pac；2021 年提问者提供的读回文件 | 第一手使用描述，但没有明确 TA-1278、地区、未修改状态、官方哈希；可能包含备份数据 | 未下载，仅保留来源 |
| https://www.mediafire.com/file/ok2c0qodao5fld1/Nokia_215_4G_readed_test.pac/file | 上述问题中的外链；未验证在线状态/大小/哈希 | 不可视为官方包或本机同版本 | 后续尝试访问页面失败；未下载文件 |
| https://4pda.to/forum/index.php?showtopic=1007593&st=1860 | Nokia 215 4G_12.00.17.01_TA-1272_EN_ARB_RU_HEBREW.spd、其他 TA-1272 语言/15.00.17.01 线索；PAC/FOTA 讨论 | 查询结果可见，但直接页面返回 403；TA-1272 不等于 TA-1278；混有 2024 机型 | 不下载、不采用刷机流程 |
| https://4pda.to/forum/index.php?showtopic=1007593&st=1480 | 用户报告 TA-1278 的 12.00.17.01 与升级问题 | 单个用户经历；不是发行包、不是本机版本 | 仅线索 |
| https://forum.gsmhosting.com/vbb/f272/nokia-215-4g-security-damaged-3232397/ | 用户报告 TA-1278 刷写后 NV/网络损坏，13.00.17.00 | 第一手故障报告，不是只读验证 | 不执行该流程 |
| https://infinity-box.com/news/ | BEST3 v3.00 列出 TA-1278 与 DIAG/Read info、格式化等功能 | 软件厂商宣称，只能证明产品声称支持；不证明原厂或完整只读采集 | 不安装/不运行 |

本次未找到既明确 TA-1278、又可验证来源和版本的完整/差分官方包。不能据此断言互联网上绝不存在包。未注册付费站、购买软件或请求他人提供固件。

## 主机连接诊断参考

- [spreadtrum_flash release 1.20260403](https://github.com/ilyakurdyukov/spreadtrum_flash/releases/tag/1.20260403)：实际下载 t117_fdl1.bin，7380 bytes，SHA-256 062d5e1a298c0e378eeefcc67dddba6fd4d6be10a683511c849ec59c17fb7414，匹配发布 asset digest。来源属于开源工具作者，不是手机原厂固件。源码审查及用途见 analysis/protocol-report.md。
- [spreadtrum_flash issue #43 作者说明](https://github.com/ilyakurdyukov/spreadtrum_flash/issues/43#issuecomment-5249968147)：UMS9117 不装电池时的启动枚举与 1782:4d00；讨论实机为 TA-1395，作为排错线索，不作为 TA-1278 全固件读取证明。
- [issue #32 作者提供固件线索](https://github.com/ilyakurdyukov/spreadtrum_flash/issues/32#issuecomment-3435929678)：指向 4PDA tfload.rar，但适用 NAND/机型需验证、下载要求注册；本次未下载、未执行其加载器。
- Apple 官方 [Using Disk Arbitration Notification and Approval Callbacks](https://developer.apple.com/library/archive/documentation/DriversKernelHardware/Conceptual/DiskArbitrationProgGuide/ArbitrationBasics/ArbitrationBasics.html)：说明挂载审批回调可返回 DADissenter 取消挂载。它没有为整个 USB 存储连接过程提供硬件级无写入保证。
- 本机 SDK：/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/DiskArbitration.framework/Headers/{DiskArbitration.h,DASession.h,DADisk.h}。只读审查，未更改系统设置、编译或安装拦截器。

## 检索与失败

关键词包括 `TA-1278 Nokia chipset firmware`、`TA-1278 pac`、`TA-1278 固件`、`Nokia 215 4G firmware pac`、`Nokia 215 4G spd_dump`、官方型号手册与 GitHub 仓库。搜索结果曾混入 2024/2nd Edition，已排除。HMD write-text 路径不可访问，改用已打开的 keypad；国际手册入口超时，印度官方同型号手册成功。4PDA 只使用可见搜索摘录作为候选，不把受阻内容假装已下载。


## 05:36 后新增候选与本地分析工具

- https://4pda.to/forum/index.php?showtopic=1007593&st=1760 ，帖子 #1778，作者 zuyj，2023-10-21 发布、可见编辑日期 2025-08-09：检索缓存文字列 `215_4G.rar`（106.28 MB）、NBX、Queen2_BASIC，并明确包括 TA-1278。适用型号是社区作者宣称，尚无包内证据；直接访问返回 403，未取得附件 URL/文件/哈希。列出的版本解释示例是 Nokia 225，不能拿来认定 215 包内版本。此线索比只写 215 4G 的 PAC 更明确，但尚不满足可信原厂包认定条件。
- https://stackoverflow.com/questions/67031568/how-to-modify-checksum-after-making-changes-in-nokia-215-4g-firmware 的 PAC 链接复查仍缺 TA/版本/原始性；当时 MediaFire 页面工具返回不可访问；后续经本地 HTTPS 访问成功，见下方新增记录。
- 本机 `xcrun --find llvm-objdump` 找到 `/Library/Developer/CommandLineTools/usr/bin/llvm-objdump`。初轮仅按 PATH 未发现，不能误报为整个系统未安装。本次用于本地 BootROM 小范围反汇编，无需新装 Ghidra/Capstone。
- 本机 BootROM 是实机采集证据，来源/双读 SHA-256 详见 firmware/README.md；不是第三方固件下载，也没有上传至上述网站。

## 后续实际取得的 PAC 与 NAND 参考

- [社区讨论原链接](https://stackoverflow.com/questions/67031568/how-to-modify-checksum-after-making-changes-in-nokia-215-4g-firmware) → [MediaFire PAC 页面](https://www.mediafire.com/file/ok2c0qodao5fld1/Nokia_215_4G_readed_test.pac/file)。本地 HTTPS 下载成功，39,204,210 bytes；SHA-256 `cff2f3a48a37879823bc1e997aeb3ae77f485d0c84a2e45e415209701f1b4ba2`。包头与数据 CRC 均通过。社区原始性/官方签名未验证，包名不是 TA-1278 兼容证明。包内 NQ2 / 23QEN72RU00 是 donor 区域/型号线索，本机版本由实机 kernel 另行取得。
- donor FDL2：75,540 bytes，PAC 偏移 0x1abe4，SHA-256 `3c6699d6add21b58f2b87e84e6fdf847e39d9d2ef70f9b19d6ba9d59e4adb7bd`。在本机 RAM 0x80100000 执行 ACK 后，四个系统头和完整组件读回成功。证据等级：下载来源中等/非官方；本次读取兼容性为实机验证，不推及刷写。
- [spreadtrum_flash issue 43](https://github.com/ilyakurdyukov/spreadtrum_flash/issues/43)：开源作者说明 custom FDL1 配合原厂 FDL2；Itel 与 Nokia 的串行/并行 NAND 差异说明不可只凭芯片 ID 互用。讨论中 TA-1395 的成功不冒充 TA-1278 成功；本次兼容结论来自独立实测。
- [fplinux NAND 只读驱动](https://github.com/fplinux/fplinux/blob/b3d012587fe41bcd92244162c798a7bcf1f418d3/platforms/ums9117/linux/overlay/drivers/memory/ums9117/ums9117-nandc-ro.c)，固定 commit b3d012587fe41bcd92244162c798a7bcf1f418d3。源码 GPL-2.0，展示 UMS9117 NANDC 的 READ_ID、状态与物理页/OOB 读取路径。其测试机型包括 TA-1618/INOI，并非 TA-1278；仅作为控制器研究参考，未在本机运行 Linux 驱动。
- [chinamobiletools](https://github.com/Crawlerop/chinamobiletools/tree/249b0156f146c85c4f6b7a03bf84820d17b7feb2/spd)，固定 commit 249b0156f146c85c4f6b7a03bf84820d17b7feb2，MIT。dumpsres.py、dumptres.py、sprd_abm.py 用于核对 SRES/TRES/ABM 布局。未安装其依赖；本地写了有边界检查的解析/PNG 输出脚本，并记录未支持的资源。
- [旧平台 FDL NAND 源码](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/blob/d48d55e028c13b6f50c291cbb7ea29201c3c8432/fdl_bootloader/fdl2/src/fdl_nand.c)：只作命名与逻辑参考，不证明当前 UMS9117 二进制支持其 raw mask。当前 donor FDL2 以实际反汇编为准；没有盲发 0x40000000 raw 读取。

## Local NAND evidence and RAM debug build

- Live metadata: analysis/work/debug-fdl2-20261025T095036+0800/. Geometry: 1024 blocks, 64 pages/block, 2048 main + 64 spare bytes/page. Selected table ID A1 A5; no vendor-name inference.
- RAM hook sources: tools/fdl2_debug_hook.c, fdl2_debug_entry.S, build_debug_fdl2.py. Compiled with installed Apple clang. Uploaded hook SHA-256: 3918bf664c45576312a8118957701f9688c6ec3fe22d9403e6031054cf32cd03. Patched FDL2 SHA-256: 2aee3deba04fd96a32de4e0ed4fcdb45038ef075cbb9c6832eb08249f22bd2d7. Both RAM uploads were read back and checked before execution.
- Physical page samples: analysis/work/nand-probe-20261025T0952/, pages 0/1/64/65535 read twice; first two main pages match the independently saved boot0.
- Full acquisition evidence: analysis/work/nand-20261025T095230+0800/acquisition.json. Completion is determined by its status and hashes, not this provenance entry. All device data remains local.

## 2026-10-25T10:28:22.732510+08:00 离线 UI 证据补充

本轮格式与实现结论直接来自已双读验证的本机 mmi.bin/user.bin；精确 SHA、偏移和指令见 analysis/ui/report.md。未使用新下载固件或新引入解析程序。ABM 仍基于本地已归档 MIT 上游解码器，修正 ABMR 尾部单字节边界；SFON/字符表/菜单解析为本项目本地脚本。没有用相似机型源码代替本机证据。


## 2026-10-25 离线资源恢复增补

- 字形格式来源：本机 user.bin 中 0x88033768、0x88033b2c、0x8855f678、0x8855fe64、0x885602c2 的静态代码；与全部三份 SFON 逐项校验。可信度：本机直接证据。
- BMP 0x1003 格式：509 个同机资源的行/字节校验，加 NOKIA logo 和模式文字目视核对；不冒充第三方格式规范。SABM 通道仍标暂定。
- 窗口名称：本机注册代码和 0x885f234e 查表逻辑；名称 ID 为 1-based。CSCS 与 locale 元数据来自 mmi.bin。
- 字体导出使用电脑已有 fontTools/Pillow；仅执行本地解析脚本，不执行固件。公开格式检索未提供此次恢复结论，没有下载新固件、安装工具或上传文件。


## 2026-10-25：框架语义、JPEG、T9 与官方行为补充

- 本机直接证据：SJPG 的四份量化表和四份 Huffman 表在 kernel 中精确匹配；输入熵流独立遍历通过，11 张已重建。表偏移/哈希见 analysis/ui/sjpg-extraction.json。可信度：本机字节验证；输出 IDCT 舍入未与手机逐像素比对。
- 本机直接证据：user 中明确 T9/T9CC/T9AW V07.04.04.01；DATA 块与初始化段词库头、加载函数共同支持词库重建。繁体长度不一致明确保留，不声称完整。见 analysis/ui/t9-resource-analysis.json。
- [公开 W19 MMI 框架](https://github.com/obaidi2005/ZW217_W19_V4.1_yisai/tree/d48d55e028c13b6f50c291cbb7ea29201c3c8432)，固定提交 d48d55e028c13b6f50c291cbb7ea29201c3c8432。35 个 C/头文件保存在 tools/upstream-reference/mmi-framework/，逐文件 URL、SHA-256 在 provenance.json。适用性：不同产品/版本的 Spreadtrum MMI 架构参考，不是 TA-1278 官方源码；没有确认全仓开源授权，保留原版权。仅阅读，不编译执行。可信度：名称/结构相关证据中等，必须与本机指令或资源交叉验证；已经发现 END_WIN 枚举版本差异。
- [HMD 2020 指南机型范围](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/keys-and-parts)：明确包含 TA-1278；[键盘](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/keypad)、[信息](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/messages?locale=en-US)、[浏览器](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/browse-the-web)、[个性化](https://www.hmd.com/en_int/support/nokia-215-4g-user-guide/change-the-look-of-your-home-screen)。可信度：官方产品行为说明高；不覆盖 17.00.17.02 全部内部实现，浏览器明确存在地区差异。详细适用边界见 analysis/ui/official-behavior-baseline.md。
- 本次没有获得新的官方 TA-1278 固件包或完整源代码。公开网络查询只请求网址/源码文本，没有上传固件、资源、硬件标识或私人信息。
