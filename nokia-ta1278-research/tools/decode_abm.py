#!/usr/bin/env python3
"""Bounded ABM RGB565 decoder, adapted from Crawlerop/sprd_abm.py (MIT).
Pure Python PNG output. Parses extracted resources; never executes firmware.
"""
from pathlib import Path
import struct,zlib,json,hashlib,collections
ROOT=Path(__file__).resolve().parents[1]
RES=ROOT/'analysis/extracted/resources';OUT=ROOT/'analysis/extracted/png';OUT.mkdir(exist_ok=True)
def png(w,h,rgba):
    def chunk(t,b):return struct.pack('>I',len(b))+t+b+struct.pack('>I',zlib.crc32(t+b)&0xffffffff)
    raw=b''.join(b'\0'+rgba[y*w*4:(y+1)*w*4] for y in range(h))
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')
def decode(b):
    assert b[:3]==b'ABM' and b[3] in (0x50,0x52)
    typ=b[3];w,h,nc,na,top,left,bottom,right=struct.unpack_from('>HHHHBBBB',b,4)
    assert 0<w<=1024 and 0<h<=1024 and 1<=nc+na<=65536
    aw,ah=w-left-right,h-top-bottom;assert aw>0 and ah>0
    count=nc+na;p=16
    assert p+2*(count-1)+na<=len(b)
    colors=[0xf81f]+list(struct.unpack_from('>'+str(count-1)+'H',b,p));p+=2*(count-1)
    alpha=[0]+[255]*(nc-1)+list(b[p:p+na]);p+=na+(na&1)
    bits=(count-1).bit_length();runs=None
    if typ==0x52:
        n=struct.unpack_from('<I',b,p)[0];p+=4;runs=b[p:p+n];assert len(runs)==n;p+=n
    pos=0;word=0;available=0;pixels=[]
    def bitread():
        nonlocal p,word,available
        result=0
        for i in range(bits):
            if not available:
                remaining=min(2,len(b)-p)
                assert remaining>0
                # ABMR streams may end with one real byte. Consume its eight
                # bits only; do not invent a padded byte or read past EOF.
                assert remaining==2 or typ==0x52
                word=int.from_bytes(b[p:p+remaining],'big' if typ==0x50 else 'little')
                p+=remaining;available=remaining*8
            result|=(word&1)<<i;word>>=1;available-=1
        assert result<count;return result
    while len(pixels)<aw*ah:
        if runs is None:n,repeat=1,0
        else:
            assert pos+2<=len(runs);n,repeat=runs[pos:pos+2];pos+=2
        pixels.extend(bitread() for _ in range(n))
        if repeat:pixels.extend([bitread()]*repeat)
        assert len(pixels)<=aw*ah
    rgba=bytearray(w*h*4)
    for i,k in enumerate(pixels):
        c=colors[k];r=(c>>11)&31;g=(c>>5)&63;bl=c&31
        dst=((i//aw+top)*w+i%aw+left)*4
        rgba[dst:dst+4]=bytes((r*255//31,g*255//63,bl*255//31,alpha[k]))
    return w,h,bytes(rgba)
def main():
    index=json.loads((ROOT/'analysis/system/resource-index.json').read_text());report=[];atlas=[]
    for row in index:
        if row['kind'] not in ('icon','logo'):continue
        b=(RES/row['file']).read_bytes()
        if b[:4]==b'sabm' and len(b)>=12 and b[6]:b=b[12:]
        if b[:3]!=b'ABM':continue
        item={'source':row['file'],'offset':row['offset']}
        try:
            w,h,rgba=decode(b);data=png(w,h,rgba);name=Path(row['file']).stem+'.png'
            (OUT/name).write_bytes(data);item.update(status='decoded',file=name,width=w,height=h,sha256=hashlib.sha256(data).hexdigest())
            if 30<=w<=80 and 30<=h<=80 and len(atlas)<80:atlas.append((w,h,rgba))
        except Exception as e:item.update(status='failed',error=repr(e))
        report.append(item)
    canvas=bytearray(bytes((235,235,235,255))*800*800)
    for i,(w,h,p) in enumerate(atlas):
        x=(i%10)*80+(80-w)//2;y=(i//10)*80+(80-h)//2
        for yy in range(h):
            for xx in range(w):
                src=(yy*w+xx)*4;dst=((y+yy)*800+x+xx)*4;a=p[src+3]
                for c in range(3):canvas[dst+c]=(p[src+c]*a+235*(255-a))//255
    (ROOT/'analysis/system/icon-contact-sheet.png').write_bytes(png(800,800,canvas))
    (ROOT/'analysis/system/abm-decode.json').write_text(json.dumps(report,indent=2)+'\n')
    print(dict(collections.Counter(x['status'] for x in report)))
if __name__=='__main__':main()
