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

## 2026-10-25：继续短信、浏览器和音乐的原生逻辑

起点：main/cfb5939（上一轮双语 README 和应用规格已提交推送），工作区原本干净。此次仅本地离线修改，没有再操作手机，也没有读取私人 NAND/NV。系统输入仍为已验证的 user.bin 工作副本。

1. 检查 git status、应用规格、现有脚本和未闭合点。新取回同一固定提交的 10 个框架文本，provenance 从 49 增为 59；本地哈希保留。首次 rg 未带 --no-ignore，忽略了被 Git 排除的参考目录；后续指定 --no-ignore 或显式文件读取。参考代码未运行。
2. 使用现有 `xcrun llvm-objdump -d --triple=thumbv7a-none-eabi --mcpu=cortex-a5` 对合成 ELF 做静态反汇编。应用范围从 22 增为 38；地址、范围 SHA、调用候选写入 code-index.json。线性输出含字面量，不将它们当指令证据。
3. 短信：核对 b50d34 的 8 组容量及 b50ec4 的 ESC 尾部处理。对 3c5e68 的纯整数比较树静态遍历 0…65535，恢复 19 个允许重试原因；人工跟踪 3c407e 的优先级和 3c4030 的等待条件。发现公共 MINI 计时分支与本机此调用路径不同，未直接套用 5000。
4. 浏览器：解出 b62b78 的 22 项 TBB；结合固定头文件命名并核对目标。跟踪地址提交、UTF-16 窄复制与 CR/LF/首字节检查，明确没有本入口 Punycode 路径；继续跟踪认证用户名/密码的不同空值处理、提交和取消回调。
5. 音乐：临时只读 BL 候选扫描找到 ace736 的视图获取；LLVM 核对 ace712→68ace4 的回调安装，再恢复 68ac44 的 fa1b→acec7a→3f125a→3f120c 链。Pause/Resume 使用本机日志确认，不推测数字状态名。4 个子视图事件新增到 event-routes.json。
6. 新增 `native_app_rules.py`、`recover_app_algorithms.py` 和 5 份机器可读证据。高级模型仍保留 P 标记；只复用已恢复的容量参数，未假装整个短信计数/浏览器引擎已经恢复。
7. 运行 `python3 tools/rebuild_app_analysis.py`：6 步通过，49 项检查通过。新增 12 项验证容量分支、ESC、原因码、地址转换、异步表和音乐中心键条件。
8. 新增 `verify_app_reproduction.py`，从 Git 可见文件建立隔离副本，仅用公开系统原件、已核验的 59 份参考文本，重新准备工作副本/合成 ELF，删除应用生成物后重建。50 个确定性输出逐字节相同，49 项检查通过；无新网络下载、无私人数据导入。最终记录见 apps/reproduction-validation.json。

当前限制：UI 排版/计数格式、T9 查询、浏览器内部 DOM/布局/网络状态、全部发送保存竞态和音乐后台行为仍未恢复，详见更新规格和 reverse-engineering-status.md。没有声称恢复完整原厂源码或完成实机验收。本次未提交、未推送。

## 2026-10-25：UI 内核收尾与应用开发准备

继续在已有修改上分析，没有覆盖之前工作。新增 10 段反汇编（总 48）和 interaction-core-rules.json。使用 +hwdiv 仅解码两段计数函数中的 SDIV/UDIV，不据此推断新 CPU。确认短信 %d/%d、页数/剩余数分支和显式 UCS2 除二初始化；纠正 UI 计数与发送 ESC 切段混淆。确认 WML/XHTML 编辑器格式、1024 上限、密码掩码、提交 guard、变化回调和取消；恢复子资源进度与音乐返回的数值条件。60 项测试通过。参考文本 63 份。

过程中发现公共触屏 MMSShowEditWordCount 不可直接当本机入口，另一个 %ld/%ld 是彩信计数，均未误写入短信结论。最终定位共享 TextEdit 的实际本机函数。用户随后授权收尾提交推送、把私人/临时文件移到工作区外 ta-1278-source，再建立模块化 wgpu 交互测试程序。后续执行记录另补。

## 2026-10-25 13:25 +0800：选择框、存储确认、音乐队列与短信字符分类

