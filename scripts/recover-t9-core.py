#!/usr/bin/env python3
"""Bounded static T9 recovery. System image only; no native execution or user dictionary read."""
from pathlib import Path
import hashlib,json,struct
from firmware_disassembly import disassemble_arm
ROOT=Path(__file__).resolve().parents[1]
U=ROOT/'nokia-ta1278-research/analysis/ui'
source=ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
b=source.read_bytes(); sha=lambda x:hashlib.sha256(x).hexdigest()
assert sha(b)=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
BASE=0x88000000
# Check the instructions which determine the translated arithmetic.
checks={0x880d92a8:0xe202003f,0x880d92ac:0xe1b02322,0x880d92b0:0xe2800001,
0x880d92b8:0xe2800003,0x880d92c4:0x02801001,0x880d92c8:0x01a010a1,
0x880d92cc:0x00810080,0x880d92d8:0x02800001,0x880d92dc:0x03a01005,
0x880d92e0:0x008100a0,0x880d92e8:0xe350003f,0x880d9308:0xe2800001,
0x880d930c:0xe3c00801,0x880e699c:0xe1800401,
0x880d7020:0xe3540008,0x880d7024:0x93a00e3b,0x880d7028:0x918040b5,
0x880e56fc:0xe3c00801,0x880e55ec:0xe3c00b02}
for address,word in checks.items(): assert struct.unpack_from('<I',b,address-BASE)[0]==word,hex(address)
# Thumb ADR encoded target, not LLVM's misleading relative symbol annotation.
logs=[]
for at, expected, callsite, target in [
(0x88a780aa,'T9AWInitialize',0x88a780a2,0x880e4e44),
(0x88a780e4,'T9AWClrLdbCompletion',0x88a780de,0x880e56c8),
(0x88a780f4,'T9AWClrUdbCustWordCompletion',0x88a780ec,0x880e55b8),
(0x88a78c5c,'T9CCInitialize',0x88a78c54,0x880d7718),
(0x88a78d06,'T9CCSetDynCtxtFence',0x88a78cfe,0x880d71d0),
(0x88a78d1a,'T9CCSetDynCtxtSearchDepth',0x88a78d12,0x880d6ff8)]:
    word=struct.unpack_from('<H',b,at-BASE)[0];assert word&0xf800==0xa000
    string=((at+4)&~3)+4*(word&255);off=string-BASE
    text=b[off:b.index(0,off)].decode();assert expected in text
    listing=disassemble_arm(b,BASE,[(callsite,callsite+4)])
    assert 'blx\t'+hex(target) in listing
    logs.append({'name':expected,'call':hex(callsite),'target_arm':hex(target),'log_adr':hex(at),'log_string':hex(string),'text':text})
thumb_ranges=[(0x88a78014,0x88a78106),(0x88a78b8c,0x88a78d2c)]
arm_ranges=[(0x880d929c,0x880d9314),(0x880e6950,0x880e69a4),(0x880d6ff8,0x880d7054),
 (0x880d71d0,0x880d7230),(0x880e55b8,0x880e55fc),(0x880e56c8,0x880e570c)]
ranges=[]
for mode,rs in [('thumb',thumb_ranges),('arm',arm_ranges)]:
    path=ROOT/f'docs/evidence/t9-core-{mode}.txt'
    path.write_text(disassemble_arm(b,BASE,rs,thumb=mode=='thumb'))
    for start,end in rs:ranges.append({'mode':mode,'start':hex(start),'end':hex(end),'sha256':sha(b[start-BASE:end-BASE])})
# Candidate key alphabet layout in the two extracted LDBs. Contents are decoded;
# pointer selection and rank interpretation still need engine-side verification.
keymaps=[]
for variant in range(2):
    path=U/f't9/t9-aw-0109-variant-{variant}.ldb';data=path.read_bytes();at=0x80;groups=[]
    assert len(data)==36651
    for size in data[0x76:0x80]:
        text=data[at:at+size*2].decode('utf-16-le');digits=[c for c in text if c.isascii() and c.isdigit()]
        assert len(digits)==1
        groups.append({'digit':digits[0],'offset':hex(at),'symbols':text});at+=size*2
    assert at==0xde
    keymaps.append({'file':str(path.relative_to(ROOT)),'sha256':sha(data),'groups':groups,'status':'decoded data layout; not a proven candidate ordering'})
result={'source':str(source.relative_to(ROOT)),'sha256':sha(b),'mode':'Thumb wrapper calls ARM engine via BLX',
'api_calls':logs,'ranges':ranges,'checked_words':{hex(k):hex(v) for k,v in checks.items()},
'confirmed':{'aw_state_bytes_cleared':0x5ac,'cc_state_bytes_cleared':0x814,
'aw_disabled_completion_masks':['0x10000','0x800'],'cc_depth_default':4,'cc_depth_max':8,
'cc_fence_argument':'u16(wrapper+0x340) >> 1','record_tag':'top two bits choose class, low six bits plus one = n',
'record_span_classes':['n+3','2*n+ceil(n/2)+3','ceil(n/2)+5','n unless tag ff: BE16(next bytes)+1 modulo 65536'],
'record_extended_endian':'big','record_helper':'runtime circular-buffer records, not established as an LDB trie parser'},
'english_key_alphabets':keymaps,
'limits':['Native candidate traversal/ranking and Chinese block interpretation are not yet recovered.',
'Guard/rebuild functions called by the translated setters are not emulated.',
'No private learning database was accessed and no original engine code was executed.',
'Shared persistent/context learning in the replica is a new implementation, not native algorithm equivalence.']}
(ROOT/'docs/evidence/t9-core-recovery.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print('Verified six native API call sites, ARM arithmetic, context settings and two LDB key alphabets.')
