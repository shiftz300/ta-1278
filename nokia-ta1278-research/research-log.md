# 调查记录

日期均为 2026-10-25，时区 Asia/Shanghai。早期步骤保留命令和结果摘要；精确时间以 JSON 快照为准，未伪造逐条搜索时间。

## 初始调查（约 04:56–05:00）

1. `pwd`、`git status --short`、目录/AGENTS 检查：工作区 /Users/shiftz/dev/ta-1278 为空，不是 Git 仓库，无适用 AGENTS.md。没有需要覆盖的修改，没有本地固件。文件检查限于项目；未扫描用户私人目录或手机卷。
2. `sw_vers`、`uname -m`、`df -h .`、工具 PATH：macOS 27.0.1/26A434、arm64、约 572 GiB 可用。
3. 首次 `ioreg -a -p IOUSB -l -w 0` 失败，stderr 为 `ioreg: error: can't open file.`；原因未确认，不把失败作为设备不存在证据。改用文本输出并在持久化/显示前过滤标识信息。
4. `ioreg -p IOUSB -l -w 0`、IOUSBHostDevice/Interface 类查询均只见主机或为空；串口仅蓝牙与 debug-console。
5. `system_profiler SPUSBDataType -detailLevel mini` 无输出；`system_profiler -listDataTypes` 显示本机实际类型为 SPUSBHostDataType，改查后也仅主机总线。
6. 官方资料确认 TA-1278 产品归属；检索固件及开源项目，见 sources.md。未下载任何固件。

## 05:00:37 — 可复现沙箱快照

运行 `python3 nokia-ta1278-research/tools/capture_environment.py`。脚本不打开 USB/串口/存储设备；只查询系统信息，过滤含 serial/IMEI/UUID 等行。

- 保存 analysis/environment/20261025T050037+0800.json。
- CPU/内存 sysctl 返回 Operation not permitted，记录错误。
- USB 仍为空；检测到本地已有 libusb，不安装软件。

## 05:00:53 — 沙箱外只读复核

同脚本经环境审批在沙箱外运行，无 sudo，无管理员设置变更。

- 保存 20261025T050053+0800.json。
- Apple M5 Pro，24 GiB，575 GiB 可用。
- USB 结果与沙箱一致。无手机 VID/PID。

## 用户状态及工具审查

用户确认“正常开机，充电模式”，随后确认 USB 菜单只有充电和存储两项。没有要求反复切换或打开存储模式。普通关于页的版本信息未取得。

GitHub connector 读取上游源码/文档并保存文本到 tools/upstream-reference/。审查 read_flash、FDL 上传执行、t117_exec_workaround、NAND 限制和 PAC/压缩解析。未运行 spd_dump，未加载任何 FDL。HMD/开源/固件候选 URL 及版本见 sources.md。

## 05:03:19–05:06:19 — 经授权的特殊模式枚举观察

用户明确批准“同意，只检查 USB 枚举”。先启动 `python3 nokia-ta1278-research/tools/watch_usb.py`，再给出拔线、正常关机、按住左软键接线约 15 秒的步骤。

- 采样间隔 0.25 秒，最长 180 秒，保存 usb-watch-20261025T050319+0800.jsonl。
- 监听只记录初始主机树，没有记录到新 USB 设备。
- 用户随后回复“已操作”。无法精确对齐其按键动作与监听结束时间，因此不排除动作在窗口外或设备出现过短；不能宣称已成功进入下载模式。
- 回复后 05:06:38 在沙箱内、05:06:45 在沙箱外复核，均无设备 VID/PID 或接口。
- 没有发送维护协议、打开存储、启动下载器、改变分区或读取用户数据。

## 05:07:49 起 — 继续独立排查

用户表示设备无需日常使用，可继续调试。此授权允许继续调查，不将其解释为取消禁止刷写、擦除和读取私人数据的边界。

使用已有 libusb，经 tools/list_usb_libusb.py 查询设备列表和缓存描述符。脚本没有 libusb_open、claim 或传输调用。

- 沙箱内初始化成功，设备计数 0，记录 libusb-20261025T050749+0800.json。
- 05:08:55 沙箱外同样初始化成功、设备计数 0，保存 libusb-20261025T050855+0800.json。

## 当前停止点

