#!/usr/bin/env python3
"""Check embedded UI images against MMI bytes. Offline, no device/audio access."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
ui = root / 'nokia-ta1278-research/analysis/ui'
mmi = (root / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/mmi.bin').read_bytes()
sha = lambda b: hashlib.sha256(b).hexdigest()
expected = '14c573027f375ea6b5f8ead2c6c432f2773f373faaa3c8a33261d0de9737886c'
assert sha(mmi) == expected
ids = {2656, 2657, 2658, 2484, 2486, 2488, 2490, 2492, 2493, 2461, 2474, 2524, 2525, 2523}
rows = []
for r in json.loads((ui / 'image-catalog.json').read_text()):
    if int(r['file'].split('-')[0]) not in ids:
        continue
    assert sha(mmi[r['offset']:r['offset'] + r['size']]) == r['sha256']
    png = (ui / r['preview']).resolve()
    rows.append({'resource_id': r['resource_id'], 'offset': r['offset_hex'],
                 'raw_sha256': r['sha256'], 'size': r['size'],
                 'png': str(png.relative_to(root)), 'png_sha256': sha(png.read_bytes()),
                 'dimensions': [r['width'], r['height']]})
assert len(rows) == len(ids)
(root / 'docs/evidence/utility-art.json').write_text(json.dumps({
    'mmi_sha256': expected, 'images': rows,
    'binding_evidence': 'Visual matches against native file, timer, stopwatch, recorder and calculator captures. Presence and image hashes do not prove every native event binding.',
}, indent=2) + '\n')
print(f'Verified {len(rows)} original utility control images against MMI bytes.')
