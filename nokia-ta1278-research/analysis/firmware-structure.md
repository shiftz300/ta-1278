# 实机固件结构分析

证据日期：2026-10-25。输入均为本机双读一致的 system-20261025T054301+0800 组件工作副本。完整分区备份进度另见 logical-backup.md。物理 NAND 主区 128 MiB 与 spare 4 MiB 已完成全页双读，哈希一致；详情见 nand-report.md。

## 容器与代码

| 组件 | 文件大小 | DHTB 载荷偏移 | 载荷长度 | 尾部长度 |
|---|---:|---:|---:|---:|
| boot0 | 31,284 | 512 | 30,080 | 692 |
| boot1 | 63,972 | 512 | 62,768 | 692 |
| kernel | 13,154,212 | 512 | 13,153,008 | 692 |
| user | 13,312,500 | 512 | 13,311,328 | 660 |
| mmi | 11,835,196 | 不适用 | SRES/TRES | 不适用 |

DHTB 魔数 `44 48 54 42 01 00 00 00`，长度字段在头 +0x30。auto 长度读取按上游 spd_dump 的签名结构处理尾部；两次读回的 SHA-256 一致不等于已验证原厂数字签名。user 指应用代码镜像，不等于短信/联系人文件系统。

代码是 ARM/Thumb 平台裸二进制，未发现依据把它当作 Android ELF/APK 分区。fphelper_t117 的特征匹配和 lzdec2 解压成功输出初始化段；尚未完整反汇编系统或还原任务调度。人工构造的 ELF 仅用于 llvm-objdump 地址对齐，不是手机原有可执行文件。

## 内核中可复核的偏移

以下偏移相对含 DHTB 头的 kernel.bin，不是物理 NAND 地址。

| 偏移/位置 | 实际证据 | 意义与限制 |
|---|---|---|
| 0xbd2e7c、0xc25ac0 | Software Version:17.00.17.02 | 本机固件内的软件版本串；未与设置屏幕交叉验证 |
| 0xa6399c / 0xc2582c–0xc258cc | 初始化表线索 | 上游 helper 成功识别并解压；部分内存布局已取得 |
| 0xa639a4 | lzdec2 例程特征 | 解压算法路径已由 helper 使用 |
| 0xb401cc | 128 字节 keymap | 64 个 u16 项；已按 8×8 矩阵导出 |
| 0xb4024c–0xb40b54 | pinmap | 导出 2,356 字节；其后有 GPIO 表 |
| 0xb3f2e0 | NAND 配置表候选 | 包含多种支持芯片，不能据此认定实装 NAND 品牌 |
| 0xc258d0 | 内嵌 tfboot | helper 提取 201,272 bytes |
| 解压段 0x81c29a24 | LCD 配置，ID 0x009305 | 本机固件配置与社区 GC9305 线索一致；非直接读取面板 ID |
| 推导加载基址 0x80e65e00 | ps_addr，ps_size=0xc56b08 | helper 的代码/表推导，非现场 CPU 寄存器快照 |

导出：analysis/extracted/kernel/ 下 init_00000000.bin（6,120 bytes）、init_81c20000.bin（474,068）、tfboot.bin、keymap.bin、pinmap.bin。原始扫描日志在 analysis/system/kernel-scan.txt 和 kernel-unpack.txt。

## SRES / TRES 资源

MMI 开头为 `sres0001`；33 组 offset/size 项从 0x10 开始，后续具名段目录从 0x11c 开始。已用边界检查解析所有实际条目。

| 区段 | 文件偏移 | 大小 |
|---|---:|---:|
| TRES 0 | 320 | 11,814,732 |
| logo | 11,815,052 | 4,068 |
| lang | 11,819,132 | 76 |
| cscs | 11,819,208 | 15,988 |

TRES 有 `tres0001` 头、分组指针、每组 8 类资源表；已从本机数据解析 2,820 图像、5 动画、75 音频、44 字符串块、1 字体块、1 WAP 配置块及两个未知类型块。计数是表记录数，可能含重复资源；存在条目不证明当前界面一定使用。