系统注册表与 libusb 均未得到可用设备入口。尚不能区分线缆/转接器/USB 口与特殊模式未进入；不是已证实不支持 TA-1278。除此之外，还缺匹配原厂加载器、版本及可排除私人信息的系统读取范围。没有固件采集或镜像静态分析成功结果。

## 05:15 — 用户确认数据链路条件，准备第二次观察

用户明确回复“直连，线缆确认能传文件”，因此不再把仅充电线或扩展坞作为已知原因。05:15:12 重新采集，仍无 USB 子设备。

05:15:23 在沙箱外启动被动注册表监听，时长 600 秒，文件 usb-watch-20261025T051523+0800.jsonl。watch_usb.py 增加可选时长与起止标记，仍不打开任何 USB 设备。

准备在既有仅枚举授权内，请用户正常关机、拔线后取出可拆电池等待 10 秒再装回，按住左软键连接，观察 15 秒并报告屏幕状态。目的是排除没有完全断电而错过启动入口；此处只记录计划，尚无第二次操作完成或进入模式的结论。

## 05:17 — 第二次操作已完成，仍未枚举

用户回复“已操作……充电图标”。此时 600 秒的沙箱外监听仍在运行。读取日志仅有 watch_start 与初始主机树，无新的 USB 设备变化；这次用户回复发生在监听窗口内。

05:17:12 的 capture_environment.py 快照仍只有主机控制器；05:17:13 的 libusb 检查初始化成功、设备数 0。没有取得 VID/PID、接口或设备串口，没有确认下载模式成功。0.25 秒轮询仍不能排除更短瞬时枚举。

停止重复相同按键操作。下一诊断分支为正常开机后的存储模式，以区分正常数据接口与启动模式。查阅本机 Apple SDK DiskArbitration 头文件及 Apple 官方 mount approval 文档：挂载拒绝回调不等于硬件写保护，不能保证系统底层对存储绝无写入。本次未编译/安装挂载拦截器，也未修改系统挂载设置。

已明确告知用户普通存储模式可能被 macOS 自动挂载并写入少量文件系统元数据，单独请求此次连接诊断授权；在答复前不指示切换、不读取手机文件。原禁止刷写/擦除/访问私人数据约束保持。

## 后续授权更新 — 用户已备份，撤销原先读写限制

用户明确回复：“无所谓读写 我之前的要求作废 我已经备份所有的文件 请随意”。这是后于前述限制的直接用户授权。已据此继续正常存储模式连接诊断，不再为先前读写限制重复请求确认。没有因此执行无研究必要的擦除、格式化或刷写。

已指导用户正常开机、重新插线并选择存储模式；如 macOS 弹出初始化/格式化提示先取消。当前等待用户切换及枚举证据。

## 05:20–05:23 — 存储模式成功，修正启动方式

用户回复已选存储模式。监听 05:20:08 捕获 2e04:0022。05:20:19 快照确认 Spreadtrum Generic Disk1.0、接口 08/06/50、macOS Mass Storage 驱动。沙箱内 libusb 仍为空，沙箱外 05:20:28 返回一台设备；沙箱内 diskutil 的框架错误也通过沙箱外复核解决。

只读 diskutil 元数据：disk4 为 USB LOCAL DISK，32,751,616 bytes、1024-byte block，FAT12，已挂载 /Volumes/NO NAME。目录/文件名/大小筛选无候选固件扩展名。raw 首块读取被 /dev 权限拒绝，未使用 sudo。详细步骤和结果保存 analysis/storage/ 及 storage-report.md。

查阅 spreadtrum_flash issues #32/#34/#43 的作者原始回复，发现 UMS9117 应验证不装电池时的入口；之前两次装回电池的建议可能错过芯片上电时机。#43 的实测设备为 Nokia 110 TA-1395，明确不把其镜像偏移/大小/加载器兼容性当作 TA-1278 事实。

已核对目标身份并成功 unmountDisk disk4。05:23:03 开启新一轮 600 秒沙箱外被动监听（usb-watch-20261025T052303+0800.jsonl）；准备请用户取出电池不装回，按住左软键接线。尚未取得该重试结果。

## 05:27–05:32 — 启动枚举和握手成功，准备有限 BootROM 读取

