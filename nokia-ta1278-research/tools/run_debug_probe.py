#!/usr/bin/env python3
"""Run only after fresh BootROM->custom FDL1 succeeds. No physical-page read.
Verifies all RAM upload inputs, saves protocol log and NAND metadata.
"""
from pathlib import Path
import hashlib,json,datetime,subprocess,struct
R=Path(__file__).resolve().parents[1];B=R/'tools/build'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    meta=json.loads((B/'fdl2-debug-build.json').read_text())
    hook=B/'fdl2-debug-hook.bin';loader=B/'fdl2-debug-ram.bin'
    assert sha(hook)==meta['hook_sha256'] and sha(loader)==meta['patched_sha256']
    original=R/'firmware/original/bootrom-20261025T053201+0800.bin'
    assert sha(original)=='1c633df8793137d49b50c3ebb853ba491a29186ae613af3a4ab33fe992b02e7c'
    out=R/'analysis/work'/('debug-fdl2-'+datetime.datetime.now().astimezone().strftime('%Y%m%dT%H%M%S%z'));out.mkdir()
    argv=[str(B/'probe_debug_fdl2'),str(hook),str(loader),str(original),str(out)]
    report={'argv':argv,'build':meta,'physical_page_read':False}
    with (out/'protocol.log').open('w') as log:
        p=subprocess.run(argv,stdout=log,stderr=subprocess.STDOUT,timeout=90)
    report['exit_code']=p.returncode
    if not p.returncode:
        spec=(out/'nand-spec.bin').read_bytes();dev=(out/'nand-device.bin').read_bytes()
        u32=lambda b,o:struct.unpack_from('<I',b,o)[0]
        u16=lambda b,o:struct.unpack_from('<H',b,o)[0]
        report['observed_fields']={'spec_u32_08':u32(spec,8),'spec_u16_14':u16(spec,0x14),
           'spec_u16_18':u16(spec,0x18),'device_u32_08':u32(dev,8),'device_u16_0c':u16(dev,12),
           'device_u16_0e':u16(dev,14),'device_u16_26':u16(dev,0x26)}
        report['nv_part_info_words']=list(struct.unpack('<8I',(out/'nv-part-info.bin').read_bytes()))
        report['status']='RAM_metadata_obtained; physical page interpretation pending'
    else:report['status']='probe_failed; inspect log, do not assume loader state'
    (out/'probe.json').write_text(json.dumps(report,indent=2)+'\n')
    print(out);print(json.dumps(report,indent=2));return p.returncode
if __name__=='__main__':raise SystemExit(main())
