#!/usr/bin/env python3
"""Evaluate bounded event-comparison trees from disassembly as integer expressions.
Not a CPU/firmware emulator: no calls, stores, stack, device or mutable RAM reads.
Stops at the first operation outside the allowlist or with an unknown operand.
"""
from pathlib import Path
import collections,hashlib,json,re,struct
from recover_framework_semantics import defines
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps';BASE=0x88000000;MASK=0xffffffff
def parse(path):
 out={}
 for line in path.read_text().splitlines():
  m=re.match(r'([0-9a-f]{8}):\s+((?:[0-9a-f]{4}\s+){1,2})\s*\t([a-z][a-z0-9.]*)\s*(.*)',line)
  if m:out[int(m[1],16)]=(len(m[2].split())*2,m[3],m[4].split(' <')[0].split(' @')[0],line)
 return out
def run(ins,b,start,register,event):
 regs={register:event};pc=start;flags=None;trace=[]
 def val(s):return int(s[1:],0)&MASK if s.startswith('#') else regs[s]
 def subflags(a,v):
  d=(a-v)&MASK
  return (bool(d&0x80000000),d==0,a>=v,bool(((a^v)&(a^d))&0x80000000))
 def cond(c):
  if flags is None:raise KeyError('flags')
  n,z,cary,v=flags
  if c in ('hi','ls','hs','cs','lo','cc') and cary is None:raise KeyError('carry')
  if c in ('gt','ge','lt','le') and v is None:raise KeyError('overflow')
  return {'eq':z,'ne':not z,'gt':not z and n==v,'ge':n==v,'lt':n!=v,'le':z or n!=v,'hi':cary and not z,'ls':not cary or z,'hs':cary,'cs':cary,'lo':not cary,'cc':not cary,'mi':n,'pl':not n}[c]
 for _ in range(150):
  if pc not in ins:return pc,'outside decoded range',trace
  size,opcode,args,line=ins[pc];op=opcode.removesuffix('.w');parts=[x.strip() for x in args.split(',')];nxt=pc+size
  try:
   if op in ('mov','movs','movw','mvn'):
    value=val(parts[1]);value=(~value)&MASK if op=='mvn' else value;regs[parts[0]]=value
    if op=='movs':
     # Narrow MOVS preserves C/V; do not invent them before a CMP.
     if size!=2:raise KeyError('wide MOVS carry expansion')
     flags=(bool(value&0x80000000),value==0,flags[2] if flags else None,flags[3] if flags else None)
   elif op in ('sub','subs','subw','add','adds','addw'):
    dest=parts[0];a=val(parts[1] if len(parts)==3 else dest);v=val(parts[-1]);is_sub=op.startswith('sub');value=(a-v if is_sub else a+v)&MASK;regs[dest]=value
    if op in ('subs','adds'):
     flags=subflags(a,v) if is_sub else (bool(value&0x80000000),value==0,a+v>MASK,bool((~(a^v)&(a^value))&0x80000000))
   elif op=='cmp':flags=subflags(val(parts[0]),val(parts[1]))
   elif op=='ldr' and '[pc,' in args:
    m=re.search(r'@ 0x([0-9a-f]+)',line);at=int(m[1],16)-BASE
    if not 0<=at<len(b)-4:raise KeyError('non-ROM literal')
    regs[parts[0]]=struct.unpack_from('<I',b,at)[0]
   elif op=='b':nxt=int(parts[0],16)
   elif op.startswith('b') and op[1:] in ('eq','ne','gt','ge','lt','le','hi','ls','hs','cs','lo','cc','mi','pl'):
    if cond(op[1:]):nxt=int(parts[0],16)
   elif op in ('cbz','cbnz'):
    if (val(parts[0])==0)==(op=='cbz'):nxt=int(parts[1],16)
   else:return pc,'side-effect boundary: '+opcode,trace
  except (KeyError,ValueError,IndexError):return pc,'unknown operand: '+opcode,trace
  trace.append(pc);pc=nxt
 return pc,'step bound',trace
def main():
 b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes();known,_=defines(['caf.h','mmk_msg.h','mmi_appmsg.h']);names=collections.defaultdict(list)
 for k,v in known.items():
  if k.startswith(('MSG_','MMI_')):names[v].append(k)
 specs=[('browser-page',0x88b62014,'r9'),('music-player',0x88acf318,'r7'),('sms-options',0x8839bf94,'r6'),('music-view-callback',0x8868ac56,'r4')]
 output=[]
 for name,start,reg in specs:
  ins=parse(O/(name+'-thumb.txt'));groups=collections.defaultdict(list)
  for ev in range(65536):
   stop,reason,trace=run(ins,b,start,reg,ev);groups[(stop,reason)].append(ev)
  dominant=max(groups,key=lambda k:len(groups[k]));rows=[]
  for (stop,reason),events in sorted(groups.items()):
   if (stop,reason)==dominant:continue
   for ev in events:
    _,_,trace=run(ins,b,start,reg,ev)
    rows.append({'event':hex(ev),'framework_names':sorted(names[ev]),'boundary_pc':hex(stop),'boundary_instruction':ins[stop][3] if stop in ins else None,'reason':reason,'comparison_path':[hex(x) for x in trace]})
  assert all(x['reason']!='step bound' for x in rows)
  scope={'music-player':'Applies only when earlier child/view handler returns not handled.',
         'music-view-callback':'Applies to the callback installed by 0x8868ace4; main handler invokes it first.'}.get(name,'Application comparison tree only.')
  output.append({'name':name,'entry':hex(start),'event_register':reg,'assumptions':scope+' No state-dependent body evaluated.','default_boundary':hex(dominant[0]),'default_event_count':len(groups[dominant]),'nondefault_routes':sorted(rows,key=lambda x:int(x['event'],16))})
 (O/'event-routes.json').write_text(json.dumps({'method':'Exhaustive evaluation of 65536 unsigned event IDs in bounded comparison trees, no firmware calls or effects. Boundary is first non-allowlisted instruction, not necessarily start of handler body. Names are cross-version.','source_sha256':hashlib.sha256(b).hexdigest(),'applications':output},indent=2)+'\n')
 for x in output:print(x['name'],len(x['nondefault_routes']),'non-default IDs')
if __name__=='__main__':main()
