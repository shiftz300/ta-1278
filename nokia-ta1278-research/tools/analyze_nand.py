#!/usr/bin/env python3
"""Validate complete physical acquisition and index structure only, offline.
No strings, contacts, messages, browser contents or NV identities are decoded.
"""
from pathlib import Path
import hashlib,json,struct,collections,datetime
R=Path(__file__).resolve().parents[1]
W=R/'analysis/work/nand-20261025T095230+0800'
def digest(b):return hashlib.sha256(b).hexdigest()
def main():
    j=json.loads((W/'acquisition.json').read_text());assert j['status']=='full_physical_address_range_two_reads_match'
    data={}
    for row in j['components']:
        b=(R/row['original']).read_bytes();assert len(b)==row['bytes'] and digest(b)==row['sha256']
        assert b==(R/row['work']).read_bytes();data[row['name']]=b
    main,spare=data['nand-main'],data['nand-spare'];assert len(main)==134217728 and len(spare)==4194304
    for rep in (1,2):
        states=[json.loads(x) for x in (W/f'pass-{rep}/page-status.jsonl').read_text().splitlines()]
        assert len(states)==64 and all(x['all_native_status_zero'] for x in states)
        assert [x['through_page'] for x in states]==list(range(1023,65536,1024))
    index=collections.defaultdict(list);empty_main=empty_spare=0
    for p in range(65536):
        b=main[p*2048:(p+1)*2048];index[digest(b)].append(p)
        empty_main+=b==b'\xff'*2048;empty_spare+=spare[p*64:(p+1)*64]==b'\xff'*64
    originals=[]
    lj=json.loads((R/'analysis/work/logical-20261025T055044+0800/acquisition.json').read_text())
    for c in lj['components']+[next(x for x in j['components'] if x['name']=='nv')]:
        b=(R/c['original']).read_bytes();count=len(b)//2048;match=0
        for p in range(count):match+=digest(b[p*2048:(p+1)*2048]) in index
        originals.append({'selector':c.get('selector','0x90000001'),'bytes':len(b),'whole_2048_byte_pages':count,
            'pages_present_exactly_in_physical_dump':match,'unmatched_pages':count-match,
            'first_page_physical_candidates':index.get(digest(b[:2048]),[])[:64]})
    headers=[]
    for magic in (b'DHTB\x01\0\0\0',b'sres0001',b'tres0001'):
        start=0
        while True:
            pos=main.find(magic,start)
            if pos<0:break
            row={'magic':magic[:4].decode('ascii'),'offset':pos,'offset_hex':hex(pos),'page':pos//2048,'block':pos//131072}
            if magic.startswith(b'DHTB') and pos+512<=len(main):row['header_payload_size']=struct.unpack_from('<I',main,pos+0x30)[0]
            headers.append(row);start=pos+1
    report={'time':datetime.datetime.now().astimezone().isoformat(),'physical_address_range_complete':True,
        'both_full_reads_hash_match':True,'original_work_match':True,'all_131072_native_page_reads_status_zero':True,
        'main_bytes':len(main),'spare_bytes':len(spare),'all_ff_main_pages':empty_main,'all_ff_spare_pages':empty_spare,
        'unique_main_page_hashes':len(index),'logical_page_comparison':originals,'header_candidates':headers,
        'limitations':['FTL/ECC-corrected logical reads can differ from raw physical bytes; unmatched pages are not automatically acquisition failures.',
          'Header matches are candidates, not proof of a contiguous complete component or a partition table.',
          'OOB is the native-controller spare view with the native ECC flag disabled; no external programmer cross-check was performed.',
          'Bad-block markers are retained, but no vendor-specific marker/ECC interpretation is asserted.']}
    out=R/'analysis/nand';out.mkdir(exist_ok=True)
    (out/'structure.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k not in ('logical_page_comparison','header_candidates','limitations')},indent=2))
    print('Logical comparisons:',len(originals),'Header candidates:',len(headers))
if __name__=='__main__':main()
