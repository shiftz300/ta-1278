# BootROM 结构分析（05:32 阶段记录）

## 样本、完整性与方法

样本来自本机 1782:4d00 启动接口，使用已核对的开源 RAM 加载器读取地址 **0xffff0000–0xffff7fff**，共 **32768 bytes**。两次连续读取均完整，SHA-256 均为：

`1c633df8793137d49b50c3ebb853ba491a29186ae613af3a4ab33fe992b02e7c`

只读原件：firmware/original/bootrom-20261025T053201+0800.bin；工作副本：analysis/work/bootrom-20261025T053201+0800/bootrom.bin。日志、精确参数、双读大小/哈希和退出码保存在该工作目录。哈希一致证明两次传输内容一致，不单独证明硬件 ROM 未经映射替换；BootROM 身份由读取地址、启动代码、SPRD3 和与实机吻合的 USB 描述符共同支持。

本地脚本 tools/analyze_bootrom.py 先验证长度与 SHA-256，再生成 analysis/bootrom/ 下的头部、ASCII/UTF-16LE 候选字符串索引、签名扫描、每 KiB 熵统计、USB 描述符及小范围反汇编。没有执行样本代码，没有上传样本。

## 已验证的结构与偏移

所有文件偏移相对于这份 32 KiB 样本；对应地址 = 0xffff0000 + 偏移。

| 文件偏移 | 实际字节/结构 | 结论 |
|---|---|---|
| 0x0000 | 98 f0 9f e5，即小端 ARM 指令 0xe59ff098 | `ldr pc, [pc, #0x98]`，读取 0xffff00a0 的入口指针 |
| 0x0004–0x001c | 每 4 字节为 0xeafffffe | 7 个自循环分支，符合启动异常向量占位结构 |
| 0x0024 | 清零寄存器、设置 CPSR、初始化栈 | 启动入口；0x00a0 的字值为 0xffff0024 |
| 0x006c | 0xfb0007d0 | ARM `blx 0xffff1fb6`，明确切换到 Thumb |
| 0x1fb6 | 4944 / f04f 30ff 等 | Thumb/Thumb-2 解码连贯，返回值参与入口分支；未完成被调用寄存器语义分析 |
| 0x1528 | ASCII `0123456789ABCDEF` | 十六进制字符表；具体引用未追踪 |
| 0x2db8、0x5288 | ASCII DHTB | 常量命中，不等于从这里开始有两个完整固件容器 |
| 0x6b80 | ASCII `SPRD3` + NUL | 与在线 BootROM 版本响应相同 |
| 0x6e80 | 18 字节 USB device descriptor | USB 2.0，VID 1782、PID 4d00、bcdDevice 0202，EP0 64 bytes，无厂商/产品/序列号字符串索引 |
| 0x6e94 | 9 字节 configuration descriptor | wTotalLength=32、1 接口、配置值 1 |
| 0x6e9d | 9 字节 interface descriptor | 接口 0、2 端点、ff/00/00，与实机吻合 |
| 0x6ea6、0x6ead | 各 7 字节 endpoint descriptor | bulk IN 0x85、OUT 0x06，最大包长 512 bytes，与实机吻合 |
| 0x6edb–0x7fff | 全零 | 此范围在读取样本内为填充；不能推导整个芯片 ROM 总大小 |

架构证据来自实际指令及调用目标，而不是仅由产品名字推测。反汇编使用本机 `xcrun llvm-objdump`，为 raw 样本生成了 ELF32 little-endian ARM 的**人工分析包装**，原始样本字节原封不动嵌入包装的 0x100 偏移；包装中的加载地址、ELF 元数据由本脚本提供，不是从手机提取出的 ELF 文件。只展示入口和明确 BLX 目标范围，避免把整份代码/常量混合数据强行当作一种指令集。ARMv7a 是解码器选择，不声称由此测出了 CPU 全部特性。

