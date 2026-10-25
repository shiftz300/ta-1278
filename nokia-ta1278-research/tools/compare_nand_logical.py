#!/usr/bin/env python3
"""Classify raw/FTL differences without decoding private contents."""
from pathlib import Path
import hashlib,json
R=Path(__file__).resolve().parents[1]
raw=(R/'analysis/work/nand-20261025T095230+0800/nand-main.bin').read_bytes()
digest=lambda b:hashlib.sha256(b).digest()
idx2={digest(raw[p:p+2048]) for p in range(0,len(raw),2048)}
idx1={digest(raw[p:p+1024]) for p in range(0,len(raw),1024)}
j=json.loads((R/'analysis/work/logical-20261025T055044+0800/acquisition.json').read_text());classes=[];result=[]
for c in j['components']:
    b=(R/c['work']).read_bytes();un=[];split=0
    for off in range(0,len(b),2048):
        page=b[off:off+2048]
        if len(page)<2048 or digest(page) in idx2:continue
        halves=all(digest(page[k:k+1024]) in idx1 for k in (0,1024));split+=halves
        if not halves:un.append(off)
    if split or un:classes.append({'selector':c['selector'],'unmatched_2048_pages_with_both_1024_halves_present':split,'remaining_unmatched_offsets':un})
    for off in un:
        page=b[off:off+2048];candidates=set()
        for part in (0,512,1024,1536):
            token=page[part:part+64]
            if token in (b'\0'*64,b'\xff'*64):continue
            start=0
            for k in range(100):
                pos=raw.find(token,start)
                if pos<0:break
                base=pos-part
                if base>=0 and base%2048==0 and base+2048<=len(raw):candidates.add(base)
                start=pos+1
        scored=[]
        for base in candidates:
            dif=[i for i,(x,y) in enumerate(zip(page,raw[base:base+2048])) if x!=y]
            bits=sum(bin(page[i]^raw[base+i]).count('1') for i in dif)
            if bits<=16:scored.append({'physical_page':base//2048,'physical_offset':base,'differing_bytes':len(dif),'differing_bits':bits,'positions_in_page':dif})
        result.append({'selector':c['selector'],'logical_offset':off,'near_matches':sorted(scored,key=lambda x:x['differing_bits'])[:8]})
out=R/'analysis/nand'
(out/'logical-difference-classification.json').write_text(json.dumps(classes,indent=2)+'\n')
(out/'raw-logical-bit-differences.json').write_text(json.dumps(result,indent=2)+'\n')
print('Split-sector page matches:',sum(x['unmatched_2048_pages_with_both_1024_halves_present'] for x in classes))
print('Remaining near-match classifications:',len(result))
