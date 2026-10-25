#!/usr/bin/env python3
"""Extract all DATA, COLOR and RECT resources, plus bounded T9 reconstruction.
Only immutable built-in system resources are used; no personal dictionary/NV.
"""
from pathlib import Path
import hashlib,json,struct
from PIL import Image,ImageDraw
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui';D=O/'data-resources';T=O/'t9'
def sha(b):return hashlib.sha256(b).hexdigest()
def save(n,x):(O/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def main():
 m=(R/'analysis/work/system-20261025T054301+0800/mmi.bin').read_bytes();u=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
 assert sha(m)=='14c573027f375ea6b5f8ead2c6c432f2773f373faaa3c8a33261d0de9737886c'
 sections=json.loads((R/'analysis/system/resource-sections.json').read_text());D.mkdir(exist_ok=True);T.mkdir(exist_ok=True)
 syms=json.loads((O/'resource-symbols.json').read_text())['symbols'];names={(x['kind'],x['group'],x['index']):x['name'] for x in syms}
 data=[];colors=[];rects=[];payloads={}
 for s in sections:
  if s['type'] not in (6,7,8):continue
  tres=struct.unpack_from('<I',m,16+s['tres']*8)[0];group=tres+struct.unpack_from('<I',m,tres+20+4*s['group'])[0]
  for i in range(s['count']):
   rid=0x40000001+(s['group']<<16)+i
   if s['type']==6:
    at=s['offset']+12*i;flag,n,p=struct.unpack_from('<III',m,at);p+=group
    assert s['offset']+s['count']*12<=p<=p+n<=s['offset']+s['size']
    raw=m[p:p+n];f=f'data-{s["group"]:02d}-{i:04d}-{p:08x}.bin';(D/f).write_bytes(raw)
    row={'kind':'data','resource_id':hex(rid),'group':s['group'],'index':i,'descriptor_offset':hex(at),'offset':hex(p),'size':n,'flags':flag,'sha256':sha(raw),'file':'data-resources/'+f,'zero_placeholder':raw==b'\0'}
    if raw[:4] in (b'OTAC',b'BKMK',b'MTVA'):row['magic']=raw[:4].decode()
    data.append(row);payloads[i]=raw
   elif s['type']==7:
    at=s['offset']+8*i;c,valid=struct.unpack_from('<II',m,at);assert c<65536 and valid in (0,1)
    rgb=(((c>>11)&31)*255//31,((c>>5)&63)*255//63,(c&31)*255//31)
    colors.append({'resource_id':hex(rid),'index':i,'offset':hex(at),'name':names.get((6,s['group'],i)),'rgb565':hex(c),'rgb888_preview':rgb,'valid':bool(valid),'raw_hex':m[at:at+8].hex()})
   else:
    at=s['offset']+12*i;v=struct.unpack_from('<4hI',m,at);assert v[4] in (0,1)
    rects.append({'resource_id':hex(rid),'offset':hex(at),'left':v[0],'top':v[1],'right':v[2],'bottom':v[3],'valid':bool(v[4]),'raw_hex':m[at:at+12].hex()})
 save('data-resource-index.json',{'correction':'Historical resource-index.json kind wap_presets is the complete DATA class, not solely WAP presets; unknown7=COLOR and unknown8=RECT. Original slice filenames retained for compatibility.','entries':data})
 save('color-resources.json',colors);save('rectangle-resources.json',rects)
 canvas=Image.new('RGB',(600,100*((len(colors)+4)//5)),'white');draw=ImageDraw.Draw(canvas)
 for i,r in enumerate(colors):
  x=i%5*120;y=i//5*100;draw.rectangle((x,y,x+112,y+60),fill=tuple(r['rgb888_preview']));draw.text((x,y+63),str(i)+' '+r['rgb565'],fill='black');draw.text((x,y+78),(r['name'] or '')[10:25],fill='black')
 canvas.save(O/'color-contact-sheet.png')
 aw=[]
 for n,ids in enumerate(((3,4,5),(6,7,8))):
  raw=b''.join(payloads[i] for i in ids);assert len(raw)==36651 and raw.startswith(b'Copyright')
  lang=struct.unpack_from('<H',raw,0x36)[0];assert lang==0x109
  f=f't9-aw-0109-variant-{n}.ldb';(T/f).write_bytes(raw)
  aw.append({'file':'t9/'+f,'size':len(raw),'sha256':sha(raw),'data_indices':ids,'language_id':hex(lang),'language':'English (public T9 language ID)','status':'native resource chunks concatenated; internal trie and candidate ranking not decoded'})
 init=(R/'analysis/extracted/kernel/init_81c20000.bin').read_bytes();ch=[]
 # Simplified layout is independently verified by the native load/store chain:
 # 8854c834..8854ca32 loads DATA indices 373..387 and assigns 15 block slots.
 for label,addr,first in [('simplified',0x81c6e3f8,373),('traditional',0x81c6e5c0,431),('traditional-alt',0x81c6e788,388)]:
  at=addr-0x81c20000;header=init[at:at+192];sizes=struct.unpack_from('<66H',header,60)
  assert header[32:36]==b'\x01\x08\x34\x12'
  blocks=[(i,n) for i,n in enumerate(sizes) if n];matches=[]
  # Native branches actually populate 15 slots for simplified, 21 for traditional.
  slots=([0,8,9,17,25,37,41,42,43,46,47,48,59,60,64] if label=='simplified' else [0,1,8,9,10,17,18,25,26,27,37,41,42,43,46,47,48,49,59,60,64])
  for j,k in enumerate(slots):matches.append({'slot':k,'data_index':first+j,'header_size':sizes[k],'resource_size':len(payloads[first+j]),'size_matches':sizes[k]==len(payloads[first+j])})
  name='t9-'+label+'-header.bin';(T/name).write_bytes(header)
  row={'variant':label,'header_ram_address':hex(addr),'header_init_offset':hex(at),'header_file':'t9/'+name,'header_sha256':sha(header),'language_id':hex(struct.unpack_from('<H',header,36)[0]),'module_flags':hex(header[50]),'modules':[name for flag,name in [(32,'stroke'),(16,'component'),(8,'pinyin'),(4,'bopomofo'),(2,'context40'),(1,'context20')] if header[50]&flag],'blocks':matches}
  if label=='simplified':
   assert all(x['size_matches'] for x in matches) and len(blocks)==len(slots)
   out=bytearray(header+bytes(264))
   for x in matches:struct.pack_into('<I',out,192+4*x['slot'],len(out));out.extend(payloads[x['data_index']])
   name='t9-simplified-0204-reconstructed.ldb';(T/name).write_bytes(out)
   row.update(file='t9/'+name,size=len(out),sha256=sha(out),status='external-offset container rebuilt; all 15 block lengths match. Candidate algorithm/checksum not validated by native engine.')
  else:row['status']='header and resource chunks preserved; header size disagreement prevents claiming a faithful complete container'
  ch.append(row)
 save('t9-resource-analysis.json',{'engine_strings':[{'offset':hex(p),'text':u[p:u.index(0,p)].decode()} for p in [0xcc3d4,0xd7704,0xe5030]],'alphabetic':aw,'chinese':ch,'personal_dictionary_accessed':False,'native_engine_executed':False,'init_sha256':sha(init)})
 save('data-resource-validation.json',{'status':'passed','data_entries':len(data),'nonplaceholder_data':sum(not x['zero_placeholder'] for x in data),'colors':len(colors),'rectangles':len(rects),'t9_aw_containers':2,'t9_chinese_container':1,'chinese_variant_mismatches':{x['variant']:sum(not b['size_matches'] for b in x['blocks']) for x in ch}})
 print(len(data),'DATA,',len(colors),'COLOR,',len(rects),'RECT; T9 built-in resources exported')
if __name__=='__main__':main()