用户报告不装电池、黑屏。05:27:14 注册表与 05:27:16 沙箱外 libusb 确认 1782:4d00，厂商接口 ff/00/00。此前装电池的失败不再作为设备不支持依据。

编写并用本机 clang/libusb 编译仅 CHECK_BAUD 的版本探针。重新连接后收到 14 字节帧，解码为 BSL_REP_VER 0x0081、6 字节 SPRD3\0，CRC 通过；端点 85/06。源码、构建产物、协议日志保存在项目中。

从 GitHub 官方项目 release 1.20260403 下载 t117_fdl1.bin（第一次沙箱 curl 因 DNS 失败；沙箱外成功），7380 bytes，发布 SHA-256 核对通过。审查同 tag 的加载器初始化/命令处理源码；它是开源 RAM 工具，非原厂固件。没有 NAND 读写实现。

用现有 spd_dump 源码本地编译（USE_LIBUSB=1，LIBUSB_DETACH=0），不安装驱动。固定脚本仅执行 FDL 入 RAM 与两次 32 KiB BootROM 读取。05:31:16 首次运行 ver expected，未上传加载器，未取得数据；05:32:01 启动 --fresh 等用户拔线重接，避免复用已有握手状态。


## 05:32 批次成功 — 首份实机 BootROM

核实会话退出码 0。日志显示 BootROM SPRD3 → CONNECT ACK → FDL 分块上传/END/EXEC ACK → Custom FDL1: CHIP ID = 0x98180001 → CONNECT ACK → 两轮各 32 个 1024-byte 内存读取，均报告 target/read 0x8000。

acquisition.json 保存固定参数、加载器哈希、两个读取文件的长度/哈希。两次样本均 32768 bytes，SHA-256 `1c633df8793137d49b50c3ebb853ba491a29186ae613af3a4ab33fe992b02e7c`。原件保存并设 0444；一个工作副本保留，重复副本删除，双读校验记录保留。未发送 NAND 读取/写入、擦除、重分区、解锁或 efuse 命令。

## 随后本地静态分析与报告更新

运行 file、xxd、strings，并编写 tools/analyze_bootrom.py，以固定样本 SHA-256 验证为前提生成可复现分析。file 的 OpenPGP Public Key 分类与实际启动代码/协议证据矛盾，记录为魔数误判。找到 xcrun 提供的 llvm-objdump，无需安装新软件；生成明确标注的人工 ELF 包装，小范围反汇编验证 ARM 入口和 0xffff1fb6 Thumb 目标。

样本内 SPRD3 在 0x6b80；USB device descriptor 在 0x6e80，1782:4d00 / bcdDevice 0202；配置在 0x6e94，总长 32，接口 ff/00/00、bulk 85/06，与实机一致。索引/签名/熵扫描和描述符边界解析输出到 analysis/bootrom/，不将随机 ASCII 命中当成 UI 资源。

再次搜索 TA-1278 FDL2、固件 PAC、Queen2_BASIC、215_4G.rar。4PDA #1778 检索缓存列明确包含 TA-1278 的 NBX 压缩包，但直接页面 403，尚未取得附件；MediaFire 页面不可访问。没有下载/执行社区闭源 EXE。完整系统读取的缺口仍为兼容原厂 FDL2 与布局/版本证据；当前成功仅限 BootROM。

更新 README、设备报告、采集方案、协议报告、结构报告、发现与原件登记，消除当前状态中“无样本/未运行工具”的过时描述；历史日志保留原状。最后重跑离线分析、核对原件/工作副本 SHA-256，并重建 SHA256SUMS。

## 05:38–05:47：取得兼容 FDL2，实机系统读取成功

