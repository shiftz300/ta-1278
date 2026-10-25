#!/usr/bin/env python3
"""Read four proven logical components from the already-running FDL2 twice.

No loader upload, erase, flash write, repartition or reset command is sent.
Requires the exact 512-byte component headers obtained by probe_fdl2.
"""
from pathlib import Path
import datetime
import hashlib
import json
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
HEADERS = ROOT/'analysis/work/fdl2-probe-20261025T054214+0800'

def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()

def main():
    stamp=datetime.datetime.now().astimezone().strftime('%Y%m%dT%H%M%S%z')
    out=ROOT/'analysis/work'/('system-'+stamp)
    out.mkdir()
    report={'time':stamp,'source':'live device via already-running donor FDL2',
            'scope':'logical system components, not whole raw NAND', 'reads':[], 'components':[]}
    def save():
        (out/'acquisition.json').write_text(json.dumps(report,indent=2)+'\n')
    print('Output:',out,flush=True)
    try:
        for i,name in enumerate(['boot0','boot1','kernel','user'],1):
            header=(HEADERS/f'component-{i}-header.bin').read_bytes()
            assert len(header)==512 and header[:8]==b'DHTB\x01\0\0\0'
            base_size=struct.unpack_from('<I',header,0x30)[0]+512
            assert 512<base_size<64*1024*1024
            pairs=[]
            for rep in (1,2):
                target=out/f'{name}-{rep}.partial'
                argv=[str(ROOT/'tools/build/spd_dump'),'--wait','1',
                      'blk_size','0x1000','read_flash',hex(0x80000000+i),'0','auto',str(target)]
                print(f'Reading {name} pass {rep}, payload+header {base_size} bytes',flush=True)
                with (out/f'{name}-{rep}.log').open('w') as log:
                    p=subprocess.run(argv,stdout=log,stderr=subprocess.STDOUT,timeout=600)
                row={'component':name,'pass':rep,'argv':argv,'exit_code':p.returncode,
                     'bytes':target.stat().st_size,'sha256':sha(target)}
                report['reads'].append(row);save()
                assert p.returncode==0 and base_size<=row['bytes']<=base_size+0x1060
                with target.open('rb') as f: assert f.read(512)==header
                pairs.append((target,row))
            assert pairs[0][1]['sha256']==pairs[1][1]['sha256']
            original=ROOT/'firmware/original'/('system-'+stamp)/f'{name}.bin'
            original.parent.mkdir(exist_ok=True)
            shutil.copyfile(pairs[0][0],original);original.chmod(0o444)
            work=out/f'{name}.bin';pairs[0][0].rename(work);pairs[1][0].unlink()
            report['components'].append({'name':name,'selector':hex(0x80000000+i),
                'bytes':original.stat().st_size,'sha256':sha(original),'two_reads_match':True,
                'original':str(original.relative_to(ROOT)),'work':str(work.relative_to(ROOT))})
            save();print(f'Verified {name}: {sha(original)}',flush=True)
        report['status']='four_components_verified'
    except Exception as error:
        report['status']='stopped';report['error']=repr(error);raise
    finally:
        save()

if __name__=='__main__':main()
