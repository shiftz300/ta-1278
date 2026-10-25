#!/usr/bin/env python3
"""Build RAM debug hook with installed Apple clang, no linker installation.
Extracts only ELF .text sections and refuses unresolved .text relocations.
No device access. Output loader is modified, never a flashable firmware image.
"""
from pathlib import Path
import subprocess,struct,hashlib,json
R=Path(__file__).resolve().parents[1];B=R/'tools/build'
def text_section(p):
    b=p.read_bytes();assert b[:7]==b'\x7fELF\x01\x01\x01'
    shoff=struct.unpack_from('<I',b,32)[0];sz,n,si=struct.unpack_from('<HHH',b,46)
    sections=[struct.unpack_from('<10I',b,shoff+i*sz) for i in range(n)]
    st=sections[si];names=b[st[4]:st[4]+st[5]]
    for i,s in enumerate(sections):
        name=names[s[0]:].split(b'\0',1)[0]
        if name==b'.text':idx=i;text=b[s[4]:s[4]+s[5]]
    assert all(not(s[1] in (4,9) and s[7]==idx and s[5]) for s in sections),'text relocation unresolved'
    return text
for name in ('fdl2_debug_entry.S','fdl2_debug_hook.c'):
    subprocess.run(['clang','--target=arm-none-eabi','-mcpu=cortex-a5','-mthumb','-Os','-ffreestanding',
      '-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-c',str(R/'tools'/name),'-o',str(B/(name+'.o'))],check=True)
stub=text_section(B/'fdl2_debug_entry.S.o');body=text_section(B/'fdl2_debug_hook.c.o');assert len(stub)==64
payload=stub+body;assert len(payload)<0x10000
loader=bytearray((R/'analysis/work/community-pac/fdl2.bin').read_bytes())
original=hashlib.sha256(loader).hexdigest();assert original=='3c6699d6add21b58f2b87e84e6fdf847e39d9d2ef70f9b19d6ba9d59e4adb7bd'
assert loader[0xc55c:0xc564]==bytes.fromhex('2de9ff4788461e46')
loader[0xc55c:0xc564]=bytes.fromhex('dff800f001000083') # LDR.W PC,[PC,#0]; 83000001
(B/'fdl2-debug-ram.bin').write_bytes(loader);(B/'fdl2-debug-hook.bin').write_bytes(payload)
meta={'status':'built, not yet loaded or validated','original_sha256':original,
 'patched_sha256':hashlib.sha256(loader).hexdigest(),'hook_sha256':hashlib.sha256(payload).hexdigest(),
 'hook_bytes':len(payload),'hook_address':'0x83000000','patched_offset':'0xc55c',
 'scope':'RAM-only diagnostic extension, not a firmware image'}
(B/'fdl2-debug-build.json').write_text(json.dumps(meta,indent=2)+'\n');print(json.dumps(meta,indent=2))