1. 复查公开资料与 issue 43；本地 HTTPS 成功打开 MediaFire 页面并下载 Nokia_215_4G_readed_test.pac，39,204,210 bytes，记录 SHA-256。之前 web 工具不可访问不等于该链接失效。
2. 编译审阅过的 unpac.c。头 CRC=0x59a2、数据 CRC=0x2496 均通过；列出 BP_R1.0.0 / UMS9117 目录。只在本地选择提取组件，没有执行 PAC 中的 ERASE 项。
3. 校验当前 custom FDL1 的 ROM 前缀；probe_fdl2 上传 donor FDL2 到 0x80100000 并执行，收到 ACK 0x80；四个系统头均为 DHTB v1。证据目录：analysis/work/fdl2-probe-20261025T054214+0800。
4. tools/acquire_system.py 对 boot0/boot1/kernel/user 执行两次完整 auto 长度读取。四者均通过长度与 SHA-256 比较，原件只读保存。MMI 根据 SRES 最大区段末端读 11,835,196 bytes，两次一致；argv 见 mmi-acquisition.json。
5. 用 fphelper_t117 分析 kernel，得到版本 17.00.17.02、keymap/pinmap、LCD ID 0x9305、初始化解压段及 tfboot。另对 DSP/CM4/APN 的固定 selector 读取 4096-byte 头，均成功；未将 donor 文件长度当成本机长度。
6. LLVM 反汇编 donor FDL2：0x06 → 0x8010b02e → 0x8010c55c；逻辑 FTL 边界先于读取校验。0x29 的 handler 0x8010b0ce 返回 0xb8 / [0,1024,0]。selector 0x90000000 从 offset 0 读取 4096 bytes 返回 0x84；它是 FTL 拼接路径，不是 raw NAND。

## 05:49 起：逻辑全分区、资源与物理页路径准备

- map_logical_parts.c 仅用 READ_FLASH，按 512-byte 对齐的指数/二分探测得到 18 个可读 FTL selector 的边界。另补 boot0 固定 32 KiB，未探测 90000001 的长度，因为反汇编发现该分区越界会合成 FF。
- acquire_logical.py 开始对 19 个范围逐个完整双读。80000001/02/03/04 先通过。用户存储区明显较慢，最初 4 KiB 块不足；显式中断属于本任务的子进程，保留 partial 和失败日志，再用 read_logical.c（16 KiB，READ_FLASH 专用）清空残余 USB IN 后续读。只有后续独立全读相同才认可续读结果。无闪存写入/擦除命令。
- 初次 map_logical_parts 编译 include 路径多了 libusb-1.0，失败后改为 /opt/homebrew/opt/libusb/include 编译通过。沙箱内 ps 被拒绝；沙箱外只查询/停止本任务进程。未使用 sudo。
- tools/analyze_system.py 解析 DHTB、SRES/TRES，输出偏移/哈希资源索引。tools/decode_abm.py 解码 1,833 张 ABM PNG，395 个候选拒绝；图标联系表已目视检查，图形连贯。仅分析系统组件，不检查私人短信内容。
- 已以本机 clang 交叉编译 fdl2_debug_hook.c / entry.S，检查 ELF .text 无未解决重定位，核对 trampoline 重放原始八字节。输出仅为 RAM 调试版本，未当作固件刷入。先查询 RAM 中 NAND 规格与 FTL 元数据，再决定物理页实验；实际运行状态另记录。

## 06:02：19 个范围备份完成

所有已映射的 18 个 FTL 逻辑分区加 boot0 共 19 个范围均完成双读，合计 122,064,896 bytes。两次 SHA-256 一致，原件与工作副本逐项复核通过；boot0/boot1/kernel/user/MMI 有效组件全部与对应完整逻辑区前缀一致。资源索引所有区间与导出字节/哈希复核通过，结果保存在 analysis/verification.json。

06:02:59 启动 acquire_bootrom.py --fresh，等待用户断开 USB 后无电池按左软键重新插线。此时没有继续向旧 FDL2 会话发送命令；下一轮 RAM 调试必须建立新 BootROM 会话。

## 06:06：等待重新进入 BootROM

180 秒等待未观察到 USB 断开，acquire_bootrom.py --fresh 退出 1，提示 `No disconnect observed; no device commands sent`。未加载新 FDL1 或 debug FDL2，物理页实验没有发生；原始 FDL2 是最后已知会话状态。analysis/session-state.json 记录续接点。

Python py_compile 默认尝试写用户缓存，被文件系统沙箱拒绝；改用内存 compile() 校验六个脚本语法，全部通过。未为此申请额外权限或修改系统缓存设置。

## 09:49：用户重新连接后 USB 初始化失败

09:49:02 acquire_bootrom.py 打开目标后，libusb_control_transfer 返回 LIBUSB_ERROR_IO，未进入 FDL 上传；无读取文件。09:49:12 沙箱外 libusb 缓存描述符列表为 0 个设备，确认设备已消失，不能把它当作仍在 BootROM/FDL 会话。09:49:28 启动 --fresh 监听新连接并提示用户保持左软键，避免错过握手时序。原有 122 MB 逻辑备份不受影响。

