#!/usr/bin/env python3
"""Recover native theme data and bounded drawing evidence. Never execute firmware."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
EXPECTED = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
b = SOURCE.read_bytes()
assert hashlib.sha256(b).hexdigest() == EXPECTED
base = 0x88000000
palette = struct.unpack_from('<I', b, 0x3a49c0)[0] - base
assert palette == 0x6f73b4
rows = [list(struct.unpack_from('<7H', b, palette + i * 14)) for i in range(6)]
style = struct.unpack_from('<I', b, 0xa79e00)[0] - base
assert style == 0x74cd5c
out = ROOT / 'assets/ui'
out.mkdir(exist_ok=True)
data = {
    'source': str(SOURCE.relative_to(ROOT)), 'source_sha256': EXPECTED,
    'palette_offset': hex(palette), 'palette_sha256': hashlib.sha256(b[palette:palette+84]).hexdigest(),
    'accents_rgb565': rows,
    'startup_fallback_accent': 1,
    'startup_note': '0x883a45d8 initializes preference to 1 before loading saved setting. The current handset setting was not read.',
    'candidate_style_offset': hex(style),
    'candidate_text_rgb565': struct.unpack_from('<H', b, style + 0x18)[0],
    'candidate_background_shade': 5, 'candidate_selection_shade': 2,
    'candidate_page_size': 4, 'candidate_row_height': 30, 'candidate_inset_row_height': 24,
    'candidate_width_padding': 31, 'candidate_flip_threshold': 184,
    'limits': 'Native Chinese draw calls and geometry recovered. Dictionary, exact font rasterization, all mode/focus branches and current saved theme remain unresolved.'
}
(out / 'native-theme.json').write_text(json.dumps(data, indent=2) + '\n')
# The synthetic ELF is just an LLVM disassembly wrapper. No source image is modified.
names = b'\0.text\0.shstrtab\0'
so = (256 + len(b) + len(names) + 3) & ~3
h = struct.pack('<16sHHIIIIIHHHHHH', b'\x7fELF\x01\x01\x01'+bytes(9), 2,40,1,base,0,so,0x5000000,52,0,0,40,3,2)
e = (h.ljust(256,b'\0') + b + names).ljust(so,b'\0') + bytes(40)
e += struct.pack('<10I',1,1,6,base,256,len(b),0,0,4,0) + struct.pack('<10I',7,3,0,0,256+len(b),len(names),0,0,1,0)
ranges = [
    ('accent-selection',0x3a45d8,0x3a4692),
    ('chinese-popup-geometry',0x1c9ccc,0x1c9e8e),
    ('chinese-draw-dispatch',0x1cb856,0x1cb8c2),
    ('candidate-draw-setup',0xa79d26,0xa79db0),
    ('candidate-draw-rows',0xa79e3c,0xa79f5c),
    ('candidate-pair-setup',0xa79f5c,0xa79fe4),
    ('candidate-pair-paint',0xa7a0ea,0xa7a19e),
]
folder = ROOT / 'docs/evidence'
folder.mkdir(exist_ok=True)
index = []
with tempfile.TemporaryDirectory(prefix='ta1278-style-') as tmp:
    elf = Path(tmp) / 'user.analysis-only.elf'
    elf.write_bytes(e)
    for name, lo, hi in ranges:
        argv = ['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi','--mcpu=cortex-a5',f'--start-address={base+lo}',f'--stop-address={base+hi}',str(elf)]
        text = subprocess.check_output(argv, text=True).replace(str(elf),'user.analysis-only.elf')
        (folder / (name+'.txt')).write_text(text)
        index.append({'name':name,'start':hex(base+lo),'end_exclusive':hex(base+hi),'sha256':hashlib.sha256(b[lo:hi]).hexdigest()})
(folder/'native-ui-ranges.json').write_text(json.dumps({'source_sha256':EXPECTED,'ranges':index},indent=2)+'\n')
print('Verified source; extracted 6 native palettes, candidate style, 7 disassembly ranges.')
