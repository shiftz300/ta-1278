#!/usr/bin/env python3
"""Offline BMP 0x1003 RLE, raw SABM, and SMOV frame extraction.
RLE grammar is data-derived and checked against every row and byte.
SABM color order is explicitly provisional BGRA; dimensions/bytes are exact.
"""
from pathlib import Path
import struct,json,hashlib
from PIL import Image,ImageDraw
from decode_abm import decode,png
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui';N=O/'native-images';N.mkdir(exist_ok=True)
def rle(b,p,w,h):
 rows=[]
 for y in range(h):
  row=[]
  while True:
   assert p+2<=len(b);literal,count=b[p:p+2];p+=2
   if not(literal or count):break
   assert p+2*literal<=len(b)
   row.extend(struct.unpack_from('<'+str(literal)+'H',b,p));p+=2*literal
   if count:
    assert p+2<=len(b);color=int.from_bytes(b[p:p+2],'little');p+=2;row.extend([color]*count)
   assert len(row)<=w
  assert len(row)==w,(y,len(row),w)
  rows.append(row)
 return rows,p
def rgba565(rows):
 return bytes(v for row in rows for c in row for v in (((c>>11)&31)*255//31,((c>>5)&63)*255//63,(c&31)*255//31,255))
def main():
 entries=[];thumbs=[]
 for src in sorted((R/'analysis/extracted/resources').glob('*.bin')):
  b=src.read_bytes();rec={'source':src.name};frames=[];kind=None
  if b[:2]==b'BM' and len(b)>66 and struct.unpack_from('<I',b,30)[0]==0x1003:
   w,h=struct.unpack_from('<ii',b,18);p=struct.unpack_from('<I',b,10)[0];assert struct.unpack_from('<III',b,54)==(0xf800,0x7e0,0x1f)
   rows,end=rle(b,p,w,abs(h));assert end==len(b)
   if h>0:rows.reverse()
   frames=[(w,abs(h),rgba565(rows))];kind='BMP-0x1003';rec['all_rows_and_bytes_verified']=True
  elif b[:4]==b'sabm' and b[6]==0:
   h,w=struct.unpack_from('>HH',b,8);assert len(b)==12+w*h*4
   frames=[(w,h,bytes(v for i in range(12,len(b),4) for v in (b[i+2],b[i+1],b[i],b[i+3])))];kind='SABM-raw32';rec['channel_order']='BGRA provisional; native renderer not yet traced';rec['header_flags']=b[4:8].hex()
  elif b[:2]==b'VM':
   w,h=struct.unpack_from('<ii',b,18);n=struct.unpack_from('<I',b,66)[0];assert n<=256
   pairs=[struct.unpack_from('<II',b,70+8*i) for i in range(n)]
   for size,off in pairs:
    rows,end=rle(b,off,w,abs(h));assert end==off+size<=len(b)
    if h>0:rows.reverse()
    frames.append((w,abs(h),rgba565(rows)))
   kind='VM/RLE565';rec['frame_ranges']=[{'offset':o,'size':n} for n,o in pairs];rec['timing']='unresolved; directory order preserved'
  elif b[:4]==b'SMOV':
   n=struct.unpack_from('>H',b,10)[0];offs=list(struct.unpack_from('>'+str(n)+'I',b,16));assert offs==sorted(set(offs)) and offs[0]==16+4*n
   for i,p in enumerate(offs):frames.append(decode(b[p:offs[i+1] if i+1<n else len(b)]))
   kind='SMOV/ABMP';rec['frame_offsets']=offs;rec['interval_raw']=struct.unpack_from('>H',b,8)[0];rec['interval_evidence']='Public anim_def.h SMOV_ARGB_HEADER_T.wInterval at +8; units/native timer conversion unresolved';rec['timing']='interval field recovered; units unresolved'
  else:continue
  rec['format']=kind;rec['frames']=[]
  for i,(w,h,rgba) in enumerate(frames):
   name=src.stem+(f'-frame-{i:03d}' if len(frames)>1 else '')+'.png';data=png(w,h,rgba);(N/name).write_bytes(data);rec['frames'].append({'file':'native-images/'+name,'width':w,'height':h,'sha256':hashlib.sha256(data).hexdigest()})
   if i==0 and w>4 and h>4 and len(thumbs)<110:thumbs.append((name,Image.frombytes('RGBA',(w,h),rgba)))
  entries.append(rec)
 canvas=Image.new('RGB',(1000,((len(thumbs)+9)//10)*100),'#cccccc');d=ImageDraw.Draw(canvas)
 for i,(name,im) in enumerate(thumbs):
  im.thumbnail((90,78));x=(i%10)*100;y=(i//10)*100;canvas.paste(im,(x,y),im);d.text((x,y+80),name[:5],fill='black')
 canvas.save(O/'extra-images-contact-sheet.png');(O/'extra-image-extraction.json').write_text(json.dumps(entries,indent=2)+'\n')
 print(len(entries),'resources',sum(len(x['frames']) for x in entries),'frames')
if __name__=='__main__':main()
