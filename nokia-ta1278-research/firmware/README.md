# 原始样本登记

**仓库发布更新（2026-10-25）：** 用户授权上传原始系统固件。已发布 system-20261025T054301+0800 的五个组件及 bootrom-20261025T053201+0800.bin，逐文件校验见 [公开清单](public-system-manifest.json)。完整逻辑/NAND/NV 备份与 donor 候选仍不提交。以下登记同时覆盖本地档案，不能据目录名认定全部已公开。

## 本机 BootROM

- 文件：`original/bootrom-20261025T053201+0800.bin`
- 来源：用户所持 TA-1278 的实机启动接口，非网络发行包。
- 采集批次：2026-10-25 05:32:01 +08:00 启动；实际成功时序见采集日志。
- 地址与长度：0xffff0000，0x8000 = 32768 bytes。
- SHA-256：`1c633df8793137d49b50c3ebb853ba491a29186ae613af3a4ab33fe992b02e7c`。
- 连续两次读取大小及 SHA-256 一致，进程退出码 0；只保留一份只读原件、一份分析工作副本，两次读取的校验记录均保留。
- 证据：analysis/work/bootrom-20261025T053201+0800/acquisition.json 和 acquisition.log。
- 芯片 ID：0x98180001；协议版本 SPRD3；此阶段 OS 未知；后续内核取得版本串 17.00.17.02，地区/硬件修订仍未知。
- 范围：启动 ROM，不是包含菜单、短信、浏览器和 UI 资源的完整 NAND 固件。

加载器 tools/loaders/t117_fdl1.bin 属于下载的开源研究工具，不是本机原件。没有把论坛镜像当作本机固件。

## 本机系统与完整逻辑范围

- system-20261025T054301+0800：boot0、boot1、kernel、user、mmi 的双读一致原件。
- logical-20261025T055044+0800：扩大至完整逻辑分区范围的备份；已经完成哪些范围以 analysis/logical-backup.md 与 acquisition.json 为准。
- 完整分区备份可能含私人数据，原件权限 0400，仅保存在本地；不展示其内容。原件与工作副本分开。
- 不能将两个目录按顺序拼接后称为物理 NAND 镜像。未取得的 NV 边界、保留区和 OOB 不用合成填充替代。

## 下载候选

candidates/Nokia_215_4G_readed_test.pac 是社区 donor，不是本机原件，也没有声称为 TA-1278 官方同版本固件。来源、CRC、SHA-256 与使用边界见 sources.md、analysis/research-continued/。

## 完整物理地址范围原件（2026-10-25T10:02:19.639733+08:00）

目录 original/nand-20261025T095230+0800/，两遍完整读取 SHA-256 一致，权限 0400。nand-main.bin 和 nand-spare.bin 分别按物理页顺序保存；nv.bin 是独立逻辑分区。

| 文件 | 字节数 | SHA-256 |
|---|---:|---|
| nv | 1,048,576 | `aa7159f33d51452c6c1fcf779296d45642da293fad49390df33c332b9414a291` |
| nand-main | 134,217,728 | `d763348516e827aa63b6aef18441b96f416c7aa3d4317c801e2e156b086c701f` |
| nand-spare | 4,194,304 | `32b4f19adc373a2f2c74b863967c0dd7a7fda0f580ed655909f08f6b61028a82` |

没有做恢复刷写测试。页布局、重组规则及验证边界见 analysis/nand-report.md。前文“尚未取得”描述历史阶段。
