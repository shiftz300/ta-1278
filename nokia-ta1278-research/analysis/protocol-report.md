# 实机启动协议验证

## 已完成的验证

2026-10-25 05:27（Asia/Shanghai），用户明确报告未装电池、黑屏。注册表和沙箱外 libusb 均看到 VID/PID 1782:4d00。

- bcdUSB 0x0200，bcdDevice 0x0202，接口号 0，类/子类/协议 ff/00/00。
- 缓存配置描述符中的 bulk IN 为 0x85，OUT 为 0x06。
- 系统没有为它建立 USB 串口；libusb 可打开并 claim 接口，无须安装 Windows 驱动。
- 监听曾记录 1782:4d00 → 2e04:0023 → 断开 → 1782:4d00。0023 的用途未验证，不认定它为诊断模式。

本地小型探针 tools/probe_brom_version.c 基于已审阅的 spd_dump 协议编码：只允许目标 VID/PID，读取配置，claim 厂商 bulk 接口，发送上游同款 SET_CONTROL_LINE_STATE 初始化，再发送一个 0x7e（CHECK_BAUD）。不包含 CONNECT、FDL 上传、EXEC、擦除、重分区或闪存读写。

使用已有 clang 和 libusb 编译，没有安装软件。日志 analysis/protocol/20261025T052816+0800-brom-probe.txt：

| 项目 | 实测 |
|---|---|
| USB 初始化 | 成功，返回 0 |
| 发送 | 1 byte CHECK_BAUD |
| 接收 | 14 byte 帧 |
| 解码类型 | 0x0081（BSL_REP_VER） |
| 内容长度 | 6 bytes |
| 版本 | SPRD3 后跟 NUL |
| CRC16 | 验证通过 |

这验证了可响应的启动协议入口；SPRD3 是 BootROM 协议版本字符串，不是手机 OS 版本，也没有单凭它证明芯片具体修订。

## 已使用的加载器

来源：作者 GitHub release 1.20260403 的 t117_fdl1.bin，7380 bytes。

SHA-256：062d5e1a298c0e378eeefcc67dddba6fd4d6be10a683511c849ec59c17fb7414，与 GitHub release asset digest 相同。

文件位于 tools/loaders/，属于研究工具，不是从手机提取的原始固件。相应来源元数据保存在 t117_fdl1.provenance.json。审查了同 release 的 main.c、entry.c、cmd_def.h、链接脚本和 SDRAM 初始化源码：加载地址 0x6200，代码入口跳过 0x200 字节 DHTB 头；初始化时钟/电源/UART/计时器等硬件；支持内存读取。NAND 读写未实现。没有声称发布二进制已被本地可复现构建逐字节验证。

固定采集脚本 tools/acquire_bootrom.py 只运行 fdl 与两次 read_mem 0xffff0000 0x8000，不使用 NAND、擦除、刷写、重分区、efuse 或 t117_exec_dist。仅当两次均读满 32768 字节且 SHA-256 相同，才保存为 firmware/original/ 中的只读原件，并保留工作副本；BootROM 不等于包含 UI/短信/浏览器的完整固件。

首次独立采集 05:31:16 在版本握手阶段返回 ver expected；按上游代码执行顺序，尚未上传 FDL，未产生任何读取文件。可能与已有连接的协议状态有关，未确证原因。

第二次以 --fresh 等待旧连接断开，在新连接中连续执行成功，进程退出码 0。加载器返回 `Custom FDL1: CHIP ID = 0x98180001`。两次 read_mem 均读满 0x8000 字节，SHA-256 均为 `1c633df8793137d49b50c3ebb853ba491a29186ae613af3a4ab33fe992b02e7c`。原件只读保存，工作副本独立，日志及命令参数见 analysis/work/bootrom-20261025T053201+0800/。此前限制已由用户明确撤销，本次 RAM 执行属于用户更新后的调试授权。

## 复现成功路径

主机工具由现有 clang/libusb 编译：