## 09:50–09:52：RAM hook 与物理页验证

用户重新插线并保持左软键后，SPRD3 握手、FDL1 加载和 BootROM 双读成功，哈希与早前一致。09:50:16 首次 hook 上传在 RAM 校验处停止：主机误用 FDL2 的分块 offset 方式，但 custom FDL1 源码忽略 offset；尚未 EXEC。改为单个 656-byte RAM 读包核对后，在同一 FDL1 会话继续成功，无需再插线。

09:50:36 patched FDL2 执行 ACK；原 boot0 读取回归正常。NAND spec/device 元数据为 1024 blocks、2048-byte page、1024-byte sector、64 pages/block、32-byte spare/sector。FTL NV info 为 1024 sectors × 1024 bytes。调试输入哈希和协议日志位于 debug-fdl2-20261025T095036+0800。

read_nand_pages 对物理页 0/1/64/65535 各读两次，每次 native status=0、spare=64；字节一致且前两页主区吻合已保存 boot0。09:52:30 开始 acquire_nand.py：先补齐 NV 两遍，再执行全部物理页两遍，保存 main/spare 和每 1024 页状态。所有操作均通过 RAM hook 的固定读取路径，未执行 flash program/erase。

## 2026-10-25T10:02:19.639733+08:00：完整物理备份验证完成

NAND 主区 134,217,728 bytes、spare 4,194,304 bytes；两遍整文件 SHA-256 均一致，131,072 次页读取全部返回 0。补充 NV 1,048,576 bytes 两遍一致。原件只读保留，工作副本独立，重复临时副本在比较成功后删除，所有哈希/状态日志保留。tools/analyze_nand.py 复核原件/副本、全页状态，并建立系统头与逻辑页哈希对照，不解码私人内容。

Final offline comparison: 1784 unmatched 2 KiB logical pages have both 1 KiB sectors present exactly in raw. Two remaining pages each differ by one bit from a unique raw candidate; raw and logical repeated reads are independently stable. No raw patching or ECC-codeword decoding performed. Details and reproduction script recorded in analysis/nand-report.md.

## 2026-10-25T10:28:22.732510+08:00：离线 UI 资源与代码分析

按用户要求仅分析系统副本，没有访问 USB 或私人数据，也没有执行固件二进制。本轮开头对通用格式和模块名的公开检索没有提供可用的新证据，后续分析均在本地完成，没有上传固件。使用已有 Python/Pillow、Xcode LLVM、Node，未安装软件。

新增 tools/extract_ui_resources.py：解析 44 组/3 语言，共 22,500 条记录；5,226 条 UTF-16LE 严格解码成功，77 条单字节非 ASCII 暂按 Latin-1 无损展示并保存 raw_hex。首次套用标准 cmap 边界触发断言，检查三份字体及实际 spml_otf_cmap 代码后，记录声明长度短 2 字节的差异，并以主体边界限制。3 份 SFON 原件已提取；glyph 主体仍未解码。

修复 tools/decode_abm.py：ABMR 最后一个压缩字可仅有一字节，按真实 8 bits 消费，不补造字节。ABM PNG 从 1,833 增至 2,228；旧 1833 个 PNG SHA-256 全部未变。新增 20 GIF、2 BMP 提取和预览。

新增 tools/analyze_ui_code.py：验证 user SHA，建立人工 ELF，仅用 llvm-objdump 静态反汇编 14 段。从 0x21cf2c 提取 30 项菜单注册，确认文字/图标 low16 ID 从 1 开始；实际菜单顺序/可见性未知。输入法两份模式注册表在 0xc35a08/0xc36640，391+390 项，指向 147 份 12×80-byte 字符表。查到同键候选循环、750 计时参数、选区/退格路径、SMS 容量和段末 ESC 处理。明确矩阵中没有 CENTER=13，修正先前工具枚举造成的歧义。

新增 tools/build_ui_catalog.py，输出完全本地资源浏览器；使用系统字体显示标签，不冒充手机字体。30 项菜单图标联系表目视检查吻合。tools/validate_ui_outputs.py 通过 5 组件哈希、22,500 文本原字节回查、3 字体切片比较、147 字符表核对、2,228 PNG 哈希、2,250 预览图片加载及 HTML 内嵌数据/JS 语法检查。未做浏览器交互验收。

