#!/usr/bin/env python3
"""Read-only checks of service/T9 anchors in immutable system firmware.
Writes only bounded listings and JSON under docs/evidence; never executes firmware.
"""
import hashlib
import json
import struct
from pathlib import Path
from firmware_disassembly import disassemble_arm
ROOT = Path(__file__).resolve().parents[1]
R = ROOT / 'nokia-ta1278-research'
U = R / 'analysis/ui'
def sha(b): return hashlib.sha256(b).hexdigest()
manifest = json.loads((R / 'firmware/public-system-manifest.json').read_text())
images = {}
for name in ['user', 'mmi']:
    item = next(x for x in manifest['files'] if x['component'] == name)
    b = (R / item['file']).read_bytes()
    assert sha(b) == item['sha256']
    images[name] = b
user, mmi = images['user'], images['mmi']
strings = []
for row in json.loads((U / 'menu-sms-texts.json').read_text()):
    if row['group'] != 8 or row['index'] not in [6, 45, 47, 56]: continue
    for lang, text in row['translations'].items():
        at = int(row['offsets'][lang], 16)
        if mmi[at:at+2] == b'\x80\0':
            length = struct.unpack_from('<H', mmi, at+2)[0]
            value = mmi[at+4:at+4+length].decode('utf-16-le')
        else:
            value = mmi[at:mmi.index(0, at)].decode('ascii')
        assert value == text
    strings.append(row)
names = []
# Name registrations establish separate windows; they do not prove runtime reachability.
def walk(x):
    if isinstance(x, dict):
        if x.get('name') in ['MMISMS_CB_NEWMSG_WIN_ID', 'MMISMS_ETWS_NEWMSG_WIN_ID', 'MMISMS_STATUE_REPORT_WIN_ID']:
            at = int(x['offset'], 16)
            assert user[at:user.index(0, at)].decode() == x['name']
            names.append(x)
        for v in x.values(): walk(v)
    elif isinstance(x, list):
        for v in x: walk(v)
walk(json.loads((U / 'window-names.json').read_text()))
windows = []
ranges = []
for row in json.loads((U / 'window-descriptors.json').read_text())['windows']:
    if row.get('name') not in ['MMISMS_CB_READ_WIN_ID', 'MMISMS_CB_CLASS0_READ_WIN_ID']: continue
    at, size = int(row['offset'], 16), row['size']
    assert sha(user[at:at+size]) == row['sha256']
    start = int(row['handler_thumb'], 16) & ~1
    ranges.append((start, start+384))
    windows.append({k:row[k] for k in ['offset','size','sha256','name','window_id','handler_thumb']})
listing = ROOT / 'docs/evidence/service-broadcast-thumb.txt'
listing.write_text(disassemble_arm(user, 0x88000000, ranges))
t9 = json.loads((U / 't9-resource-analysis.json').read_text())
for x in t9['engine_strings']:
    at = int(x['offset'], 16)
    assert user[at:user.index(0,at)].decode() == x['text']
ldb = []
for row in t9['alphabetic'] + t9['chinese']:
    if 'file' not in row: continue
    data = (U / row['file']).read_bytes()
    assert len(data) == row['size'] and sha(data) == row['sha256']
    ldb.append({k:row[k] for k in ['file','size','sha256','status']})
sms = json.loads((ROOT/'docs/evidence/sms-lifecycle.json').read_text())
for row in sms['verified_ranges']:
    at = int(row['start'],16)-0x88000000
    end = int(row['end_exclusive'],16)-0x88000000
    assert sha(user[at:end]) == row['sha256']
result = {'inputs':{k:sha(v) for k,v in images.items()}, 'broadcast_texts':strings,
    'notification_name_registrations':names, 'broadcast_reader_descriptors':windows,
    'bounded_listing':{'file':str(listing.relative_to(ROOT)), 'sha256':sha(listing.read_bytes()),
        'ranges':[[hex(a),hex(b)] for a,b in ranges]},
    'sms_verified_ranges':sms['verified_ranges'], 't9_engine_strings':t9['engine_strings'],
    't9_files':ldb,
    'limits':['Window names and labels do not prove native ETWS support is enabled on this variant.',
        'Only the first 384 bytes of two broadcast reader callbacks are listed, not complete handlers.',
        'No ETWS/CMAS transport, geographic rules, severity mapping, interrupt policy or mandatory alert sound is recovered.',
        'The host Backend API, request IDs, stream cursor, persistence and alert queue rules are new replica code.',
        'T9 dictionaries are hash checked; native trie, candidate ranking, learning and complete engine remain unimplemented.']}
(ROOT/'docs/evidence/service-native-anchors.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(f'Checked {len(strings)} multilingual labels, {len(names)} notification names, {len(windows)} reader descriptors, {len(ldb)} LDB files and SMS branch hashes.')
