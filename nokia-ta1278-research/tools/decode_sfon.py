#!/usr/bin/env python3
"""Static SFON v3 outline decoder, reconstructed from this device's Thumb code.
No firmware code is executed. Bounds/point counts/contour ends are checked.
"""
from pathlib import Path
import struct,json,hashlib,collections
ROOT=Path(__file__).resolve().parents[1]; OUT=ROOT/'analysis/ui/fonts'
def u16(b,p): return struct.unpack_from('>H',b,p)[0]
def u32(b,p): return struct.unpack_from('>I',b,p)[0]
class Reader:
 def __init__(self,b):self.b=b;self.p=0
 def read(self,n):
  assert self.p+n<=len(self.b),('overrun',self.p,n,len(self.b))
  q=self.b[self.p:self.p+n];self.p+=n;return q
 def byte(self):return self.read(1)[0]
 def word(self):return int.from_bytes(self.read(2),'big')
 def signed(self,n):return int.from_bytes(self.read(n),'big',signed=True)
def decode(data,default_advance):
 r=Reader(data);flags=r.byte();advance=r.byte() if flags&8 else default_advance
 result={'flags':flags,'advance_32':advance}
 if flags&1:
  comps=[]
  for _ in range(r.word()):
   fl=r.word();gid=r.word();a=r.signed(2 if fl&1 else 1);b=r.signed(2 if fl&1 else 1)
   matrix=[1,0,0,1]
   if fl&8:matrix[0]=matrix[3]=r.signed(2)/16384
   elif fl&64:matrix[0]=r.signed(2)/16384;matrix[3]=r.signed(2)/16384
   elif fl&128:matrix=[r.signed(2)/16384 for _ in range(4)]
   comps.append({'glyph_id':gid,'flags':fl,'args':[a,b],'matrix':matrix})
  result['components']=comps
 else:
  pack=r.byte();n=r.byte() if pack&1 else r.word();nc=r.byte() if pack&2 else r.word(); fs=[]
  while len(fs)<n:
   f=r.byte();fs.extend([f]*(1+(r.byte() if f&8 else 0)))
  assert len(fs)==n
  axes=[]
  for short,same in [(2,16),(4,32)]:
   acc=0;coords=[]
   for f in fs:
    delta=(r.byte()*(1 if f&same else -1)) if f&short else (0 if f&same else r.signed(2))
    acc+=delta;coords.append(acc)
   axes.append(coords)
  ends=[r.byte() if pack&4 else r.word() for _ in range(nc)]
  assert (not n and not nc) or (ends and ends[-1]==n-1 and ends==sorted(set(ends))), (n,nc,ends)
  result.update(points=list(zip(*axes)),on_curve=[bool(f&1) for f in fs],cubic=[bool(f&64) for f in fs],contour_ends=ends)
 assert r.p==len(data),('trailing bytes',r.p,len(data))
 return result

def parse(p):
 b=p.read_bytes();s=u32(b,40);ng=u16(b,s+26);ns=u16(b,s+32);base=s+34
 ends=[u16(b,base+2*i) for i in range(ns)];starts=[u16(b,base+2*ns+2*i) for i in range(ns)];indices=[u16(b,base+4*ns+2*i) for i in range(ns)]
 gids={gid:idx+gid-a for a,z,idx in zip(starts,ends,indices) for gid in range(a,z+1)}
 table=base+6*ns; offsets=[s+32+u32(b,table+i*4) for i in range(ng)]; unique=sorted(set(offsets));last={o:(unique[i+1] if i+1<len(unique) else len(b)) for i,o in enumerate(unique)}
 glyphs={};errors=[]
 for gid,idx in gids.items():
  try:
   assert idx<ng;at=offsets[idx];stop=last[at];assert table+ng*4<=at<=stop<=len(b)
   g=decode(b[at:stop],b[s+16]);g.update(glyph_id=gid,index=idx,offset=at,size=stop-at);glyphs[gid]=g
  except (AssertionError,IndexError,struct.error) as e:errors.append({'glyph_id':gid,'index':idx,'error':str(e)})
 cmap=json.loads(p.with_name(p.stem+'-cmap.json').read_text());missing=[x for x in cmap if x['glyph_id'] not in glyphs]
 result={'source':p.name,'sha256':hashlib.sha256(b).hexdigest(),'units_per_em':u16(b,18),'strike_offset':s,'strike_scale_128':b[s],'declared_glyph_count':ng,'segment_count':ns,'glyph_count':len(glyphs),'mapped_codepoints':len(cmap),'missing_codepoints':missing,'errors':errors,'glyphs':list(glyphs.values())}
 (OUT/(p.stem+'-outlines.json')).write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
 print(p.name,'decoded',len(glyphs),'errors',len(errors),'missing',len(missing),'composite',sum('components'in g for g in glyphs.values()),'cubic',sum(any(g.get('cubic',[])) for g in glyphs.values()))
 if errors:print(errors[:3])
 return result,cmap
if __name__=='__main__':
 results=[parse(p)[0] for p in sorted(OUT.glob('*.sfon'))]
 metadata=json.loads((OUT.parent/'fonts.json').read_text())
 for row,d in zip(metadata,results):
  row.update(body_decoded=not d['errors'],glyph_count=d['glyph_count'],missing_cmap_glyphs=len(d['missing_codepoints']),outline_file='fonts/'+Path(d['source']).stem+'-outlines.json')
 (OUT.parent/'fonts.json').write_text(json.dumps(metadata,indent=2)+'\n')
