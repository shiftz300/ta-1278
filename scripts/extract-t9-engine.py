#!/usr/bin/env python3
"""Extract only T9 ARM code/constants and public LDBs, with fixed hash checks."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
u=ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
b=u.read_bytes();assert hashlib.sha256(b).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
k=(u.parent/'kernel.bin').read_bytes()
assert hashlib.sha256(k).hexdigest()=='b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303'
# The only kernel helpers exposed by the runtime: UDIV/SDIV with MLS remainder.
assert b[0x1009cc:0x1009d4].hex()=='04f01fe5c0598f81'
assert b[0x100844:0x10084c].hex()=='04f01fe5405a8f81'
assert k[0x818f59c8-0x80e65e00:0x818f59d8-0x80e65e00].hex()=='10f132e7920161e00200a0e11eff2fe1'
assert k[0x818f5a48-0x80e65e00:0x818f5a58-0x80e65e00].hex()=='10f112e7920161e00200a0e11eff2fe1'
out=ROOT/'assets/input/t9';out.mkdir(exist_ok=True)
files={'engine.arm':b[0xcc000:0xe9000], 'character-tables.bin':b[0x4ac300:0x4acc00]}
for name,src,sha in [('english.ldb','t9-aw-0109-variant-0.ldb','7c85dc8eb53b275475d3d13e82198eae8c57c9e64fe91a2d8adffa9ba72176a7'),('chinese.ldb','t9-simplified-0204-reconstructed.ldb','ea0435d16b27147db877284d099444a0300aece87a6670fed9ab9e780408cee9')]:
 data=(ROOT/'nokia-ta1278-research/analysis/ui/t9'/src).read_bytes();assert hashlib.sha256(data).hexdigest()==sha;files[name]=data
for name,data in files.items():(out/name).write_bytes(data)
(out/'manifest.json').write_text(json.dumps({'source':str(u.relative_to(ROOT)),'engine_start':'0x880cc000','engine_end':'0x880e9000','readonly_character_tables':['0x884ac300','0x884acc00'],'kernel_helpers':{'0x881009cc':'unsigned divmod','0x88100844':'signed divmod'},'runtime':'bounded ARM instruction interpretation; no native loading or device access','files':{n:{'bytes':len(d),'sha256':hashlib.sha256(d).hexdigest()} for n,d in files.items()}},indent=2)+'\n')
print('T9 assets:',sum(map(len,files.values())),'bytes')
