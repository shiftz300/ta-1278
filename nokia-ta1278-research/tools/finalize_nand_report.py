#!/usr/bin/env python3
"""Publish verified local acquisition conclusions, only after full validation."""
from pathlib import Path
import json,datetime,subprocess,hashlib
R=Path(__file__).resolve().parents[1]
W=R/'analysis/work/nand-20261025T095230+0800'
j=json.loads((W/'acquisition.json').read_text());a=json.loads((R/'analysis/nand/structure.json').read_text())
assert j['status']=='full_physical_address_range_two_reads_match' and a['original_work_match']
assert a['all_131072_native_page_reads_status_zero']
now=datetime.datetime.now().astimezone().isoformat()
table='\n'.join(f"| {x['name']} | {x['bytes']:,} | `{x['sha256']}` |" for x in j['components'])
page_table='\n'.join(f"| {x['selector']} | {x['whole_2048_byte_pages']} | {x['pages_present_exactly_in_physical_dump']} | {x['unmatched_pages']} |" for x in a['logical_page_comparison'])
(R/'analysis/nand-report.md').write_text(f'''# 完整物理 NAND 采集与结构

**已经读取全部 65,536 个物理页地址：128 MiB 主区 + 4 MiB OOB/spare；两遍整片 SHA-256 完全一致。** 两遍共 131,072 次原生页读取，均返回 status=0，没有跳过页、合成填充或丢弃报错页。更新：{now}。

| 文件 | 字节数 | SHA-256 |
|---|---:|---|
{table}

原件目录：`firmware/original/nand-20261025T095230+0800/`（0400）；工作副本：`analysis/work/nand-20261025T095230+0800/`。该目录中的 nv.bin 是另行读出的 1 MiB 逻辑 NV，不是物理镜像的第三部分。原件/工作副本的大小与哈希均复核通过。

## 几何与真实物理路径

来自运行中原厂 FDL2 的 NAND spec/device：1,024 blocks × 64 pages/block × 2,048 main bytes/page；每页 spare 64 bytes。选择的规格表项 ID 为 A1 A5，仅标作配置表 ID，尚无独立 READ_ID 抓包或芯片丝印确认，不猜厂牌/封装。

自编译 RAM hook 调用固定原厂 NAND READ 入口 0x8010949c。反汇编显示 SPI PAGE_READ 0x13 的行地址由 `(block << 6) + page_in_block` 构成；此路径没有 FTL 映射或坏块跳过。调用传入原生 ECC 标志 0，保存主区与控制器返回的 spare。原厂函数会配置 feature/controller 寄存器，但没有调用 program、erase、repartition。

hook 在 RAM 0x83000000，SHA-256 `3918bf664c45576312a8118957701f9688c6ec3fe22d9403e6031054cf32cd03`；donor FDL2 仅在 RAM 副本替换读入口。上传字节经过 RAM 回读核对，再执行，并通过原 boot0 读取回归。工具、构建文件与协议日志全部本地保留。

页 0、1、64、65535 事前两次抽样完全一致；前两页主区与之前 boot0 的独立逻辑采集相同。事后验证完整主区、spare、NV 原件/工作副本及所有状态记录。物理范围完整、重复读取一致均有证据；未进行外接编程器交叉验证，未声称已经解码厂商 ECC 或坏块标记。

## 结构索引与逻辑对照

全 FF 主区页：{a['all_ff_main_pages']}；全 FF spare 页：{a['all_ff_spare_pages']}；不同主区页 SHA-256 数：{a['unique_main_page_hashes']}。FF 为真实读取结果，不是失败填充。DHTB/SRES/TRES 头候选共 {len(a['header_candidates'])} 处；偏移、页号、块号及长度字段在 `nand/structure.json`。

下表把每个逻辑范围切成 2,048-byte 页，查找其内容是否精确存在于任一物理主区页。相同 FF/重复页会有多个位置，因此匹配不是唯一的 FTL 地址映射；未匹配可涉及 ECC 修正、特殊编码或对齐，不自动等同采集失败。

| selector | 逻辑整页数 | 物理页精确命中 | 未命中 |
|---|---:|---:|---:|
{page_table}

只比对字节哈希和系统头，不解析私人短信、联系人、浏览记录或 NV 标识。用户已授权完整备份，数据未上传。

## 重组与复现

物理页 p 的主区位于 main[p*2048:(p+1)*2048]，spare 位于 spare[p*64:(p+1)*64]。若其他分析工具要求 2,112-byte 交织页，应逐页连接这两段；不能把整份 spare 简单接在 main 尾部当作交织格式。备份没有经过刷回验证，不是 PAC 刷机包。

1. 无电池、左软键插 USB；电脑已监听时握手最可靠。
2. tools/acquire_bootrom.py → custom FDL1，校验 ROM 与加载器来源。
3. tools/build_debug_fdl2.py 离线构建；tools/run_debug_probe.py 上传并核验 RAM 字节，只查询 NAND 元数据。固定哈希与本型号此次实测布局，不推广至其他 loader。
4. read_nand_pages probe 抽样并比对已知 boot0；tools/acquire_nand.py 执行 NV 双读与物理全页双读。只对满足实测几何的设备继续。
5. tools/analyze_nand.py 离线复核；完整命令与每遍字节数/哈希在 acquisition.json，每 1,024 页完成状态在 pass-*/page-status.jsonl。

无需再寻找其他固件包才能研究本机。系统结构、资源和交互线索分别见 firmware-structure.md、../findings.md；完整物理镜像本身不能直接作为模拟器运行。
''')
subprocess.run(['python3',str(R/'tools/update_summary.py')],check=True)
p=R/'analysis/firmware-structure.md';s=p.read_text().replace('物理 NAND/OOB 尚未取得。','物理 NAND 主区 128 MiB 与 spare 4 MiB 已完成全页双读，哈希一致；详情见 nand-report.md。',1);p.write_text(s)
p=R/'findings.md';s=p.read_text().replace('完整物理 NAND/OOB 尚未取得。','完整物理 NAND 主区与附加区已双读一致取得，另保留 20 个逻辑/启动范围。',1);p.write_text(s)
p=R/'device-report.md';s=p.read_text();s=s.replace('日期：2026-10-25。证据级别：实测 / 用户报告 / 官方资料 / 开源作者记录 / 未知。','日期：2026-10-25。证据级别：实测 / 用户报告 / 官方资料 / 开源作者记录 / 未知。\n\n**最新：完整物理 NAND 已读取两遍且哈希一致，128 MiB 主区 + 4 MiB spare；实测规格为 1024 块 × 64 页，每页 2048+64 bytes。已补齐 1 MiB NV，逻辑/启动范围共 20 个。规格表项 ID A1 A5 不是板级硬件修订，也未据此断言厂牌。见 analysis/nand-report.md。');p.write_text(s)
p=R/'analysis/acquisition-plan.md';p.write_text('''# 固件获取方法：本机已成功完成

完整物理 NAND 的主区 128 MiB、spare 4 MiB 已全页双读一致；同时保留 20 个逻辑/启动范围（123,113,472 bytes）和 32 KiB BootROM。完整性、SHA-256 和精确步骤见 [nand-report.md](nand-report.md)。

成功路径：无电池左软键 → SPRD3 BootROM → 开源 custom FDL1 → 社区 Nokia 215 4G donor FDL2 → 自编译 RAM 读取 hook → 原生 NAND 页读取。没有刷入 donor 系统、擦除、重分区、解锁，也没有安装闭源刷机 EXE 或新驱动。

先前 0x90000000 的拼接逻辑读取返回 0x84，原厂接口不能直接取得所需全片数据；0x90000001 越界会填 FF，也不能通过错误边界求长度。通过反汇编固定 FDL2、读取运行中的 NAND/FTL 元数据后解决：NV 长度为 1 MiB，NAND 物理行地址按块/页直接寻址，原生 ECC 标志为 0。

最初 RAM 回读校验失败来自主机误用 offset：custom FDL1 忽略该字段。修正为单个 656-byte RAM 读包后校验通过，再执行 hook。此失败发生在 EXEC 之前；没有向 NAND 发送 program/erase。详细错误和恢复过程见 research-log.md。

没有取得可验证的 TA-1278 官方公开发行包。使用的社区 PAC 为 donor，包内 NQ2/TA-1272 RU 线索不能证明可刷入本机；其 FDL2 在本机的读取兼容性已实测。研究资料最终来自用户这台设备，内核版本串为 17.00.17.02。

后续无需更多手机操作即可做离线逆向：资源/字体、文本编辑、按键事件、窗口与菜单状态、浏览器组件。整片备份覆盖物理地址范围且重复一致；外接编程器交叉验证、厂商 ECC/坏块解码及刷回恢复测试没有执行。完整镜像不是可直接运行的模拟器，也不是已验证的 PAC 刷机包。
''')
p=R/'analysis/protocol-report.md';p.write_text(p.read_text()+f'\n## {now}：RAM 调试与全物理采集完成\n\n前述“仅构建”是历史阶段。hook 在本机执行成功、原逻辑读回归正常、实际几何符合保护条件，物理页抽样和两遍全页读取均通过。主区与 spare 的完整哈希见 nand-report.md。NV FTL 元数据已确认 1 MiB 并双读一致。所有 131,072 次页读均返回 0，不跳过坏块，不合成填充。\n')
p=R/'firmware/README.md';p.write_text(p.read_text()+f'\n## 完整物理地址范围原件（{now}）\n\n目录 original/nand-20261025T095230+0800/，两遍完整读取 SHA-256 一致，权限 0400。nand-main.bin 和 nand-spare.bin 分别按物理页顺序保存；nv.bin 是独立逻辑分区。\n\n| 文件 | 字节数 | SHA-256 |\n|---|---:|---|\n{table}\n\n没有做恢复刷写测试。页布局、重组规则及验证边界见 analysis/nand-report.md。前文“尚未取得”描述历史阶段。\n')
p=R/'research-log.md';p.write_text(p.read_text()+f'\n## {now}：完整物理备份验证完成\n\nNAND 主区 134,217,728 bytes、spare 4,194,304 bytes；两遍整文件 SHA-256 均一致，131,072 次页读取全部返回 0。补充 NV 1,048,576 bytes 两遍一致。原件只读保留，工作副本独立，重复临时副本在比较成功后删除，所有哈希/状态日志保留。tools/analyze_nand.py 复核原件/副本、全页状态，并建立系统头与逻辑页哈希对照，不解码私人内容。\n')
state={'updated':now,'last_observed_device_state':'RAM debug FDL2 active, no USB acquisition process running',
 'completed':'Full physical NAND main 128 MiB + spare 4 MiB, two complete reads match; 20 logical/boot ranges plus BootROM',
 'physical_nand_complete':True,'debug_loader_built_but_not_loaded':False,'active_usb_process':False,'pending_user_action':None,
 'acquisition':'analysis/work/nand-20261025T095230+0800/acquisition.json','next_steps':['Offline static analysis; no further connection needed for the acquired firmware.','Do not send FDL1 upload commands into the current FDL2 session. User may unplug and reinstall battery to leave RAM debugging.']}
(R/'analysis/session-state.json').write_text(json.dumps(state,ensure_ascii=False,indent=2)+'\n')
verify=json.loads((R/'analysis/verification.json').read_text());verify.update(physical_nand_complete=True,physical_main_bytes=134217728,physical_spare_bytes=4194304,physical_two_reads_match=True,logical_nv_added=True,logical_total_count=20);(R/'analysis/verification.json').write_text(json.dumps(verify,indent=2)+'\n')
print('Reports finalized from verified evidence')