遵循用户“未完成先继续逆向”：继续本机系统工作副本分析，没有手机 I/O、没有运行目标二进制、没有移动私人数据或开始应用开发。记录后续要求：winit 管理窗口/输入，wgpu 渲染；用户明确选择鼠标按钮映射：左键→左软键、右键→右软键、中键→确认键。详见 apps/replica-plan.md。

1. 核对 WML/XHTML 选择框 b48350/b487c4 和 b482c2/b48534 提交函数。恢复单选确认提交、多选确认停留、多选返回提交、变化回调不对称及失效上下文关闭。BrowserApp 接入该规则，移除通用“选择框取消不保存”假设。
2. 跟踪 bed86 的 10 项输入动作表及 bea84 重置函数。记录 checkbox 先变更后 click 回调、radio 候选处理、提交分发与非完整字段验证、重置默认字段/选项；未假装恢复完整 DOM/JS/HTTP 请求编码。
3. 对照本机 3c3986/3c435e 与已存在的固定版本 mmisms_send.c，恢复发送成功与存储确认先后顺序、状态 1/3/4/5 继续规则、段号 u8 加一、多收件人推进、失败余段只保存和错误 8 分支。没有发送真实短信或读取消息数据库。
4. 用临时只读 Thumb BL 候选扫描跟踪 6861f8/68629c/6862d4/6002fa/61b452，再逐条核对 LLVM 输出。恢复手动与自然结束两套模式、600066 回调、队尾索引判断和 GetNoSupportNumber 终止保护。随机排列生成和媒体 helper 的全部效果仍未解。
5. 由 b0f0e 追到 3a492c/3a4ade，提取 c1797c/c17a7c 两张 128 项表及 c172ec 的 217 个转换项。对 5 个整数比较树各穷举 65,536 个 UTF-16 单元，与手工转写集合独立比较；不执行目标指令或服务。恢复分类优先级、语言槽 0/2/4、FF/ESC 特殊行为和 alphabet_management 开关。sms_counter 改用原生分类；默认配置明确为复刻选择。
6. 应用反汇编增至 65 段。新增/更新纯规则、JSON 范围哈希、短信/浏览器/音乐规格及交互内核文档。先前一次断言多写了日志字符串的 NUL 填充，检查原字节后改为比较终止前的字符串，未更改固件。其他新规则测试通过。
7. `python3 tools/rebuild_app_analysis.py` 七步通过，75 项检查通过。`python3 tools/verify_app_reproduction.py` 从公开系统原件的隔离导出重建，79 个确定性输出逐字节相同；63 个参考文本仅在原哈希核验后复制，没有新下载、私人数据或设备访问。合成测试不是实机验收。

仍有 T9 查询/排序、像素排版、全部窗口焦点/输入法切换、发送取消/数据库异常、完整网页布局/协议和播放器后台副作用等未解项。此次提交为继续逆向的可复现进度版本，不声明完整交互已恢复；不据此跳过用户要求的逆向优先顺序。

## 2026-10-25：必要调查收尾、归档与桌面程序

按用户“按必要性快速逆向，做完前别停”的要求，将 T9 查询、精确布局/时序、完整 Dorado 和后台音频等明确留作未恢复项，不继续无差别展开底层代码。新增 replica-readiness.md，提交 cac59db 并推送后开始实现。

- 使用 `git ls-files --others --ignored --exclude-standard -z` 枚举，353 个被忽略的私人/临时文件移动至同级 ta-1278-source/workspace，739,586,385 字节。移动前后 SHA-256 全部一致，外部 archive-manifest.json 保留逐文件记录；未移动已跟踪的系统原件。
- 新建 phone-core/phone-desktop 两个 Rust crate，winit 窗口与统一输入，wgpu 240×320 显示，复用公开字体和菜单图标。短信本地草稿/待发箱、简单 HTTP(S) 文本浏览器、音乐模拟状态；电话、联系人服务空置。
- Cargo 初次下载受到失效 127.0.0.1:7890 代理影响；仅该命令设置 CARGO_HTTP_PROXY 为空后成功下载 crates.io 声明依赖，没有改全局配置。
- 原生 Metal 窗口启动并呈现页面。实际鼠标三键、短信输入和浏览器加载测试发现短信弹窗绘制 unwrap 崩溃；修复后增加全屏幕/弹窗绘制测试，重新完成保存、重开和待发箱 NOT SENT 路径。
- 测试、Clippy、构建和窗口验证记录见仓库 docs/validation.md。没有访问实机、执行固件、发送短信或拨号。