执行顺序：extract_ui_resources.py → decode_abm.py → analyze_ui_code.py → build_ui_catalog.py → validate_ui_outputs.py。失败和格式差异未掩盖，详情 analysis/ui/report.md、decode-errors.json、validation.json。原始固件保持不变。


## 2026-10-25T11:02:32.468906+08:00：SFON 全字形与交互结构扩展

用户要求全部解码并尽量恢复完整交互。本轮仅用系统工作副本，未访问 USB、私人分区或执行固件。读取项目记录后沿 SFON 查表和绘制代码定位，新增 decode_sfon.py、export_sfon_fonts.py：三份字体 22,349 字形全部精确消费，606 组合字形引用完整，输出 JSON/SVG/OTF；FreeType 样张目视通过。

新增 decode_extra_images.py：509 个 BMP 0x1003、49 个 raw SABM 暂定 BGRA、4 SMOV 的39帧、1 VM 的4帧。初版 RLE 把重复段放在 literal 前，虽然长度通过但 logo 拉花；目视发现后修正为 literal 在前，NOKIA logo 和语言缩写正确。raw SABM 通道及动画时长未宣称确认。SJPG 仍未恢复，未使用猜测 JPEG 表填充。

新增 extract_ui_metadata.py：388 个 CSCS 名称、明确三 locale、73 个媒体文件、2 个空音频占位。通过 llvm-objdump 全文筛出 29 个窗口名称注册调用，其中28个可用简单 literal/movs 模式；50-byte 字符数组提取1357名称。查表代码证明 low16 ID 减1；避免将语音信箱错认作短信编辑。另提取751段 callback 起始的窗口描述，“新建信息”定位到组7的0x7000c，而非仅搜索SMS名称。

新增 build_interaction_map.py 保存11节点控件树与25条原始事件分支；未擅自把事件编号命名为物理键。interaction_reference.py 保存多击输入参考模型并验证6类行为。validate_ui_outputs.py 扩展到新字体、601帧、媒体、窗口原字节及名称；全部通过。字体/图片目视检查已做，浏览器交互和实机状态转换未验收。

调查中出现一次临时Python命令括号语法错误、一次资源相对路径误用，修正后继续；CSCS初版只接受标识符，遇原生铃声路径触发断言，改为保留可打印ASCII路径并严格检查padding。尝试的bitmap常量候选命中无关通信代码，已排除，未作格式证据。误命名的临时反汇编片段已移除，原始备份未变。完整复现入口 tools/rebuild_offline_ui.py；结果 analysis/ui/recovery-report.md。

最终复跑：11步离线流水线全部通过；随后把 SVG viewBox 改成实际曲线边界以避免裁切，重导字体并重跑目录和完整验证。最终仍为 22,349 字形、2,813 预览、601 新帧哈希、751 窗口片段、1,357 名称、73 媒体。未进行浏览器交互验收。


## 2026-10-25T11:37:49.340069+08:00：资源尾项、框架语义与未恢复项交付

继续离线分析，未连接手机或执行固件。新增 decode_sjpg.py：从 SJPG 头恢复高/宽、4:2:2 采样及质量枚举；生成标准 framing 后，将所有 DQT/DHT 与本机 kernel 精确匹配，独立逐 MCU/Huffman 检查熵流与边界，11 张 JPEG/PNG 成功。初次匹配 DHT 包含了类/表 ID 而失败，检查本机布局后改为 counts+symbols；最初要求 padding 全 1 也不符合样本，改为严格记录尾部 0–7 位，验证无未消费字节。联系表目视通过；原生 IDCT 舍入不作一致性承诺。

公开网页工具两次连接失败，改用已连接搜索服务读取 HMD 2020 指南，明确型号覆盖和地区浏览器差异。GitHub connector 获取固定提交 d48d55e028c13b6f50c291cbb7ea29201c3c8432 的 35 个 MMI/T9 文本文件，源码只读不执行。首次本地 HTTPS 树索引请求因 DNS 受限失败，已通过只读网络权限取得索引；URL/文本 SHA 在 mmi-framework/provenance.json。该仓库并非本机原版源码，也未认定全仓开源许可。

