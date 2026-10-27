"""Bounded T9 isolated-routine probe; synthetic buffers only."""
from pathlib import Path
import hashlib
from arm import Arm
ROOT=Path(__file__).resolve().parents[2]
rom=(ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin').read_bytes()
assert hashlib.sha256(rom).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
a=Arm(rom);field=0x20001000;text=0x20010000;aux=0x20012000
ldb=(ROOT/'nokia-ta1278-research/analysis/ui/t9/t9-aw-0109-variant-0.ldb').read_bytes();db=0x20040000
# The harness copies public LDB data, never a handset user dictionary.
a.ram[db-a.rb:db-a.rb+len(ldb)]=ldb
requests=[]
def request(a):
    requests.append((a.r[0],a.r[1],a.read(a.r[1],4)));a.r[0]=0

def read_ldb(a):
    off=a.r[1];assert off<len(ldb)
    block=off//0x3c00*0x3c00
    a.write(a.r[2],db+block);a.write(a.r[3],block)
    a.r[0]=min(0x3c00,len(ldb)-block)
def divmod_u32(a):
    assert a.r[1]!=0
    a.r[0],a.r[1]=divmod(a.r[0],a.r[1])
# Verified ARM veneer -> kernel UDIV + MLS, no other kernel routine is allowed.
k=(ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/kernel.bin').read_bytes()
assert hashlib.sha256(k).hexdigest()=='b871d5a004c9251f3bb7cc58ea637d486f05d680efd3c39be83c38051f11e303'
assert k[0x818f59c8-0x80e65e00:0x818f59d8-0x80e65e00].hex()=='10f132e7920161e00200a0e11eff2fe1'
def divmod_i32(a):
    n=a.r[0] if a.r[0]<0x80000000 else a.r[0]-0x100000000
    d=a.r[1] if a.r[1]<0x80000000 else a.r[1]-0x100000000
    assert d
    q=abs(n)//abs(d)*(-1 if (n<0)!=(d<0) else 1)
    a.r[0],a.r[1]=q&0xffffffff,(n-q*d)&0xffffffff
assert k[0x818f5a48-0x80e65e00:0x818f5a58-0x80e65e00].hex()=='10f112e7920161e00200a0e11eff2fe1'
a.hooks[0x88100844]=divmod_i32
a.hooks[0x881009cc]=divmod_u32
a.hooks[0x10000010]=request;a.hooks[0x10000020]=read_ldb
if __name__=='__main__':
    print('init',a.call(0x880e4e44,field,text,aux,2048,0,0x10000010,0x10000020),a.steps)
    print('lang',a.call(0x880e54a8,field,0x109,field,0),a.steps)
    for f in [0x880e56c8,0x880e55b8]:print('flags',a.call(f,field),a.steps)
    for key in [4,6,6,3]:
        print('key',key,a.call(0x880cf39c,field,key),a.steps)
        print('text',bytes(a.ram[text-a.rb:text-a.rb+80]).decode('utf16',errors='replace'))
    print('requests',requests[:20])