## 2026-10-25：运行中实机诊断与第一轮交互采样

用户接受测试模式会填充联系人并修改使用设置后，选择原机 `#*8378#8#` 的 Open 并重新连接。USB `2e04:0019` 的接口 1（IN 83 / OUT 04）经原机 UCOM open 控制请求后返回 `17.00.17.02`；未发送 open 时版本请求超时。恢复了截图 type 0x33 / response 0xa0、按键 type 1 及 framing，未加载其他代码、刷写固件或执行从固件提取的二进制。

- `python3 scripts/recover-runtime-entry.py`：验证测试入口、USB Log NV、设置副作用和联系人填充路径；新增校验生成号码 `123%04d`、姓名 `a%04d` 的原件偏移。
- `python3 scripts/handset_capture.py step/run/capture --profile docs/handset-profiles/ta1278-17.00.17.02.json ...`：逐步截图守卫，错误立即停止，不自动重发按键。命令、生成的 plan、ACK 和哈希均在各会话中。
- JPEG 的开头有时被覆盖。仅修复已验证元数据范围；量化表与本机 kernel 表一致，保留每份原始响应和哈希，不猜补像素。四次完整图像拒绝与其他守卫停止均保留。
- 编号 001–200：200 次版本验证，2,055 张解码截图，927 个按键 ACK，183 个完成计划、6 个纯截图会话、11 个停止尝试。这不是功能完成比例。
- 数字短信边界回归：172 步、7 个截图断言通过，清空测试正文并恢复拼音；另有一次菜单重入焦点断言通过。ASCII 160/161 为 0/1、145/2；Unicode 70/71 为 0/1、63/2。
- 主菜单 18 个入口已核实；支付宝、喜马拉雅仅经过焦点，未打开。16 个其余入口及 Other 的四项工具均有不同程度采样，不能把入口覆盖称为完整应用采样。
- 浏览器只提交一次公开测试网址 http://example.com；只得到无标题空白内容区。记录地址编辑、页面菜单和两种关闭确认，未把桌面浏览器行为当作原机网页渲染证据。
- 通讯录被测试模式填满。核对 a0000 的号码确为 1230000 后，仅删除这一条本次生成的测试联系人，腾出一个槽位，成功打开并取消空白新联系人表单。其余原有记录未改动。
- 有一条无收件人模板短信执行了另存草稿，退出编辑；持久化与重开尚未证明。公开网址可能进入历史；贪吃蛇测试局已暂停；音乐停在测试曲目暂停态。收音机、手电筒已关闭，秒表清零，最终回主屏。测试模式仍启用。
- `PYTHONPATH=scripts python3 -m handset.archive_sessions --through 200 --out /Users/shiftz/dev/ta-1278-source/handset-sessions/20261025`：归档 5,677 个文件并核验复制；其中 5,647 个属于编号会话。私人/原始画面不进入 Git。
- `PYTHONPATH=scripts python3 -m handset.export_samples`：明确白名单导出 112 张已审阅实机 PNG，附原始响应、PNG 和像素 SHA-256。`PYTHONPATH=scripts python3 -m handset.compare_theme` 对六个色块与固件表作可复现对照。
- `PYTHONPATH=scripts python3 -m unittest handset.test_handset`：14 项通过。完整采样、桌面逐页对照、网页内容/表单、实际服务和多项边界仍未完成；详见仓库 docs/handset-observations.md 与覆盖清单。


## 2026-10-25 — SIM insertion and live browser follow-up (201–270)