新增 recover_framework_semantics.py：以 AST 白名单解析整型宏，不执行源码；751 窗口片段共 2,331 属性注释，25 个消息编辑事件全部关联名称。发现 public END_WIN=f031 与 native=f032，原始相同假设失败后修正，不强行套用晚期枚举。0x0500/0515/0800 分别关联联系人选择、vCard、媒体选择；名称是跨版本关联，处理目标是本机直接证据。

沿原生 0x88135052 转换函数及 0x882e2e9c 调用者定位 105-byte key table，12 对二次映射，19 个矩阵键全部有有效 UI 代码。中心键硬件路径与长按阈值尚未闭合。对77条单字节文本比较 Latin-1/Windows-1252，全部输出一致；3条问号字节已存在原数据，不能唯一恢复。误用 Thumb 解码 ARM 区域的临时 key-translation-thumb.txt 已删除，不用作证据。

新增 extract_data_resources.py：纠正类别6为通用DATA而非仅WAP预置，类别7/8为COLOR/RECT；导出479 DATA（66非零占位）、55 RGB565主题颜色及3 RECT。识别 T9/T9CC/T9AW 7.04.04.01，英文两组三块各拼接36,651字节；简体15块与初始化头、原生加载器和长度全部匹配，重建派生容器。繁体两个变体有5/4处长度不一致，保留原块及头而不伪造完整容器。内部候选算法未恢复。SMOV interval原始值24已记入输出，时间单位未知。

analyze_ui_code.py 新增按键分发、转换器、T9加载/初始化和浏览器启动范围，定向反汇编共24段；过滤浏览器字符串时移除过宽的 opera 匹配，避免 operation/operator 误计。动态浏览器启动读设置0x286并选择两个入口，尚未确认当前启用引擎。临时 SJPG 候选输出目录在最终输出验证后删除；原始文件未更改。

运行 python3 tools/rebuild_offline_ui.py：14步全部成功。validate_ui_outputs.py 新增479 DATA原字节、55颜色、3矩形、3词库容器、3词库头、105键表字节校验。最终2,824图像预览、612增补帧哈希、22,349字形、22,500文本、73媒体等通过。之后目录补充新报告/词库/颜色链接，并对两个user资源记录明确无像素原因，重跑 build_ui_catalog.py 与 validate_ui_outputs.py 成功；未做浏览器交互或实机验收。

交付 reverse-engineering-status.md、interaction-spec.md、official-behavior-baseline.md；同步 README、findings、结构报告和来源。当前无法恢复部分逐项记录已尝试路径、卡点和验证条件，不宣称整个操作系统或全部交互已经还原。检查工具误读不存在的 rebuild_ui_analysis.py 一次，随后使用实际 rebuild_offline_ui.py。

最终完整性流程：python3 tools/refresh_checksums.py。先逐个对照旧 SHA256SUMS 验证 firmware/original/ 清单和内容不变，再生成 original-integrity.json 和新的稳定文件 SHA256SUMS；失败则不替换清单。结果以 original-integrity.json 为准。


## 2026-10-25：Git 仓库整理与原始系统固件发布准备

用户要求 origin 指向 https://github.com/shiftz300/ta-1278 并提交，随后明确要求原始固件也上传。远程 ls-remote 为空；初始化 main，设置 origin。默认代理 127.0.0.1:7890 不可用，临时对 Git/公开参考下载绕过代理后可访问；未修改全局代理设置。

采用 Git 白名单，纳入五个本机系统组件及一份 BootROM，总计38,429,932字节，与原件SHA一致；整片NAND/OOB、NV、完整逻辑备份、工作目录、donor/加载器及第三方参考源码仍仅本地保存。资源浏览器所需系统派生文件同时发布；重复人工ELF、大型轮廓JSON和原始主机日志可重建而不提交。

增加根README、PUBLICATION、第三方许可声明、系统发布清单、工作副本准备脚本和固定参考文本获取脚本。源码获取仅下载并校验，不执行参考文件。保留ABM上游MIT声明。修复设备报告开头粗体和结尾空行。