## 工具误判和分析限制

macOS `file` 把原始样本报为 **OpenPGP Public Key**。这与已知采集地址、可解码的启动向量、SPRD3 和 USB 描述符证据不符，本报告判定为文件魔数启发式误判；未据此尝试密钥导入。原始输出保留在 analysis/bootrom/file.txt。

扫描未命中 ELF、PNG、ZIP、gzip、SquashFS_LE、UBI 的所列魔数；这只排除相应简单签名，不证明没有其他格式/压缩/文件系统。共建立 253 项 ASCII 或有限 ASCII 范围 UTF-16LE 候选字符串，绝大多数是指令字节偶然形成的短串；索引不是 253 个有效资源或符号，不覆盖所有中文字符串编码。

目前没有识别到分区表、文件系统、OS 版本、图标、字体、键盘映射、短信/电话簿/浏览器应用、菜单资源或图形 API。由于样本只是启动 ROM，这些仍需要 NAND 中的系统组件。存储模式 FAT12 是另外的运行时/用户存储视图，不能与 BootROM 拼接成完整固件。

## 系统固件后续解析依据（尚无 PAC 样本验证）

下面保留已审阅的上游解析器信息，**不是本机已提取分区或容器结果**。

## 已审查的解析机制

上游 spreadtrum_flash commit：3d0582579f3521f35a4937b2e871312e4dd6c308；本地文本在 tools/upstream-reference/spreadtrum_flash/。

| 对象 | 源码证据 | 当前可得结论 |
|---|---|---|
| PAC 外层 | unpac/unpac.c:10，sprd_head_t | 该解析器支持含 UTF-16 名称/版本、文件数、目录偏移、magic、两项 CRC 的 PAC 变体 |
| 文件目录 | unpac/unpac.c:25，sprd_file_t | 项含 ID、名称、大小高低位、包内偏移高低位、类型、地址；不是 Android GPT 的证明 |
| PAC magic | unpac/unpac.c:150 | 检查 ~0x50005u，即 0xFFFAFFFA；不是文件开头固定四字节的断言 |
| CRC | unpac/unpac.c:42、170、263 | 反射多项式 0xA001、此实现初值 0，分别处理头和数据；不能简单套用默认初值 0xFFFF 的 MODBUS 工具 |
| DHTB | spd_dump.c:694 | 条件为 LE u32 0x42544844、下一 u32 为 1；长度字段位于组件头 +0x30，再加 0x200；后面还可能处理签名结构 |
| VNTS | spd_dump.c:699 | 另一自动长度路径，按块数和块大小计算；实际组件类型仍需样本 |
| 初始化/压缩段 | fphelper_t117/main.c:278、369 | 使用特征寻找初始化表，支持自定义 lzdec2/lzdec3；匹配不等于语义确认 |
| 硬件资源 | fphelper_t117/main.c:432、477、515、715 | 候选 keymap/pinmap 检测和导出，可辅助按键矩阵/LCD 引脚研究 |

工具 README 的 4G 示例将逻辑选择值 0x80000001、02、03、04 分别写成 boot0、boot1、kernel、user 文件；这是文档示例，不是已验证本机分区边界。不得将它们当作物理内存偏移或按名称放心读取。


## 后续系统组件分析

取得可信 PAC/NBX 或经过确认的系统转储后，先核对来源和长度/校验，再检查容器目录边界、重叠、加载地址和组件类型。保留原件，只在副本上提取；加载器、NV/校准、系统代码和用户区分别辨认。对明确的系统组件建立字符串和资源索引，随后追踪 keymap、绘图/字体、事件处理及应用入口。关键词命中和图片魔数只能作为线索，需结构和引用验证。

当前无需把全部 BootROM 当作 UI 程序反编译，也没有可直接运行原厂 OS 的模拟器。Ghidra、binwalk、7-Zip 未安装；本次使用已有工具已足以完成启动样本的基本验证。
