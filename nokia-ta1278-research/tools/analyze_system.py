#!/usr/bin/env python3
"""Offline system-only format/resource index. Never interprets private FTL parts.
SRES/TRES layouts cross-checked against Crawlerop/chinamobiletools at 249b0156.
"""
from pathlib import Path
import hashlib,json,re,struct,collections
ROOT=Path(__file__).resolve().parents[1]
WORK=ROOT/'analysis/work/system-20261025T054301+0800'
OUT=ROOT/'analysis/system';OUT.mkdir(exist_ok=True)
EXT=ROOT/'analysis/extracted/resources';EXT.mkdir(parents=True,exist_ok=True)
def sha(b):return hashlib.sha256(b).hexdigest()
def writej(path,v):path.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def main():
    results=[]
    terms=re.compile(r'sms|message|t9|zi8|input.?method|ime_|opera|browser|wap|gui|font|keypad|softkey|lcd|menu|cursor|unicode|phonebook|mmi',re.I)
    for name in ('boot0','boot1','kernel','user','mmi'):
        b=(WORK/f'{name}.bin').read_bytes();row={'name':name,'size':len(b),'sha256':sha(b),'magic_hex':b[:16].hex()}
        if b[:8]==b'DHTB\1\0\0\0':
            size=struct.unpack_from('<I',b,0x30)[0]
            row.update(payload_offset=512,payload_size=size,trailer_bytes=len(b)-512-size)
            (ROOT/'analysis/extracted'/f'{name}.payload.bin').write_bytes(b[512:512+size])
        strings=[]
        for m in re.finditer(rb'[\x20-\x7e]{6,}',b):
            s=m.group().decode('ascii')
            if len(s)<=600 and terms.search(s):strings.append({'offset':m.start(),'offset_hex':hex(m.start()),'text':s})
        writej(OUT/f'{name}-topic-strings.json',strings);row['topic_string_count']=len(strings);results.append(row)
    b=(WORK/'mmi.bin').read_bytes();assert b[:8]==b'sres0001'
    res=[];sections=[]
    def take(off,n):
        if off<0 or n<0 or off+n>len(b):raise ValueError((off,n,len(b)))
        return b[off:off+n]
    def save(kind,off,n,meta):
        data=take(off,n)
        filename=f'{len(res):05d}-{kind}-{off:08x}.bin'
        (EXT/filename).write_bytes(data)
        res.append(dict(kind=kind,file=filename,offset=off,offset_hex=hex(off),size=n,sha256=sha(data),**meta))
    for ti in range(33):
        base,size=struct.unpack_from('<II',b,16+ti*8)
        if not size:continue
        t=take(base,size);assert t[:8]==b'tres0001'
        end=struct.unpack_from('<I',t,16)[0];assert end<=size
        for group in range(64):
            ptr=struct.unpack_from('<I',t,20+group*4)[0]
            if not 0<ptr<end:break
            for typ in range(8):
                off,count,length=struct.unpack_from('<III',t,ptr+typ*12)
                if not count:continue
                assert count<100000
                absolute=base+ptr+off
                sections.append({'tres':ti,'group':group,'type':typ+1,'offset':absolute,'count':count,'size':length})
                if typ in (0,1):
                    info,offsets=struct.unpack_from('<II',b,absolute)
                    for i in range(count):
                        w,h,format,n=struct.unpack_from('<HHII',b,base+ptr+info+i*12)
                        dataoff=base+ptr+struct.unpack_from('<I',b,base+ptr+offsets+i*4)[0]
                        meta={'tres':ti,'group':group,'index':i,'width':w,'height':h,'format':hex(format)}
                        if typ==1:meta.update(format=hex(format&65535),frames=format>>16)
                        if n:save('icon' if typ==0 else 'animation',dataoff,n,meta)
                elif typ==2:
                    for i in range(count):
                        format,n,dataoff=struct.unpack_from('<III',b,absolute+i*12)
                        if n:save('sound',base+ptr+dataoff,n,{'tres':ti,'group':group,'index':i,'format':format})
                else:
                    names={3:'strings',4:'font',5:'wap_presets',6:'unknown7',7:'unknown8'}
                    if length:save(names[typ],absolute,length,{'tres':ti,'group':group,'count':count})
    for i in range(b[13]):
        tag,off,n=struct.unpack_from('<4sII',b,284+i*12)
        save(tag.decode('ascii'),off,n,{})
    writej(OUT/'components.json',results);writej(OUT/'resource-sections.json',sections);writej(OUT/'resource-index.json',res)
    counts=dict(collections.Counter(x['kind'] for x in res));writej(OUT/'resource-summary.json',counts)
    keys=struct.unpack('<64H',(ROOT/'analysis/extracted/kernel/keymap.bin').read_bytes())
    names={1:'DIAL',4:'UP',5:'DOWN',6:'LEFT',7:'RIGHT',8:'LSOFT',9:'RSOFT',13:'CENTER',35:'HASH',42:'STAR',65535:'unused'}
    names.update({i:str(i-48) for i in range(48,58)})
    writej(OUT/'keymap.json',[{'row':i//8,'column':i%8,'code':hex(k),'name':names.get(k,'unknown'),'kernel_file_offset':0xb401cc+i*2} for i,k in enumerate(keys)])
    print(json.dumps({'components':results,'resource_counts':counts},indent=2))
if __name__=='__main__':main()
