#!/usr/bin/env python3
"""Recover alarm picker tables and bounded handlers; never execute firmware."""
import argparse
import hashlib
import json
import struct
from pathlib import Path
from firmware_disassembly import disassemble_arm

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--disassemble', action='store_true')
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
source = root/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
b = source.read_bytes()
sha = hashlib.sha256(b).hexdigest()
assert sha == '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
u32 = lambda at: struct.unpack_from('<I', b, at)[0]
strings = json.loads((root/'nokia-ta1278-research/analysis/ui/strings-zh-Hans-observed.json').read_text())
text = {r['resource_id']: r['text'] for r in strings}
assert u32(0x11a2a4) == 0x884ad9a8
assert u32(0x1194bc) == 0x884ad9e0
assert u32(0x1190c0) == 0x884acf7d
weekday_ids = struct.unpack_from('<7I', b, 0x4ad9a8)
snooze_ids = struct.unpack_from('<5I', b, 0x4ad9e0)
weekday_bits = list(b[0x4acf7d:0x4acf84])
assert weekday_bits == [2,4,8,16,32,64,1]
snooze_labels = [text[hex(i)] for i in snooze_ids]
assert snooze_labels == ['2分钟','5分钟','10分钟','15分钟','20分钟']
values = {'weekday_labels':[text[hex(i)] for i in weekday_ids],
          'weekday_bits':weekday_bits, 'snooze_labels':snooze_labels,
          'snooze_minutes':[int(s.removesuffix('分钟')) for s in snooze_labels]}
(root/'assets/ui/native-alarm.json').write_text(json.dumps(values,ensure_ascii=False,indent=2)+'\n')
ranges=[(0x881184c8,0x8811866a),(0x88118aec,0x88118da8),
        (0x88118da8,0x88118e72),(0x8811907c,0x8811915c),(0x8811a004,0x8811a0c4)]
report={'source':str(source.relative_to(root)),'source_sha256':sha,
 'generated_asset_sha256':hashlib.sha256((root/'assets/ui/native-alarm.json').read_bytes()).hexdigest(),
 'tables':{'weekday_ids_offset':'0x4ad9a8','snooze_ids_offset':'0x4ad9e0','weekday_bits_offset':'0x4acf7d',**values},
 'code_findings':{
  'repeat_back':'0x88118e18..0x88118e68: E002 gets up to seven checked rows, ORs their mask bytes, stores draft repeat mode0 for none,1 for all,2 for custom, then closes. Back applies to the draft, not NV.',
  'snooze':'0x881190f0..0x88119142 builds five choices. E004 at0x88119144 writes selected index to draft+0x18; E002 branches directly to close at0x8811914c.',
  'form':'Descriptor user+0x4ad54c binds an inline time control and a20-unit name editor. Softkey resource0x40000b28 is Save. Handler0x88118aec routes E004 through focused-control dispatch; time/name invoke save while repeat/ring/snooze open controls.'},
 'ranges':[{'start':hex(lo),'end_exclusive':hex(hi),'sha256':hashlib.sha256(b[lo-0x88000000:hi-0x88000000]).hexdigest()} for lo,hi in ranges],
 'limits':['Form geometry is sampled. Individual time-digit advance/rejection is replica policy; the shared edit-time widget was not fully traced.',
 'Sound playback, OS wake from sleep and all alarm-interruption paths remain outside this pass.']}
(root/'docs/evidence/alarm-recovery.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
if args.disassemble:
 (root/'docs/evidence/alarm-thumb.txt').write_text(disassemble_arm(b,0x88000000,ranges))
print('Verified seven weekday masks, five snooze choices, and bounded handler ranges.')
