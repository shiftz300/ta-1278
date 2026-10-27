"""Small fail-closed ARM-state research interpreter. No device or host syscalls.
Only supplied ROM/RAM and explicitly registered Python callbacks are reachable.
Not a general ARM emulator or a production prediction engine.
"""
from collections import deque, Counter
MASK=0xffffffff

def signed(x,bits=32):
    return (x & ((1<<(bits-1))-1))-(x & (1<<(bits-1)))
def ror(x,n):
    n%=32
    return ((x>>n)|(x<<(32-n)))&MASK if n else x&MASK

class Arm:
    def __init__(self,rom,base=0x88000000):
        self.rom=rom; self.base=base
        self.ram=bytearray(0x100000);self.rb=0x20000000
        self.r=[0]*16;self.n=self.z=self.c=self.v=0
        self.hooks={}; self.trace=deque(maxlen=32);self.visited=Counter()
        self.exec_ranges=[(0x880cc000,0x880e9000)]
    def read(self,a,n):
        for base,data in [(self.base,self.rom),(self.rb,self.ram)]:
            if base<=a and a+n<=base+len(data):return int.from_bytes(data[a-base:a-base+n],'little')
        raise ValueError(f'unmapped read {a:08x}+{n}')
    def write(self,a,v,n=4):
        if not self.rb<=a or a+n>self.rb+len(self.ram):raise ValueError(f'forbidden write {a:08x}+{n}')
        self.ram[a-self.rb:a-self.rb+n]=(v&((1<<(n*8))-1)).to_bytes(n,'little')
    def reg(self,n): return (self.pc+8)&MASK if n==15 else self.r[n]
    def put(self,n,v):
        v&=MASK
        if n==15:
            if v&1 and v not in self.hooks:raise ValueError(f'unsupported Thumb branch {v:08x}')
            self.next=v
        else:self.r[n]=v
    def cond(self,c):
        return [self.z,not self.z,self.c,not self.c,self.n,not self.n,self.v,not self.v,
          self.c and not self.z,not self.c or self.z,self.n==self.v,self.n!=self.v,
          not self.z and self.n==self.v,self.z or self.n!=self.v,True,False][c]
    def shift(self,x,k,n,register=False):
        if register and n==0:return x,self.c
        if k==0:
            if n==0:return x,self.c
            return ((x<<n)&MASK, (x>>(32-n))&1) if n<=32 else (0,0)
        if k==1:
            n=n or 32
            return (x>>n,(x>>(n-1))&1) if n<=32 else (0,0)
        if k==2:
            n=n or 32
            return ((signed(x)>>min(n,32))&MASK,(x>>(min(n,32)-1))&1)
        if n==0:return (self.c<<31)|(x>>1),x&1
        return ror(x,n), (ror(x,n)>>31)&1
    def operand(self,w):
        if w&(1<<25):
            rot=((w>>8)&15)*2;x=ror(w&255,rot)
            return x, ((x>>31)&1 if rot else self.c)
        rs=(w>>8)&15;byreg=bool(w&16)
        amount=(self.reg(rs)&255) if byreg else ((w>>7)&31)
        return self.shift(self.reg(w&15),(w>>5)&3,amount,byreg)
    def nz(self,x):self.n=(x>>31)&1;self.z=int((x&MASK)==0)
    def call(self,pc,*args,limit=3000000):
        self.r=[0]*16;self.r[13]=self.rb+len(self.ram)-0x1000;self.r[14]=0x10000000
        for i,v in enumerate(args):
            if i<4:self.r[i]=v
            else:self.write(self.r[13]+(i-4)*4,v)
        self.pc=pc;self.trace.clear();count=0
        try:
            while self.pc!=0x10000000:
                if count>=limit:raise ValueError('instruction budget exceeded')
                self.trace.append(self.pc);self.visited[self.pc]+=1;count+=1
                if self.pc in self.hooks:
                    self.hooks[self.pc](self);self.pc=self.r[14];continue
                if not any(a<=self.pc<b for a,b in self.exec_ranges):raise ValueError(f'forbidden execution {self.pc:08x}')
                w=self.read(self.pc,4);self.next=self.pc+4
                if self.cond(w>>28):self.step(w)
                self.pc=self.next
        except Exception as e:
            raise ValueError(f'{e}; pc={self.pc:08x} regs={[hex(v) for v in self.r]} trace={[hex(p) for p in self.trace]}') from e
        self.steps=count
        return self.r[0]
    def step(self,w):
        op=w&0x0fffffff;rn=(w>>16)&15;rd=(w>>12)&15
        if op&0x0ffffff0 in (0x012fff10,0x012fff30):
            target=self.reg(w&15)
            if op&0x20:self.r[14]=self.pc+4
            self.put(15,target);return
        if op>>25==5:
            if w&(1<<24):self.r[14]=self.pc+4
            self.put(15,self.pc+8+signed(w&0xffffff,24)*4);return
        if op&0x0f8000f0==0x00800090: # long multiply
            a=self.reg(w&15);b=self.reg((w>>8)&15)
            if w&(1<<22):a=signed(a);b=signed(b)
            res=a*b
            if w&(1<<21):res+=self.reg(rd)+(self.reg(rn)<<32)
            self.put(rd,res);self.put(rn,res>>32)
            if w&(1<<20):self.n=(res>>63)&1;self.z=int((res&0xffffffffffffffff)==0)
            return
        if op&0x0fc000f0==0x00000090: # mul / mla
            res=self.reg(w&15)*self.reg((w>>8)&15)
            if w&(1<<21):res+=self.reg(rd)
            self.put(rn,res)
            if w&(1<<20):self.nz(res)
            return
        if op&0x0ff00090==0x01600080: # SMULxy
            a=signed((self.reg(w&15)>>(16 if w&32 else 0))&65535,16)
            b=signed((self.reg((w>>8)&15)>>(16 if w&64 else 0))&65535,16)
            self.put(rn,a*b);return
        if op&0x0e000090==0x00000090: # halfword / signed load
            kind=(w>>5)&3;load=bool(w&(1<<20))
            off=(((w>>8)&15)<<4 | (w&15)) if w&(1<<22) else self.reg(w&15)
            base=self.reg(rn);delta=off if w&(1<<23) else -off
            pre=bool(w&(1<<24));a=base+delta if pre else base
            if not load and kind in (2,3):
                if kind==2:
                    self.put(rd,self.read(a&MASK,4));self.put(rd+1,self.read((a+4)&MASK,4))
                else:
                    self.write(a&MASK,self.reg(rd));self.write((a+4)&MASK,self.reg(rd+1))
                if not pre or w&(1<<21):self.put(rn,base+delta)
                return
            if kind==1:n=2
            elif load and kind==2:n=1
            elif load and kind==3:n=2
            else:raise ValueError(f'unsupported double transfer {w:08x}')
            if load:
                value=self.read(a&MASK,n)
                if kind in (2,3):value=signed(value,n*8)
                self.put(rd,value)
            else:self.write(a&MASK,self.reg(rd),n)
            if not pre or w&(1<<21):self.put(rn,base+delta)
            return
        if op>>25==4: # block transfer
            if w&(1<<22):raise ValueError('privileged block transfer')
            regs=[i for i in range(16) if w&(1<<i)];base=self.reg(rn);up=bool(w&(1<<23));pre=bool(w&(1<<24))
            addr=base+(4 if pre else 0) if up else base-4*len(regs)+(0 if pre else 4)
            for i in regs:
                if w&(1<<20):self.put(i,self.read(addr&MASK,4))
                else:self.write(addr&MASK,self.reg(i))
                addr+=4
            if w&(1<<21):self.put(rn,base+4*len(regs)*(1 if up else -1))
            return
        if op>>26==1: # single transfer
            if w&(1<<25):
                if w&16:raise ValueError('unsupported register-shift transfer')
                off,_=self.shift(self.reg(w&15),(w>>5)&3,(w>>7)&31)
            else:off=w&4095
            base=self.reg(rn);delta=off if w&(1<<23) else -off
            pre=bool(w&(1<<24));addr=base+delta if pre else base;n=1 if w&(1<<22) else 4
            if w&(1<<20):self.put(rd,self.read(addr&MASK,n))
            else:self.write(addr&MASK,self.reg(rd),n)
            if not pre or w&(1<<21):self.put(rn,base+delta)
            return
        if op>>26==0: # data processing
            opcode=(w>>21)&15;flags=bool(w&(1<<20));a=self.reg(rn);b,carry=self.operand(w)
            if opcode in (8,9,10,11) and not flags:raise ValueError(f'unsupported system instruction {w:08x}')
            arithmetic=False;total=0;overflow=0
            if opcode in (0,8):res=a&b
            elif opcode in (1,9):res=a^b
            elif opcode==12:res=a|b
            elif opcode==13:res=b
            elif opcode==14:res=a&~b
            elif opcode==15:res=~b
            else:
                arithmetic=True
                if opcode in (2,10):x,y,cin=a,(~b)&MASK,1
                elif opcode==3:x,y,cin=b,(~a)&MASK,1
                elif opcode in (4,11):x,y,cin=a,b,0
                elif opcode==5:x,y,cin=a,b,self.c
                elif opcode==6:x,y,cin=a,(~b)&MASK,self.c
                elif opcode==7:x,y,cin=b,(~a)&MASK,self.c
                else:raise ValueError(f'unknown alu {w:08x}')
                total=x+y+cin;res=total&MASK;overflow=int((~(x^y)&(x^res)&0x80000000)!=0)
            if flags:
                self.nz(res)
                self.c=int(total>MASK) if arithmetic else carry
                if arithmetic:self.v=overflow
            if opcode not in (8,9,10,11):self.put(rd,res)
            return
        raise ValueError(f'unsupported instruction {w:08x}')
