#!/usr/bin/env python3
"""Verify timer descriptors and retain bounded disassembly. Offline; no USB/audio."""
import hashlib
import json
from pathlib import Path
from firmware_disassembly import disassemble_arm

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
EXPECTED = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
sha = lambda b: hashlib.sha256(b).hexdigest()
b = SOURCE.read_bytes()
assert sha(b) == EXPECTED
names = {'MMIACC_PRIMO_CT_WIN_ID', 'MMIACC_COUTIME_WIN_ID'}
windows = [r for r in json.loads((ROOT / 'nokia-ta1278-research/analysis/ui/window-semantics.json').read_text())['windows'] if r.get('name') in names]
assert len(windows) == 2
for r in windows:
    offset = int(r['offset'], 16)
    assert sha(b[offset:offset + r['size']]) == r['sha256']
current = next(r for r in windows if r['name'] == 'MMIACC_PRIMO_CT_WIN_ID')
assert current['handler_thumb'] == '0x880a437f'
text = next(a for a in current['attributes'] if a.get('framework_opcode') == 'CREATE_CHILD_EDIT_TEXT')
assert text['named_arguments']['_STR_MAX_LEN_'] == '0x14'
keys = next(a for a in current['attributes'] if a.get('framework_opcode') == 'MMK_CREATE_SOFTKEY')
assert keys['named_arguments']['_MID_SOFT_ID_'] == '0x40000b2c'
ranges = [(0x880a437e, 0x880a4730)]
out = ROOT / 'docs/evidence'
listing = disassemble_arm(b, 0x88000000, ranges)
(out / 'native-timer-thumb.txt').write_text(listing)
result = {
    'source': str(SOURCE.relative_to(ROOT)), 'sha256': EXPECTED,
    'windows': windows,
    'code': [{'start': hex(s), 'end': hex(e), 'sha256': sha(b[s-0x88000000:e-0x88000000])} for s, e in ranges],
    'verified': ['Current form has separate time and text children.', 'Text child maximum is 20 UTF-16 units.', 'Start belongs to the center softkey, unlike the older countdown descriptor.'],
    'limits': ['The two-byte readiness predicate at 0x880a46fa has not been fully traced to named fields.', 'Nonempty-name gating and running/expired transitions are handset observations, not conclusions from descriptor presence alone.', 'Native arithmetic, timer scheduling and every invalid-entry branch remain unverified.'],
}
mmi = SOURCE.with_name('mmi.bin').read_bytes()
assert sha(mmi) == '14c573027f375ea6b5f8ead2c6c432f2773f373faaa3c8a33261d0de9737886c'
catalog = json.loads((ROOT / 'nokia-ta1278-research/analysis/ui/image-catalog.json').read_text())
art = next(r for r in catalog if r.get('resource_id') == '0x4017002e')
assert sha(mmi[art['offset']:art['offset'] + art['size']]) == art['sha256']
png = (ROOT / 'nokia-ta1278-research/analysis/ui' / art['preview']).resolve()
result['hourglass'] = {'resource_id': art['resource_id'], 'offset': art['offset_hex'],
                       'size': art['size'], 'raw_sha256': art['sha256'],
                       'png': str(png.relative_to(ROOT)), 'png_sha256': sha(png.read_bytes()),
                       'placement': [94, 132], 'basis': 'Original 52x52 asset visually matched to timer-expired.png. Render-call chain not yet traced.'}
(out / 'native-timer.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print('Verified firmware, two countdown descriptors, 20-unit name and center Start; saved bounded code.')
