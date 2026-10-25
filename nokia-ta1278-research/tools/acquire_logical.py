#!/usr/bin/env python3
"""Two complete reads of each bounded FTL partition using the active FDL2.

Only read_flash is invoked. No loader change or flash write/erase is performed.
Includes private storage in the backup as authorized by the owner; this script
does not interpret or print its contents. This is NOT a physical NAND/OOB dump.
"""
from pathlib import Path
import datetime, hashlib, json, shutil, subprocess, argparse

ROOT=Path(__file__).resolve().parents[1]
MAP=ROOT/'analysis/work/resource-probe-20261025/partition-bounds.jsonl'
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1048576),b''):h.update(b)
    return h.hexdigest()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--resume');args=ap.parse_args()
    stamp=datetime.datetime.now().astimezone().strftime('%Y%m%dT%H%M%S%z')
    if args.resume:stamp=args.resume.removeprefix('logical-')
    work=ROOT/'analysis/work'/('logical-'+stamp);work.mkdir(exist_ok=bool(args.resume))
    orig=ROOT/'firmware/original'/('logical-'+stamp);orig.mkdir(exist_ok=bool(args.resume))
    rows=[json.loads(x) for x in MAP.read_text().splitlines()]
    rows=[x for x in rows if x['readable']]
    rows.insert(0,{'selector':'0x80000001','provisional_bytes':32768})
    report={'time':stamp,'scope':'bounded logical FTL partitions plus boot0 32KiB; not physical NAND',
            'map_sha256':sha(MAP),'components':[],'attempts':[],'status':'running'}
    if args.resume:
        report=json.loads((work/'acquisition.json').read_text())
        assert report['map_sha256']==sha(MAP)
        report['status']='resuming';report.pop('error',None)
    def save():(work/'acquisition.json').write_text(json.dumps(report,indent=2)+'\n')
    print(work,flush=True)
    try:
        for r in rows:
            id=r['selector'];size=r['provisional_bytes'];pairs=[]
            previous=next((v for v in report['components'] if v['selector']==id),None)
            if previous:
                assert sha(ROOT/previous['original'])==previous['sha256']
                continue
            for rep in (1,2):
                target=work/f'{id}-{rep}.partial'
                argv=[str(ROOT/'tools/build/read_logical'),id,str(size),str(target)]
                print(f'{id}: pass {rep}, {size} bytes',flush=True)
                resumed_bytes=target.stat().st_size if target.exists() else 0
                with (work/f'{id}-{rep}.log').open('a') as log:
                    p=subprocess.run(argv,stdout=log,stderr=subprocess.STDOUT,timeout=3600)
                entry={'selector':id,'pass':rep,'argv':argv,'exit_code':p.returncode,
                       'bytes':target.stat().st_size,'sha256':sha(target),'resumed_bytes':resumed_bytes}
                report['attempts'].append(entry);save()
                if p.returncode or entry['bytes']!=size:raise RuntimeError(f'incomplete {id}')
                pairs.append((target,entry))
            if pairs[0][1]['sha256']!=pairs[1][1]['sha256']:raise RuntimeError(f'read mismatch {id}')
            original=orig/f'{id}.bin';shutil.copyfile(pairs[0][0],original);original.chmod(0o400)
            target=work/f'{id}.bin';pairs[0][0].rename(target);target.chmod(0o600);pairs[1][0].unlink()
            report['components'].append({'selector':id,'bytes':size,'sha256':sha(original),
                'two_reads_match':True,'original':str(original.relative_to(ROOT)),
                'work':str(target.relative_to(ROOT))});save()
        report['status']='all_mapped_parts_verified'
    except Exception as e:report['status']='stopped';report['error']=repr(e);raise
    finally:save()
if __name__=='__main__':main()