- 用户装卡重启后重新核验 USB/固件；204 起屏幕显示 SIM 2 中国电信 4G/VoLTE，诊断接口无需重新 Open 即可使用。没有修改 APN。
- 依次用公开 example.com、httpbin 表单、CERN 静态链接、Selenium web-form 做 GET 访问。成功采集文本、表单控件、换行、滚动、链接、前进/后退、HTTPS 跳转和单选下拉列表。只输入合成 test、test 加换行加 a；不提交表单或打开文件上传。
- `python3 scripts/handset_capture.py step/run/capture --profile docs/handset-profiles/ta1278-17.00.17.02.json ...`：每次完整参数和生成计划保存在对应外部会话的 plan.json/events.jsonl。201–270 共 890 张解码截图、410 个按键 ACK、64 次完整计划、6 次纯截图；5 个图片断言通过。219 第一次 Down 未移动，220 再次 Down 才移动，原因未定；没有以 ACK 替代画面证据。
- 261 执行 `docs/handset-plans/browser-select-cancel.json`：3 步、2 断言；263 执行 browser-link-back.json：2 步、2 断言。结果文件记录计划及事件日志 SHA-256。公共页面变化或网络延迟会使回放停止，不作任意初态全自动保证。
- 主机首次请求 CERN/Selenium 因沙箱 DNS 不可用失败；使用获准网络访问重新读取成功。主机页面源保存到外部 public-test-pages/，哈希和 URL 在 docs/evidence/browser-test-pages.json；不把主机响应等同于手机网络抓包。
- `PYTHONPATH=scripts python3 -m handset.archive_sessions --through 270 --out /Users/shiftz/dev/ta-1278-source/handset-sessions/20261025`：原始会话留在外部，逐文件校验。export_samples 白名单增至 136 张，新增 24 张均为审阅过的公共测试页面。没有复制私人主屏或上传设备数据。
- 本轮纠正两处概括：有前页时右软键是 Back，无前页才提示关闭；下拉列表重开焦点与选中值分离。主屏左侧显示“功能表”，但注入键 8 打开浏览器，物理按键等价性仍待核对，不能据此直接改复刻键位。270 已退出浏览器返回主屏，测试模式保留。公共 URL 可能留在历史中，未清空历史。


### User correction and settings verification (271–283)

用户纠正：带大时钟页的左右快捷键可在设置配置，不是普通功能表键映射。只读进入 设置→个性化选项→键盘→锁定屏幕快捷操作，278 明确显示右键=信息、左键=互联网，解释了前述疑问。遍历左键 18 个候选，记录无、视频、多媒体资料等选项；没有启动候选应用或保存设置。282 取消后仍为原绑定，283 返回主屏。13 会话、97 截图、42 按键、12 完成计划、1 次纯截图。新增 6 张已审阅设置截图，公开总数 142。已修正文档，不再将当前绑定当作默认或诊断映射错误。


## 2026-10-25 — Apply live observations to the desktop implementation

No additional phone command in this pass. Firmware extraction/reverse-engineering completeness is unchanged. Corrected the observed 18-entry menu/order/return behavior, configurable Home side shortcuts, horizontal theme preview/cancel, original icon scale and preview resources, SMS grids/templates/modes and browser navigation/form behavior. Added local organizer and utility routes as explicitly bounded replica implementations. Rime stroke data is pinned and hash-recorded in `assets/sources.json`; no downloaded executable was run.

Local validation: 51 Rust tests; strict Clippy; resource hashes; all ten multi-tap rows. Final real winit/wgpu run: 33 synthetic scenarios, 34 Metal-presented frames; captures and SHA-256 manifest in `docs/images/`. An occluded first run exposed a busy redraw/test-wait issue; corrected throttling, occlusion handling, per-frame progress and a 30-second timeout, then reran successfully. See `docs/fidelity.md` for unresolved native paths. Native sampling remains 283 sessions / 142 public reviewed images; implementation screenshots are separate.

### 2026-10-25 — browser and SMS layout correction

Continued actual runtime USB screenshots and guarded keys, sessions 284–312. Expanded public native samples from 142 to 160. Browser settings revealed one home type, two font sizes, inline image toggle, history/cache clear popup and a separate restore section. Image preference was temporarily toggled then restored OFF. No clear/reset was activated. Exit LSOFT did nothing; Center exited. Raw samples were archived outside Git with SHA-256 verification.

Implementation now preserves inline HTML flow and shares painting/focus bounds; native chrome, settings/pickers, editor underline, softkey geometry and SMS inactive-body layout were corrected. Original T9, full browser engine and full UI parity are not claimed. See `../../docs/browser-layout.md` and `../../docs/evidence/browser-layout-20261025.json` for metrics, commands, tests and unresolved behavior.
