#!/usr/bin/env python3
"""Extract Snake maps and bind sprites to native call sites. Offline; executes no firmware."""
import argparse
import hashlib
import json
import struct
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--disassemble', action='store_true', help='Also generate bounded listings with installed Xcode LLVM')
args = parser.parse_args()
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
EXPECTED = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
b = SOURCE.read_bytes()
assert hashlib.sha256(b).hexdigest() == EXPECTED
u32 = lambda off: struct.unpack_from('<I', b, off)[0]
assert [u32(x) for x in [0x690cec, 0x690cf0, 0x690cf4, 0x690cf8]] == [0x882032b8, 0x88203438, 0x882035b8, 0x88203738]
assert u32(0x690c90) == 0x401f0009
maps = b[0x2032b8:0x2032b8+7*192]
assert len(maps) == 1344 and set(maps) <= set(range(8))
out=ROOT/'assets/snake';out.mkdir(exist_ok=True)
init_path=ROOT/'nokia-ta1278-research/analysis/extracted/kernel/init_81c20000.bin'
init=init_path.read_bytes()
assert hashlib.sha256(init).hexdigest() == '76b95ae0d1948b677a38d8e6311b65f154642fb19723a4dcdcc8ec0a2da04210'
state=0x81c3bea8-0x81c20000
assert struct.unpack_from('<H',init,state+0x2c)[0] == 1
assert u32(0x690c8c) == 0x81c3bedc
speeds=init[state+0x34:state+0x34+18]
assert struct.unpack('<9H',speeds)==(250,225,200,175,150,125,100,90,88)
(out/'tick-ms.bin').write_bytes(speeds)
# Verify the immediate operands independently of the Rust implementation.
for off,opcode in {0x69024c:0x2108,0x69024e:0x2206,0x690242:0x2603,
                   0x6904aa:0x290c,0x6904b4:0x2910,0x99f6e:0x2801,0x99f76:0x2808,0x690a52:0x2614,0x690ac8:0x2332,
                   0x690acc:0x22b4,0x690ace:0x2146,0x690ad0:0x201e}.items():
 assert struct.unpack_from('<H',b,off)[0]==opcode,hex(off)
assert u32(0x690cd8) == 0x401c002c
assert b[0x690a54:0x690a58].hex() == '40f22715'  # MOVW r5, 0x0127
assert b[0x690a5a:0x690a5e].hex() == '4ff60007'  # MOVW r7, 0xf800
(out/'maps.bin').write_bytes(maps)
catalog=json.loads((ROOT/'nokia-ta1278-research/analysis/ui/image-catalog.json').read_text())
images=[r for r in catalog if str(r.get('resource_id','')).startswith('0x401f')]
assert len(images)==11
records=[]
for row in images:
 p=(ROOT/'nokia-ta1278-research/analysis/ui'/row['preview']).resolve()
 records.append({'id':row['resource_id'],'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'width':row['width'],'height':row['height']})
result={'source':str(SOURCE.relative_to(ROOT)),'source_sha256':EXPECTED,'maps':{'path':'assets/snake/maps.bin','offset':'0x2032b8','sha256':hashlib.sha256(maps).hexdigest(),'count':7,'visible_types':6,'width':12,'height':16,'cell_px':20,'nonzero_counts':[sum(x!=0 for x in maps[i*192:(i+1)*192]) for i in range(7)]},'images':records,'code_evidence':{'map_dispatch':'0x88690bbc..0x88690c10: seven 192-byte maps; visible picker exposes six.','grid':'0x88690988..0x886909ec: x<12, y<16, coordinates *20.','initial_body':'0x8869023e..0x886902a6: head (8,6), two segments behind, direction 4 (right).','wrap':'0x886906e0..0x88690758: x wraps 0..11, y wraps 0..15.','collision':'0x8869044c..0x886904e0: any nonzero map cell or any existing body node collides, including the tail.','food':'0x886902ce..0x886902fa: x modulo 12, y modulo 14 +2.','score':'0x88690794..0x8869079a adds state+0x2c per normal fruit; bonus at positive scores divisible by six.','level':'0x88099f6e..0x88099f7c: clamps 1..8; left/right preview, Back cancels, confirm stores.','level_geometry':'0x88099e04..0x88099e74: eight bars x=24+i*24, y=160..200; RGB565 0x07e0/0xad55.','score_digits':'0x886907f6..0x8869087a: sprite 0x401f0008, least significant digit at x105/y3, following digits move LEFT by16, glyphs16x23.','background':'0x88690c60..0x88690c88: sprite 0x401f0002.','body_head':'0x88690166..0x8869023c: body 0x401f0003, head 0x401f0006.'}}
result['initial_data']={'path':str(init_path.relative_to(ROOT)), 'sha256':hashlib.sha256(init).hexdigest(),
 'ram_base':'0x81c20000','state':'0x81c3bea8','food_increment_offset':'0x1bed4','food_increment':1,
 'timer_ram':'0x81c3bedc','timer_file_offset':'0x1bedc','tick_ms_by_level':list(struct.unpack('<9H',speeds))[1:],
 'timer_asset_sha256':hashlib.sha256(speeds).hexdigest(), 'provenance':'Previously extracted initialized kernel data, hash checked; not a live RAM read.'}
result['code_evidence']['turns']='0x88099b88..0x88099bea rejects either direction on current axis before writing pending turn. Same-axis key does not cancel a queued perpendicular turn.'
result['code_evidence']['timer']='0x88690a6e..0x88690a82 indexes signed halfwords directly by level (1..8) and passes interval to timer helper.'
result['code_evidence']['bonus']='0x886907a2..0x886907e8: positive score multiple of six spawns bonus; countdown *3 on collection without growth. 0x8869087e..0x886908fe counts rendered ticks modulo floor(1000/interval), clears one interval after reaching zero.'
result['code_evidence']['game_over']='0x88690a36..0x88690aec: states 2/3 redraw board then draw (30,70,180,50), fill 0x0127, text 0xf800 at font20. Literal 0x690cd8 is 0x401c002c (游戏结束). State4 draws 暂停 via 0x40000b25.'
result['limits']=['PRNG sequence is a local replacement; no claim of native food sequence.',
 'Timer table and call arguments are verified; wall-clock scheduler timing is not measured.',
 'Sound output, transient death/win effects and complete frame parity still require work; static game-over overlay verified.',
 'Seventh map is compiled but not exposed by the six-item native picker; not enabled in replica.']
result['reviewed_code_ranges']=[]
for lo,hi in [(0x690034,0x6900ec),(0x690166,0x6902a6),(0x6902ce,0x6902fc),(0x69044c,0x6904e2),(0x69062c,0x6908fe),(0x690988,0x6909ee),(0x690a36,0x690aec),(0x690bbc,0x690cfa),(0x99b88,0x99bec),(0x99e04,0x99f92)]:
 raw=b[lo:hi]
 result['reviewed_code_ranges'].append({'start':hex(0x88000000+lo),'end_exclusive':hex(0x88000000+hi),'sha256':hashlib.sha256(raw).hexdigest(),'hex':raw.hex()})
if args.disassemble:
 from firmware_disassembly import disassemble_arm
 listing=disassemble_arm(b,0x88000000,[(int(r['start'],16),int(r['end_exclusive'],16)) for r in result['reviewed_code_ranges']])
 (ROOT/'docs/evidence/snake-thumb.txt').write_text(listing)
# Decoder +hwdiv enables the SDIV instruction display; it is not a CPU identification.
(ROOT/'docs/evidence/snake-recovery.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print('Verified source; recovered seven maps and eleven sprite bindings.')
