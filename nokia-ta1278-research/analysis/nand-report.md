# 完整物理 NAND 采集与结构

**已经读取全部 65,536 个物理页地址：128 MiB 主区 + 4 MiB OOB/spare；两遍整片 SHA-256 完全一致。** 两遍共 131,072 次原生页读取，均返回 status=0，没有跳过页、合成填充或丢弃报错页。更新：2026-10-25T10:02:19.639733+08:00。

| 文件 | 字节数 | SHA-256 |
|---|---:|---|
| nv | 1,048,576 | `aa7159f33d51452c6c1fcf779296d45642da293fad49390df33c332b9414a291` |
| nand-main | 134,217,728 | `d763348516e827aa63b6aef18441b96f416c7aa3d4317c801e2e156b086c701f` |
| nand-spare | 4,194,304 | `32b4f19adc373a2f2c74b863967c0dd7a7fda0f580ed655909f08f6b61028a82` |

原件目录：`firmware/original/nand-20261025T095230+0800/`（0400）；工作副本：`analysis/work/nand-20261025T095230+0800/`。该目录中的 nv.bin 是另行读出的 1 MiB 逻辑 NV，不是物理镜像的第三部分。原件/工作副本的大小与哈希均复核通过。

## 几何与真实物理路径

来自运行中原厂 FDL2 的 NAND spec/device：1,024 blocks × 64 pages/block × 2,048 main bytes/page；每页 spare 64 bytes。选择的规格表项 ID 为 A1 A5，仅标作配置表 ID，尚无独立 READ_ID 抓包或芯片丝印确认，不猜厂牌/封装。

自编译 RAM hook 调用固定原厂 NAND READ 入口 0x8010949c。反汇编显示 SPI PAGE_READ 0x13 的行地址由 `(block << 6) + page_in_block` 构成；此路径没有 FTL 映射或坏块跳过。调用传入原生 ECC 标志 0，保存主区与控制器返回的 spare。原厂函数会配置 feature/controller 寄存器，但没有调用 program、erase、repartition。

hook 在 RAM 0x83000000，SHA-256 `3918bf664c45576312a8118957701f9688c6ec3fe22d9403e6031054cf32cd03`；donor FDL2 仅在 RAM 副本替换读入口。上传字节经过 RAM 回读核对，再执行，并通过原 boot0 读取回归。工具、构建文件与协议日志全部本地保留。

页 0、1、64、65535 事前两次抽样完全一致；前两页主区与之前 boot0 的独立逻辑采集相同。事后验证完整主区、spare、NV 原件/工作副本及所有状态记录。物理范围完整、重复读取一致均有证据；未进行外接编程器交叉验证，未声称已经解码厂商 ECC 或坏块标记。

## 结构索引与逻辑对照

全 FF 主区页：21779；全 FF spare 页：8507；不同主区页 SHA-256 数：39167。FF 为真实读取结果，不是失败填充。DHTB/SRES/TRES 头候选共 7 处；偏移、页号、块号及长度字段在 `nand/structure.json`。

下表把每个逻辑范围切成 2,048-byte 页，查找其内容是否精确存在于任一物理主区页。相同 FF/重复页会有多个位置，因此匹配不是唯一的 FTL 地址映射；未匹配可涉及 ECC 修正、特殊编码或对齐，不自动等同采集失败。

| selector | 逻辑整页数 | 物理页精确命中 | 未命中 |
|---|---:|---:|---:|
| 0x80000001 | 16 | 16 | 0 |
| 0x80000002 | 128 | 128 | 0 |
| 0x80000003 | 7168 | 7168 | 0 |
| 0x80000004 | 8192 | 8192 | 0 |
| 0x80000008 | 16002 | 14346 | 1656 |
| 0x90000002 | 64 | 64 | 0 |
| 0x90000003 | 2520 | 2462 | 58 |
| 0x90000004 | 12288 | 12287 | 1 |
| 0x90000005 | 3968 | 3968 | 0 |
| 0x90000006 | 1600 | 1600 | 0 |
| 0x90000007 | 3528 | 3518 | 10 |
| 0x9000000f | 512 | 512 | 0 |
| 0x90000013 | 512 | 512 | 0 |
| 0x90000018 | 512 | 512 | 0 |
| 0x90000019 | 64 | 64 | 0 |
| 0x9000001a | 2016 | 1955 | 61 |
| 0x9000001b | 320 | 320 | 0 |
| 0x9000001c | 128 | 128 | 0 |
| 0x9000001d | 64 | 64 | 0 |
| 0x90000001 | 512 | 512 | 0 |

只比对字节哈希和系统头，不解析私人短信、联系人、浏览记录或 NV 标识。用户已授权完整备份，数据未上传。

## 重组与复现

物理页 p 的主区位于 main[p*2048:(p+1)*2048]，spare 位于 spare[p*64:(p+1)*64]。若其他分析工具要求 2,112-byte 交织页，应逐页连接这两段；不能把整份 spare 简单接在 main 尾部当作交织格式。备份没有经过刷回验证，不是 PAC 刷机包。

1. 无电池、左软键插 USB；电脑已监听时握手最可靠。
2. tools/acquire_bootrom.py → custom FDL1，校验 ROM 与加载器来源。
3. tools/build_debug_fdl2.py 离线构建；tools/run_debug_probe.py 上传并核验 RAM 字节，只查询 NAND 元数据。固定哈希与本型号此次实测布局，不推广至其他 loader。
4. read_nand_pages probe 抽样并比对已知 boot0；tools/acquire_nand.py 执行 NV 双读与物理全页双读。只对满足实测几何的设备继续。
5. tools/analyze_nand.py 离线复核；完整命令与每遍字节数/哈希在 acquisition.json，每 1,024 页完成状态在 pass-*/page-status.jsonl。

无需再寻找其他固件包才能研究本机。系统结构、资源和交互线索分别见 firmware-structure.md、../findings.md；完整物理镜像本身不能直接作为模拟器运行。

## Raw versus logical differences: further classification

All boot0, boot1, kernel and user-image 2048-byte pages match physical main-area pages exactly. Of 1786 other unmatched logical 2048-byte pages, 1784 have both 1024-byte halves present exactly in the physical dump, consistent with the FTL sector view.

The remaining two have unique close matches differing by exactly one bit each: logical 80000008 offset 9807872 versus physical page 59202, and logical MMI 90000004 offset 4382720 versus physical page 31132. Both complete raw passes are identical; both earlier logical passes are identical. This is consistent with ECC correction, but the ECC codewords were not independently decoded. Neither raw original was patched; UI resource analysis uses the separate logical MMI acquisition.

Reproduce with tools/compare_nand_logical.py. Exact locations and counts, without private content, are in nand/logical-difference-classification.json and nand/raw-logical-bit-differences.json.