仅从待提交文件导出临时目录，6原件校验成功；35参考文件实际下载并匹配固定SHA；analyze_system和14步流水线全部成功。验证不依赖本地私人档案，见analysis/ui/publication-validation.json。暂存路径审查无NAND/NV/用户分区，凭据特征检查无命中，单文件均低于100MiB。公开校验清单为PUBLIC_SHA256SUMS；原本完整SHA256SUMS继续仅本地使用。提交/推送结果以Git历史和远程引用为准。

## 2026-10-25 — 短信、浏览器、音乐应用继续离线逆向

目标：为后续功能机交互复刻补齐可实施状态合同。保持本轮本地修改，未提交或推送；无手机 I/O、私人分区读取、真实发信或固件执行。

1. 核对 Git 当前状态和已有输入/控件/窗口/字体资源。固定系统输入 `user.bin` SHA-256 为 `8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca`，所有分析使用原工作副本。
2. 从固定 W19 提交追加 14 个应用参考文本；原有 35 个与新增合计 49 个全部本地 SHA 核验通过。UTF-8 解码失败的文本使用 GB18030 后规范为 UTF-8/LF；记录 source_encoding/URL/hash，不执行源码。参考源码未重新发布。
3. `python3 tools/trace_app_code.py`：最终输出 22 段 Thumb 反汇编和逐范围 SHA/引用候选。LLVM 会把内联字符串/跳表也线性解码，报告明确不把那些伪指令当真实代码。
4. `python3 tools/recover_event_routes.py`：对三个比较树穷举 65,536 个事件 ID，仅求整数/标志/ROM 字面量，遇到调用或未知数据停止。浏览器 32、音乐 39、短信菜单 9 个非默认 ID；修正 narrow MOVS 对 C/V 标志的保留，补独立边界测试。
5. `python3 tools/extract_app_contracts.py`：恢复浏览器五张双 u32 菜单表共 24 项、30 项 TBB；短信正文 137 项相对 u32 分发表（43 个唯一目标）、10 个动态构造菜单节点和 14 项操作提示；恢复音乐重复设置 0=Off、1=All、2=One。
6. 发现并修正旧窗口索引遗漏：按名称筛选漏掉浏览器大量无名描述。按本机窗口 ID 组纳入消息 69、浏览器 40、音乐 28 个片段。窗口数量不是可达页面数。资源按类和组索引；不把文本/图像的同值 ID 混用。
7. 短信确认收件人候选列表优先返回、正文菜单、条件关闭保存、保存草稿转发 e002、消息中心/FDN/空号码/空间不足提示。浏览器确认普通/持续方向键码、timer 参数 500/100（单位未知）、停止请求优先于后退再优先于询问关闭。音乐确认短按释放切歌与长按 seek/释放分离。
8. 新建 `app_interaction_reference.py` 和三份应用合同。没有原生证据的草稿/异步失败/历史/队列规则明确标 P，不算逆向完成。模型仅接合成消息、页面和时长，无网络、SIM、存储或音频调用。已有资源保持原副本，由应用索引链接。
9. `python3 tools/rebuild_app_analysis.py`：5 步全部通过。`test_app_contracts.py` 的 37 项检查通过，包括原生菜单/分支锚点和模型行为。发现原注释脚本重复追加 known_gaps，已修正为幂等更新。
10. 在 `/private/tmp` 建立隔离 Git 可见文件导出；只从已校验发布清单准备系统工作副本，重新生成合成 ELF，删除应用生成结果后运行新流水线。29 个确定性应用输出与当前目录逐字节相同，37 项检查通过。49 个参考文本仅在校验后复制，不声称本次隔离验证又联网下载。没有复制私人 NAND/NV/采集工作副本。记录 `analysis/ui/apps/reproduction-validation.json`。

过程错误：一次文件 glob 无匹配导致读取不存在的 sms-segmentation 文件失败，后确认旧文件名为 sms-length；一次文档批处理 stdin UTF-8 解析失败且 Python 未执行，改用 apply_patch；一次组合补丁因 findings 标题不匹配整体未应用，核对实际标题后重试成功。均不影响原件。Git diff --check 通过。

交付入口 `analysis/ui/apps/README.md`。仍未恢复原生 T9 查询/排序、Opera 应用全状态、页面焦点/排版引擎、完整短信计数/网络异常和像素级 UI；各项明确标记未决或复刻策略，没有编造原机完整恢复。