```sh
clang -O2 -Wall -Wextra -DUSE_LIBUSB=1 -DLIBUSB_DETACH=0 \
-I/opt/homebrew/opt/libusb/include \
nokia-ta1278-research/tools/upstream-reference/spreadtrum_flash/spd_dump.c \
-L/opt/homebrew/opt/libusb/lib -lusb-1.0 \
-o nokia-ta1278-research/tools/build/spd_dump
```

在旧目标连接仍存在时启动 `python3 nokia-ta1278-research/tools/acquire_bootrom.py --fresh`，按提示拔线，电池保持取出，按住左软键重新接 USB。脚本等待断开再监听新连接，验证加载器 SHA-256，并连续执行上传与双读。若原本已断开，可以省略 --fresh，先运行脚本再接线。实际成功是在沙箱外普通用户运行，不是 sudo；沙箱内空枚举不能作为手机未连接证据。

固定参数为 `fdl t117_fdl1.bin 0x6200 read_mem 0xffff0000 0x8000 read1.partial read_mem 0xffff0000 0x8000 read2.partial`，此外设置等待 180 秒、verbose 1。没有使用 t117_exec_workaround、keep_charge 或 NAND 指令。脚本只在双读大小/哈希完全符合时保存原件。重复采集不是离线分析的必要步骤。

## 已验证 FDL2 阶段

本次加载的 FDL2 来自已保存 PAC，SHA-256 3c6699d6add21b58f2b87e84e6fdf847e39d9d2ef70f9b19d6ba9d59e4adb7bd。工具 probe_fdl2 先用 custom FDL1 读取已知 ROM 前缀验证会话状态，随后 START/MIDST/END 只发送到 RAM 0x80100000，EXEC 返回 0x80。这之后不再用 FDL1 的内存读写语义。

原厂 READ_FLASH 0x06 载荷为三个 BE u32（selector、size、offset）；成功回复 0x93。系统 selector 80000001–04 已通过完整双读。0x90000004 是 MMI SRES；其他 selector 与边界见 logical-backup.md。最大单次 size 在本二进制 handler 中限于 0x4000，read_logical.c 采用此值。

原厂 0x29 返回 0xb8、12 bytes，u32LE 为 [0,1024,0]。0x90000000 返回 0x84；不发送猜测的 raw 标志。没有执行 PACK 中的擦除项或 loader 的 ERASE/REPARTITION。

### RAM 调试扩展（构建状态，不等于现场成功）

build_debug_fdl2.py 核对 donor 哈希及 0xc55c 原始 8 字节，只将 RAM 副本读入口转向 0x83000000 的 656-byte 自编译扩展。普通 selector 回到原始函数；新增诊断项只提供受限 RAM 读取、FTL info 和固定几何条件下的原生页读取。没有 arbitrary execute、program、erase 接口。

probe_debug_fdl2 必须从 custom FDL1 状态开始；先上传 hook，再上传 patched FDL2，并读回 RAM 核对，随后执行。第一轮仅读取 boot0 头、NAND globals/spec/device、NV 分区元数据，绝不自动触发物理页实验。需要断开 USB 后再次按左软键接线才能可靠回到 BootROM。若元数据不满足几何约束则停止物理读取，不以 guessed 128 MiB 当实测容量。

补充：物理诊断扩展预留三种固定读取模式（原生 ECC 标志开、标志关、主区加 spare），尚未在实机验证。spare 的布局/长度必须从实际 metadata 和对照读取确认，不能仅凭代码把它标注为完整 raw OOB。低层原厂 READ 函数会配置 NAND feature/controller 寄存器；未调用 program 或 erase 例程。

## 2026-10-25T10:02:19.639733+08:00：RAM 调试与全物理采集完成

前述“仅构建”是历史阶段。hook 在本机执行成功、原逻辑读回归正常、实际几何符合保护条件，物理页抽样和两遍全页读取均通过。主区与 spare 的完整哈希见 nand-report.md。NV FTL 元数据已确认 1 MiB 并双读一致。所有 131,072 次页读均返回 0，不跳过坏块，不合成填充。
