#!/usr/bin/env python3
"""Inspect the original kernel's LCD diagnostic path. No USB or firmware execution."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/kernel.bin'
EXPECTED = 'b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303'
b = SOURCE.read_bytes()
assert hashlib.sha256(b).hexdigest() == EXPECTED
handler = struct.unpack_from('<I', b, 0x1a54f0)[0]
assert handler == 0x8100afc9
assert b[0x1a5372:0x1a5374] == bytes.fromhex('3320')  # movs r0, #0x33
assert b[0x1a5310:0x1a5314] == bytes.fromhex('a020a071')  # response type 0xa0
base = (handler & ~1) - 0x1a51c8
assert base == 0x80e65e00
names = b'\0.text\0.shstrtab\0'
so = (256 + len(b) + len(names) + 3) & ~3
h = struct.pack('<16sHHIIIIIHHHHHH', b'\x7fELF\x01\x01\x01'+bytes(9), 2,40,1,base,0,so,0x5000000,52,0,0,40,3,2)
e = (h.ljust(256,b'\0') + b + names).ljust(so,b'\0') + bytes(40)
e += struct.pack('<10I',1,1,6,base,256,len(b),0,0,4,0) + struct.pack('<10I',7,3,0,0,256+len(b),len(names),0,0,1,0)
out = ROOT / 'docs/evidence'
out.mkdir(exist_ok=True)
ranges = [('lcd-diagnostic', 0x1a5100, 0x1a5378),
          ('lcd-diagnostic-caller', 0x163c34, 0x163c74),
          ('lcd-diagnostic-registration', 0x1fbe4, 0x1fbf0),
          ('diagnostic-key-events', 0x1f30c, 0x1f456),
          ('diagnostic-key-dispatch', 0x1f456, 0x1f4c6),
          ('diagnostic-key-dispatch-call', 0x1f5b8, 0x1f5c6),
          ('runtime-ucom-control', 0x177bc4, 0x177cc0)]
records = []
with tempfile.TemporaryDirectory(prefix='ta1278-lcd-') as tmp:
    elf = Path(tmp) / 'kernel.analysis-only.elf'
    elf.write_bytes(e)
    for name, lo, hi in ranges:
        args = ['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi','--mcpu=cortex-a5',f'--start-address={base+lo}',f'--stop-address={base+hi}',str(elf)]
        text = subprocess.check_output(args, text=True).replace(str(elf),'kernel.analysis-only.elf')
        (out / (name+'.txt')).write_text(text)
        records.append({'start_file_offset':hex(lo), 'end_exclusive':hex(hi), 'sha256':hashlib.sha256(b[lo:hi]).hexdigest()})
assert 0x1f4c6 + 2*b[0x1f4c7] == 0x1f5b8
assert b[0x1f39c:0x1f39e] == bytes.fromhex('6868')  # ldr r0, [r5, #4], r5=request+8
result = {
    'source':str(SOURCE.relative_to(ROOT)), 'source_sha256':EXPECTED,
    'mapping_base':hex(base), 'handler_thumb_pointer':hex(handler),
    'registration_file_offset':'0x1a5360', 'handler_file_offset':'0x1a51c8',
    'key_request_type':1, 'key_subtypes':{'tap':0, 'down':1, 'up':2},
    'key_payload_bytes':8, 'key_uint32_request_offset':12,
    'request_type':51, 'response_type':160, 'header_bytes':8, 'max_chunk_bytes':32768,
    'ranges':records,
    'status':'Static matching path exists. Runtime registration, transport access and actual screenshot acquisition are unverified.',
    'note':'All file offsets include the DHTB header. No command was sent to a device.'
}
(out/'lcd-diagnostic.json').write_text(json.dumps(result,indent=2)+'\n')
print('Verified native LCD diagnostic registration and JPEG response path; runtime access remains unverified.')
