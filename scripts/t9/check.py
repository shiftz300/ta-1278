#!/usr/bin/env python3
"""Independent Python-interpreter fixtures for the Rust runtime. Synthetic text only."""
import copy,json,hashlib
from pathlib import Path
import probe as e
import probe_chinese as z
F=e.field;T=e.text;U=0x20016000

def call(a,pc,*args):
    status=a.call(pc,*args)
    assert status==0,(hex(pc),status)

def base(lang):
    a=copy.deepcopy(e.a if lang=='English' else z.a)
    if lang=='English':
        call(a,0x880e4e44,F,T,e.aux,2048,0,0x10000010,0x10000020)
        call(a,0x880e54a8,F,0x109,F,0)
        for f in [0x880e56c8,0x880e55b8]:call(a,f,F)
        call(a,0x880e2220,F,U,4096,2,0x109)
    else:
        call(a,0x880d7718,F,T,z.aux,2048,0,0x10000010,4,0x10000020)
        call(a,0x880d7ba4,F,0x204,F,0)
        call(a,0x880d73d8 if lang=='Stroke' else 0x880d6e94,F)
        call(a,0x880d6ff8,F,4)
        call(a,0x880d7884,F,U,110,2)
    return a
rows=[]
for lang,codes in [('English',['4663','43556','96753','66542','8378','2','999']),('Chinese',['64','426','94664','744','943','932','7','2']),('Stroke',['1','12','32','4','5'])]:
    initial=base(lang)
    for code in codes:
        a=copy.deepcopy(initial)
        accepted=True
        for d in code:
            status=a.call(0x880cf39c,F,int(d))
            if status==7:accepted=False;break
            assert status==0,(lang,code,status)
        if not accepted:
            rows.append({'language':lang,'digits':code,'first_candidates':[]})
            continue
        if lang=='English':
            words=[]
            for i in range(4):
                n=a.read(F+10,2);w=a.read(F+14,2)+a.read(F+16,2)
                word=bytes(a.ram[T-a.rb+2*(n-w):T-a.rb+2*n]).decode('utf-16-le')
                if word in words:break
                words.append(word)
                if a.call(0x880cf39c,F,17):break
        else:
            if lang=='Chinese':call(a,0x880d7330,F,1)
            words=[chr(a.read(F+0x30a+2*i,2)) for i in range(a.read(F+0x31e,2))]
        rows.append({'language':lang,'digits':code,'first_candidates':words})
report={'method':'Original ARM routines interpreted using scripts/t9/arm.py; synthetic RAM and public LDB only. Not handset captures or a proof of all ARM opcodes.','firmware_sha256':hashlib.sha256(e.rom).hexdigest(),'cases':rows}
path=e.ROOT/'docs/evidence/t9-runtime-probe.json'
path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(f'{len(rows)} fixture cases written to {path.relative_to(e.ROOT)}')
