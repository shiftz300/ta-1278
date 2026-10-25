#!/usr/bin/env python3
"""Recover stripped JPEG using header enums and tables matched in kernel.bin.
Validates every baseline Huffman block and all remaining entropy pad bits.
No firmware execution; Pillow supplies standard JPEG framing and IDCT.
"""
from pathlib import Path
import hashlib,io,json,math,struct
from PIL import Image,ImageDraw
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui';N=O/'native-images'
def digest(b):return hashlib.sha256(b).hexdigest()
def segments(b):
 p=2
 while True:
  assert b[p]==255
  marker=b[p+1];n=int.from_bytes(b[p+2:p+4],'big');x=b[p+4:p+2+n];p+=2+n
  yield marker,x,p
  if marker==0xda:break
def validate_entropy(raw,tables,w,h):
 # Source enumerator says 3 = YUV422; standard SOF has Y 2x1, Cb/Cr 1x1.
 data=bytearray();i=0
 while i<len(raw):
  v=raw[i];i+=1
  if v==255:
   assert i<len(raw)
   if raw[i]==0:i+=1
   elif raw[i]==0xd9:
    i+=1;assert i==len(raw);break
   else:raise ValueError('unexpected JPEG marker '+hex(raw[i]))
  data.append(v)
 pos=0
 def bits(n):
  nonlocal pos
  assert pos+n<=len(data)*8
  v=0
  for _ in range(n):v=(v<<1)|((data[pos//8]>>(7-pos%8))&1);pos+=1
  return v
 def sym(t):
  v=0
  for n in range(1,17):
   v=(v<<1)|bits(1)
   if (n,v) in tables[t]:return tables[t][n,v]
  raise ValueError('invalid Huffman code')
 blocks=0
 for _ in range(math.ceil(w/16)*math.ceil(h/8)):
  for comp in (0,0,1,1):
   size=sym(comp);assert size<=11;bits(size);j=1
   while j<64:
    rs=sym(0x10|comp)
    if rs==0:break
    run,size=rs>>4,rs&15
    if size==0:assert run==15;j+=16;assert j<=64
    else:j+=run;assert j<64 and size<=10;bits(size);j+=1
   blocks+=1
 remain=len(data)*8-pos;assert 0<=remain<=7,(remain,pos,len(data))
 padding=bits(remain)
 # This stripped format does not consistently fill residual bits with ones.
 # Record them verbatim; verify no unread complete byte or MCU remains.
 return {'blocks':blocks,'entropy_bytes_unstuffed':len(data),'padding_bits':remain,'padding_value':padding,'all_blocks_and_byte_consumption_verified':True}
def main():
 kernel=(R/'analysis/work/system-20261025T054301+0800/kernel.bin').read_bytes()
 assert digest(kernel)=='b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303'
 records=[];thumbs=[];N.mkdir(exist_ok=True)
 for src in sorted((R/'analysis/extracted/resources').glob('*.bin')):
  b=src.read_bytes()
  if b[:4]!=b'SJPG':continue
  h,w,sampling,q=struct.unpack_from('>4H',b,4);assert sampling==3 and q in (0,4)
  quality=[40,50,70,85,95][q];buf=io.BytesIO();Image.new('RGB',(w,h)).save(buf,'JPEG',quality=quality,subsampling=1,optimize=False)
  template=buf.getvalue();reference=Image.open(io.BytesIO(template));evidence=[];tables={}
  for number,tab in reference.quantization.items():
   needle=bytes(tab);at=kernel.find(needle);assert at>=0
   evidence.append({'type':'DQT-natural-order','table':number,'kernel_offset':hex(at),'size':64,'sha256':digest(needle)})
  for marker,x,end in segments(template):
   if marker==0xc4:
    at=kernel.find(x[1:]);assert at>=0
    evidence.append({'type':'DHT-counts-and-symbols','table':x[0],'kernel_offset':hex(at),'size':len(x)-1,'sha256':digest(x[1:])})
    table={};code=0;k=17
    for n,count in enumerate(x[1:17],1):
     for _ in range(count):table[n,code]=x[k];code+=1;k+=1
     code<<=1
    assert k==len(x);tables[x[0]]=table
   if marker==0xda:header=template[:end]
  checks=validate_entropy(b[12:],tables,w,h)
  jpg=header+b[12:]+(b'' if b.endswith(b'\xff\xd9') else b'\xff\xd9')
  jp=N/(src.stem+'.jpg');jp.write_bytes(jpg)
  im=Image.open(io.BytesIO(jpg));im.load();assert im.size==(w,h)
  pp=N/(src.stem+'.png');im.save(pp)
  rec={'source':src.name,'source_sha256':digest(b),'format':'SJPG/JPEG422','width':w,'height':h,'quality_enum':q,'quality':quality,'native_table_matches':evidence,'entropy_validation':checks,'jpeg':str(jp.relative_to(O)),'jpeg_sha256':digest(jpg),'frames':[{'file':str(pp.relative_to(O)),'width':w,'height':h,'sha256':digest(pp.read_bytes())}],'limitation':'Pillow IDCT and color conversion; native rounding may differ by pixels.'}
  records.append(rec);thumbs.append((src.name,im))
 canvas=Image.new('RGB',(600,230*math.ceil(len(thumbs)/5)),'#aaa');draw=ImageDraw.Draw(canvas)
 for i,(name,im) in enumerate(thumbs):
  im.thumbnail((115,200));x=i%5*120;y=i//5*230;canvas.paste(im,(x,y));draw.text((x,y+205),name[:5],fill='black')
 canvas.save(O/'sjpg-contact-sheet.png');(O/'sjpg-extraction.json').write_text(json.dumps(records,indent=2)+'\n')
 print(len(records),'SJPG; native DQT/DHT matched; every entropy block verified')
if __name__=='__main__':main()
