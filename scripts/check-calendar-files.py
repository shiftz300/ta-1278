#!/usr/bin/env python3
"""Verify schedule child controls and related file windows. Static, no USB/audio."""
import hashlib
import json
from pathlib import Path
from firmware_disassembly import disassemble_arm

root = Path(__file__).resolve().parents[1]
source = root / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
b = source.read_bytes()
sha = lambda data: hashlib.sha256(data).hexdigest()
expected = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
assert sha(b) == expected
names = {'MMISCH_EDIT_WIN_ID', 'MMISCH_EDIT_OPTION_WIN_ID', 'MMISCH_EDIT_REMINDER_WIN_ID',
         'MMIFMM_DEL_SELECT_FILE_WIN_ID', 'MMIFMM_MOVE_FILE_WIN_ID', 'MMIFMM_COPY_SELECT_FILE_WIN_ID'}
rows = [r for r in json.loads((root / 'nokia-ta1278-research/analysis/ui/window-semantics.json').read_text())['windows'] if r.get('name') in names]
assert len(rows) == len(names)
for row in rows:
    start = int(row['offset'],16)
    assert sha(b[start:start+row['size']]) == row['sha256']
form = next(r for r in rows if r['name'] == 'MMISCH_EDIT_WIN_ID')
editors = [a for a in form['attributes'] if a.get('framework_opcode') == 'CREATE_CHILD_EDIT_TEXT']
assert len(editors) == 3
assert all(a['named_arguments']['_STR_MAX_LEN_'] == '0x28' for a in editors)
keys = next(a for a in form['attributes'] if a.get('framework_opcode') == 'MMK_CREATE_SOFTKEY')
assert keys['named_arguments']['_MID_SOFT_ID_'] == '0x40180121'
assert keys['strings']['0x40180121']['1'] == '储存'
ranges = [(0x880572d8,0x88057b00),(0x88ab2e3a,0x88ab3558)]
out = root / 'docs/evidence'
(out/'calendar-files-thumb.txt').write_text(disassemble_arm(b,0x88000000,ranges))
(out/'calendar-files.json').write_text(json.dumps({
    'source':str(source.relative_to(root)), 'source_sha256':expected, 'windows':rows,
    'verified':['Schedule form contains three 40-unit text children and separate date/time children.',
                'Schedule form center softkey is Save. The separate edit-options window is registered.'],
    'ranges':[{'start':hex(a),'end':hex(z),'sha256':sha(b[a-0x88000000:z-0x88000000])} for a,z in ranges],
    'limits':['Window registration alone does not identify the exact file checkbox page or its complete handler flow.',
              'File selection geometry is from the existing handset screenshot; select-all, confirmation, preflight and nonrecursive deletion are explicit host policies.',
              'Calendar digit validation, defaults and recurrence are local policies. The full shared native edit widgets are not recovered.'],
},ensure_ascii=False,indent=2)+'\n')
print('Verified six descriptors, three 40-unit schedule editors and center Save; saved bounded listings.')
