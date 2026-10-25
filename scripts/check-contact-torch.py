#!/usr/bin/env python3
"""Verify original torch assets/tables and phonebook control descriptors. Offline."""
import hashlib
import json
import struct
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
EXPECTED = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
sha=lambda data:hashlib.sha256(data).hexdigest()
b=SOURCE.read_bytes()
assert sha(b)==EXPECTED
mmi=SOURCE.with_name('mmi.bin').read_bytes()
MMI_SHA='14c573027f375ea6b5f8ead2c6c432f2773f373faaa3c8a33261d0de9737886c'
assert sha(mmi)==MMI_SHA
assert list(struct.unpack_from('<6I',b,0x7a9b50))==[0x4017002c,0x4017002b,0x4017002d,0x4017002a,0x40170029,0x40170028]
assert struct.unpack_from('<I',b,0x42c180)[0]==0x40170020
assert struct.unpack_from('<H',b,0x42c012)[0]==0x1de4 # Adds seven to select the off image.
ui=ROOT/'nokia-ta1278-research/analysis/ui'
ids={0x40170020,0x40170027,*range(0x40170028,0x4017002e)}
assets=[]
for row in json.loads((ui/'image-catalog.json').read_text()):
 if not isinstance(row.get('resource_id'),str) or int(row['resource_id'],16) not in ids:continue
 raw=mmi[row['offset']:row['offset']+row['size']]
 assert sha(raw)==row['sha256']
 png=(ui/row['preview']).resolve()
 assets.append({'id':row['resource_id'],'resource_offset':row['offset_hex'],
  'resource_sha256':row['sha256'],'png':str(png.relative_to(ROOT)),
  'png_sha256':sha(png.read_bytes()),'width':row['width'],'height':row['height']})
assert len(assets)==8
names={'MMIPB_ADD_IN_NV_WIN_ID','MMIPB_ENTRY_LIST_WITH_SEARCH_WIN_ID','MMIACC_FLASHLIGHT_WIN_ID'}
windows=[]
for row in json.loads((ui/'window-semantics.json').read_text())['windows']:
 if row.get('name') not in names:continue
 offset=int(row['offset'],16)
 assert sha(b[offset:offset+row['size']])==row['sha256']
 windows.append(row)
result={'source_sha256':EXPECTED,'resource_source':'mmi.bin','resource_source_sha256':MMI_SHA,'assets':assets,'windows':windows,
 'torch':{'power_off':'0x40170027','power_on_by_theme':['0x4017002c','0x4017002b','0x4017002d','0x4017002a','0x40170029','0x40170028'],
 'theme_table_offset':'0x7a9b50','draw_handler':'0x8842bf2a','event_handler':'0x8842c090',
 'placement':[94,100],'placement_source':'Measured public off/on captures; draw code centers the image in the available rectangle.'},
 'captures':['torch-off','torch-on','contacts-options','contact-new','contact-phone-fields','contact-birthday','contact-ringtone','contact-group','test-contact-details','test-contact-options','test-contact-delete-dialog'],
 'local_policies':['Phone calling and speed dial stay grey/inert by owner instruction.',
 'The empty-store caption is unobserved; the local empty list is blank with a search underline and root menu.',
 'Search is a local case-insensitive substring filter; exact native matching is unresolved.',
 'New-form save uses a one-item local menu; its native popup has not been sampled.',
 'Picture assignment, contact sharing, group management and ringtone selection are unavailable.',
 'Only previously reviewed synthetic test contacts are used in public comparisons.'],
 'ranges':[{'start':hex(0x88000000+a),'end_exclusive':hex(0x88000000+z),'sha256':sha(b[a:z]),'hex':b[a:z].hex()} for a,z in [(0x42bf2a,0x42c08a),(0x42c090,0x42c176),(0x7a9b20,0x7a9b68)]]}
(ROOT/'docs/evidence/contact-torch-layout.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(f'Verified firmware, {len(assets)} original images and {len(windows)} window descriptors.')