图像表给出宽、高、类型、数据长度与偏移；发现 ABM、SABM 等格式。本地解码器现成功输出 2,228 个 ABM PNG，此前 395 个末尾单字节失败项已修复；另提取 20 GIF、2 标准 BMP。3 份 SFON 和 22,500 条语言记录已导出，详细格式、代码证据和未解码项见 [UI 离线报告](ui/report.md)。联系表来自固件素材，不代表实机菜单位置。

- 全资源索引：system/resource-index.json（原 MMI 文件偏移、大小、类型、SHA-256）。
- 分组表：system/resource-sections.json。
- 图像解码结果：system/abm-decode.json；PNG 在 analysis/extracted/png/。
- 可视样本：system/icon-contact-sheet.png。

## 应用线索

ASCII 关键词索引只在系统代码和资源组件中建立，不扫描私人 FTL 分区。user-topic-strings.json 共收集 11,282 条候选，其中包括误命中，需要按结构和代码引用继续筛选。

| user.bin 偏移 | 字符串/结构 | 可支持的结论 |
|---|---|---|
| 0x2f544 | MMIIM_SP_Smart9Key_GetKeyTable | 存在 Smart9Key 输入法键表代码线索；不能直接等同于 T9 品牌 |
| 0x60098 | Native Opera Mini/4.4.34868 | 镜像含 Opera Mini 版本标识；尚未确认本机菜单入口及实际默认引擎 |
| 0x6a534、0x6a550 | Browser Version(DORADO)、Browser Version(Opera Mini) | 两种浏览器标识路径都存在，不能仅选一个作为实际运行结论 |
| 0x7fa88 | mmisms_api.c | 短信模块文件名线索；结合大量 MMISMS 日志定位实现 |
| 0x94648、0x953ec | baseedit_ctrl_ptr->cursor.cur_pos | 编辑控件具有光标位置与删除边界判断；具体字符计数尚未还原 |
| 0xc9614 | ctrlmainmenu.c | 主菜单控件线索 |
| 0x11b5c0 | GUI_UTF82UCS2B | GUI 文本转换路径线索 |

没有执行任何提取出的固件二进制。没有把该固件称作可直接启动的模拟器。BootROM 的独立完整分析见 bootrom-structure.md。


## 后续 UI 结构恢复

SFON 分段 glyph 表及全部 22,349 个轮廓记录、BMP 0x1003、SMOV/VM 帧目录、语言和 CSCS 名称表均已进一步解析。详见 [资源与交互恢复报告](ui/recovery-report.md)，以及字体、图像、窗口描述的可复现解析器。


## 2026-10-25：UI、词库与资源分类最终增补

当前完整交付与未恢复项见 [离线逆向状态](ui/reverse-engineering-status.md)，具体行为证据见 [交互实现依据](ui/interaction-spec.md)。此前分析状态按时间保留，以下更新覆盖早期未解码结论。

- SFON 三份全部 22,349 字形已解码，导出 OTF/SVG；不代表原生像素栅格化已还原。
- 11 个 SJPG 已通过本机 kernel 量化/Huffman 表匹配和独立熵流检查重建。图像预览总计 2,824 / 2,826；两个余项为无像素的 user 名称记录。
- SRES 类别 6 是 DATA，旧 `wap_presets` 标签过窄；479 个记录中含网络预置与 T9 块。类别 7/8 为 COLOR/RECT，分别恢复 55/3 项。旧切片名称仅为流程兼容保留。
- T9 7.04.04.01 的英文两组三块资源已拼接；简体 15 块由原生加载代码和全部头部长度验证，重建派生容器。繁体两套候选存在 5/4 个长度差异，尚不构成完整容器。候选算法未解。
- 751 段窗口描述注释 2,331 个属性；新建信息的 25 分支匹配公开框架名称。105-byte 原生键表与调用路径已定位，19 个矩阵键到 UI 的映射可复核。跨版本符号不是恢复的原版源码。
- 14 步离线流水线通过。输出验证含原字节回查和源文件哈希；完整流程、JSON 证据与原始 SHA 清单均本地保存。
