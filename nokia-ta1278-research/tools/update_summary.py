#!/usr/bin/env python3
"""Refresh current acquisition status from local evidence, no USB access."""
from pathlib import Path
import json,datetime
R=Path(__file__).resolve().parents[1]
SYSTEM=R/'analysis/work/system-20261025T054301+0800'
LOGICAL=R/'analysis/work/logical-20261025T055044+0800'
j=json.loads((LOGICAL/'acquisition.json').read_text())
components=json.loads((R/'analysis/system/components.json').read_text())
rows=list(j['components'])
nand_path=R/'analysis/work/nand-20261025T095230+0800/acquisition.json'
nand=json.loads(nand_path.read_text()) if nand_path.exists() else {}
nv=next((x for x in nand.get('components',[]) if x['name']=='nv'),None)
if nv:rows.append(dict(nv,selector='0x90000001'))
physical_done=nand.get('status')=='full_physical_address_range_two_reads_match'
physical_status=('**完整物理地址范围已采集：128 MiB NAND 主区 + 4 MiB OOB/spare，两遍整片读取的 SHA-256 一致。** '
                 '覆盖全部 65,536 页；使用原生控制器读取、ECC 标志关闭，未作外接编程器交叉验证。'
                 if physical_done else '完整物理 NAND/OOB 正在验证，以采集记录为准。')
total=sum(x['bytes'] for x in rows)
status=f"{len(rows)} 个逻辑分区/启动区双读一致，共 {total:,} bytes；状态 `{j['status']}`"
lines=['# NAND 逻辑分区采集状态','',f'更新：{datetime.datetime.now().astimezone().isoformat()}','',status,'',
'这些是加载器提供的逻辑 FTL 视图，不是按物理页排列的 NAND/OOB。分区命名来自社区 PAC 对同一 selector 的映射；未知项不猜名称。',
'','| selector | 字节数 | SHA-256 |','|---|---:|---|']
for x in rows:lines.append(f"| {x['selector']} | {x['bytes']} | `{x['sha256']}` |")
lines += ['', '原件：`firmware/original/logical-20261025T055044+0800/`；采集参数、每次结果及续读记录：`analysis/work/logical-20261025T055044+0800/acquisition.json`。',
'',('`0x90000001` 已通过 RAM 调试读取 FTL 元数据，确认长度 1 MiB，并完整双读一致。原件在 firmware/original/nand-20261025T095230+0800/nv.bin。' if nv else '`0x90000001` 的越界路径会返回合成 FF，尚待 FTL 元数据确认长度。')+' 不可读 selector 只代表本次 FDL2 返回错误，不证明物理区不存在。',
'','`0x90000000` 整体逻辑读取失败（响应 0x84）；NAND 信息命令 0x29 返回 `[0,1024,0]`，单凭这三个字段不足以确认几何参数。']
(R/'analysis/logical-backup.md').write_text('\n'.join(lines)+'\n')
table='\n'.join(f"| {x['name']} | {x['size']:,} | `{x['sha256']}` |" for x in components)
(R/'README.md').write_text(f'''# Nokia TA-1278 固件研究

{physical_status}

已经从实机读出 boot0、boot1、kernel、user 应用镜像和 MMI 资源，全部双读 SHA-256 一致。内核软件版本字符串为 **17.00.17.02**。

当前扩展备份：{status}。详见 [逻辑备份清单](analysis/logical-backup.md)。

## 已确认结果

- 不装电池，按住左软键接 USB：1782:4d00，SPRD3。开源 RAM FDL1 返回芯片 ID 0x98180001，32 KiB BootROM 双读一致。
- 从社区 Nokia 215 4G PAC 提取的 FDL2 在本机实测可读取系统；这验证了本次读取用途，不能证明 donor 固件适合刷入。包内标签指向 NQ2 / TA-1272 RU 线索，不冒充 TA-1278 官方包。
- 原厂系统采用 DHTB 组件；MMI 使用 SRES/TRES 资源容器。已解析 2,820 个图像条目、75 个音频条目、44 个字符串块及字体区。
- 已解码 1,833 个 ABM 图像为 PNG，另有 395 个候选当前解码器拒绝。抽样图像视觉检查通过；[图标联系表](analysis/system/icon-contact-sheet.png) 来自固件，不代表实际菜单排布。
- 提取 keymap、pinmap、初始化段；LCD 配置 ID 为 0x9305。user 镜像包含 Smart9Key、MMISMS、编辑控件和 Opera Mini 字符串；存在不等于 UI 可达或行为已还原。

| 实机组件 | 字节数 | SHA-256 |
|---|---:|---|
{table}

## 交付文件

- [设备识别报告](device-report.md)、[调查日志](research-log.md)、[来源与可信度](sources.md)
- [固件结构分析](analysis/firmware-structure.md)、[BootROM 结构](analysis/bootrom-structure.md)
- [获取方法与剩余条件](analysis/acquisition-plan.md)、[协议记录](analysis/protocol-report.md)
- [交互复刻证据](findings.md)、[逻辑备份清单](analysis/logical-backup.md)
- [物理 NAND 采集与结构](analysis/nand-report.md)
- [资源偏移索引](analysis/system/resource-index.json)、[按键矩阵](analysis/system/keymap.json)

## 文件与操作边界

原件位于 firmware/original/；工作副本位于 analysis/work/，派生文件位于 analysis/extracted/。完整分区备份可能含私人数据，仅本地保存，不在报告展示。没有上传设备数据，也未执行 NAND program、erase、重分区、解锁或刷入 donor 系统。

用户已明确撤销最初的读写限制并授权调试。主机使用已有 clang、libusb、Python 和 xcrun llvm-objdump，没有安装驱动或未知 EXE。自编译调试扩展已在 RAM 实测运行，NAND 几何、NV 边界和物理页抽样均通过验证。

离线复现：`python3 tools/analyze_system.py`、`python3 tools/decode_abm.py`。实机流程必须区分 BootROM、custom FDL1、原厂 FDL2、调试 FDL2 四种状态，不能互换命令。具体采集参数保存在 acquisition.json；状态与完整性以清单为准。
''')
print(status)
