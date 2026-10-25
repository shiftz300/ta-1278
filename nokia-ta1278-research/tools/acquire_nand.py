#!/usr/bin/env python3
"""Two complete native-controller NAND reads, preceded by missing NV backup.
Run only with the verified 3918bf66 debug hook already active. No uploads here.
"""
from pathlib import Path
import hashlib,json,datetime,subprocess,shutil,os
R=Path(__file__).resolve().parents[1]
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1048576),b''):h.update(b)
    return h.hexdigest()
def main():
    os.umask(0o077)
    probe=R/'analysis/work/debug-fdl2-20261025T095036+0800/probe.json'
    meta=json.loads(probe.read_text());assert meta['exit_code']==0
    assert meta['build']['hook_sha256']=='3918bf664c45576312a8118957701f9688c6ec3fe22d9403e6031054cf32cd03'
    assert meta['nv_part_info_words']==[1363956806,0,0,98306,256,1024,1024,128]
    samples=json.loads((R/'analysis/work/nand-probe-20261025T0952/verification.json').read_text())
    assert all(v['two_reads_match'] and v['native_status']==0 for v in samples)
    assert all(v['matches_saved_boot0'] for v in samples if v['page']<2)
    stamp=datetime.datetime.now().astimezone().strftime('%Y%m%dT%H%M%S%z')
    out=R/'analysis/work'/('nand-'+stamp);out.mkdir();orig=R/'firmware/original'/('nand-'+stamp);orig.mkdir()
    report={'time':stamp,'scope':'all 65536 physical page addresses; main and native-controller spare, ECC flag=0',
            'geometry':{'blocks':1024,'pages_per_block':64,'main_per_page':2048,'spare_per_page':64},
            'reads':[],'components':[],'status':'running','probe_sha256':sha(probe)}
    def save():(out/'acquisition.json').write_text(json.dumps(report,indent=2)+'\n')
    def call(argv,log):
        with log.open('w') as f:p=subprocess.run(argv,stdout=f,stderr=subprocess.STDOUT,timeout=3600)
        return p.returncode
    def retain(name,a,b):
        assert sha(a)==sha(b),f'{name}: two reads differ; preserve both for diagnosis'
        dest=orig/(name+'.bin');shutil.copyfile(a,dest);dest.chmod(0o400)
        work=out/(name+'.bin');a.rename(work);b.unlink()
        report['components'].append({'name':name,'bytes':dest.stat().st_size,'sha256':sha(dest),
            'two_reads_match':True,'original':str(dest.relative_to(R)),'work':str(work.relative_to(R))});save()
    print(out,flush=True)
    try:
        paths=[]
        for rep in (1,2):
            target=out/f'nv-{rep}.partial';argv=[str(R/'tools/build/read_logical'),'0x90000001','1048576',str(target)]
            rc=call(argv,out/f'nv-{rep}.log');row={'name':'nv','pass':rep,'argv':argv,'exit_code':rc,'bytes':target.stat().st_size,'sha256':sha(target)}
            report['reads'].append(row);save();assert rc==0 and row['bytes']==1048576;paths.append(target)
        retain('nv',*paths);print('NV 1 MiB verified twice',flush=True)
        for rep in (1,2):
            part=out/f'pass-{rep}';part.mkdir();argv=[str(R/'tools/build/read_nand_pages'),'full',str(part)]
            print(f'Physical NAND pass {rep}: 65536 pages',flush=True)
            rc=call(argv,part/'protocol.log');row={'name':'physical','pass':rep,'argv':argv,'exit_code':rc,'files':[]}
            for name,size in [('nand-main',134217728),('nand-spare',4194304)]:
                p=part/(name+'.partial');row['files'].append({'name':name,'bytes':p.stat().st_size,'sha256':sha(p)})
            report['reads'].append(row);save()
            assert rc==0 and [v['bytes'] for v in row['files']]==[134217728,4194304]
            print(f'Pass {rep} complete: '+json.dumps(row['files']),flush=True)
        for name in ('nand-main','nand-spare'):retain(name,out/'pass-1'/(name+'.partial'),out/'pass-2'/(name+'.partial'))
        report['status']='full_physical_address_range_two_reads_match'
    except Exception as e:report['status']='stopped';report['error']=repr(e);raise
    finally:save()
    print('Complete: '+str(out/'acquisition.json'),flush=True)
if __name__=='__main__':main()
