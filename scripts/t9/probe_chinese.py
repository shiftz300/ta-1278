from probe import ROOT,rom,Arm,divmod_u32,divmod_i32,request
import json
meta=json.loads((ROOT/'nokia-ta1278-research/analysis/ui/t9-resource-analysis.json').read_text())['chinese'][0]
raw=(ROOT/'nokia-ta1278-research/analysis/ui/t9/t9-simplified-0204-reconstructed.ldb').read_bytes()
header=raw[:192];chunks={};at=456
for block in meta['blocks']:
    n=block['resource_size'];chunks[block['slot']]=raw[at:at+n];at+=n
assert at==len(raw)
a=Arm(rom);field=0x20001000;text=0x20010000;aux=0x20012000
pointers={};at=0x20040000
for slot,chunk in chunks.items():
    pointers[slot]=at;a.ram[at-a.rb:at-a.rb+len(chunk)]=chunk;at+=len(chunk)+8

def load_chinese(a):
    out=a.r[1]
    for i,b in enumerate(header):a.write(out+i,b,1)
    for i in range(66):a.write(out+192+i*4,pointers.get(i,0))
    a.r[0]=0

a.hooks={0x10000010:request,0x10000020:load_chinese,0x881009cc:divmod_u32,0x88100844:divmod_i32}
if __name__=='__main__':
 print('init',a.call(0x880d7718,field,text,aux,2048,0,0x10000010,4,0x10000020),a.steps)
 print('lang',a.call(0x880d7ba4,field,0x204,field,0),a.steps)
 print('pinyin',a.call(0x880d6e94,field),a.steps)
 for key in [6,4,4,2,6]:
    print('key',key,a.call(0x880cf39c,field,key),a.steps)
    print('output',bytes(a.ram[text-a.rb:text-a.rb+80]).decode('utf-16-le',errors='replace'))
    print('core',bytes(a.ram[field-a.rb+0x300:field-a.rb+0x360]).hex())
