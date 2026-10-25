#!/usr/bin/env python3
"""Selected application disassembly and literal references, never firmware execution."""
from pathlib import Path
import hashlib,json,re,struct,subprocess
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps';BASE=0x88000000
RANGES=[
 ('sms-storage-flags',0x80100,0x802c0),('sms-draft-helpers',0x397d60,0x398020),
 ('browser-navigation-core',0xbda20,0xbda78),('browser-key-timers',0xb619fc,0xb61a78),
 ('music-transport-helpers',0xacecce,0xacf2de),('music-view-dispatch',0xad0a00,0xad1100),
 ('sms-compose-helpers',0x398020,0x39962a),('sms-compose',0x39962a,0x39af96),
 ('sms-options',0x39bf5a,0x39cb70),('sms-recipient-editor',0x39cb70,0x39d6d8),
 ('sms-text-editor',0x39d6d8,0x39d96e),('sms-insert-options',0x39e5b4,0x39f13c),
 ('browser-page',0xb61ff2,0xb62a68),('browser-address',0xb62a68,0xb62ce0),
 ('browser-options',0xb6361c,0xb639b4),('browser-start',0xb6483c,0xb649c4),
 ('browser-start-options',0xb649c4,0xb64e56),
 ('music-player',0xacf2de,0xad0a00),('music-settings',0x68825a,0x688f40),
 ('music-repeat-shuffle',0x689a9c,0x689bf6),('music-repeat-list',0x68a1aa,0x68a310),('music-track-list',0xad3f96,0xad46f0)]
def main():
 O.mkdir(exist_ok=True);b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
 assert hashlib.sha256(b).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
 strings=json.loads((R/'analysis/ui/strings.json').read_text());ss={}
 for x in strings:ss.setdefault(int(x['resource_id'],16),{})[x['locale']]=x['text']
 index=[]
 for name,lo,hi in RANGES:
  out=subprocess.check_output(['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi','--mcpu=cortex-a5',f'--start-address={BASE+lo}',f'--stop-address={BASE+hi}',str(R/'analysis/ui/user.analysis-only.elf')],text=True)
  out=out.replace(str(R),'PROJECT_ROOT');(O/(name+'-thumb.txt')).write_text(out)
  refs=[];calls=[]
  for line in out.splitlines():
   m=re.match(r'([0-9a-f]{8}):.*?\t([a-z][a-z0-9.]*)\s+(.*)',line)
   if not m:continue
   pc=int(m[1],16);op=m[2];args=m[3];ref=None
   if op in ('bl','blx'):
    t=re.search(r'0x([0-9a-f]+)',args)
    if t:calls.append({'at':hex(pc),'target':hex(int(t[1],16))})
   if op.startswith('ldr') and '[pc,' in args:
    t=re.search(r'@ 0x([0-9a-f]+)',args)
    if t:
     at=int(t[1],16)-BASE
     if 0<=at<=len(b)-4:ref=struct.unpack_from('<I',b,at)[0]
   elif op.startswith('adr'):
    t=re.search(r'#(-?[0-9]+)',args)
    if t:ref=((pc+4)&~3)+int(t[1])
   if ref is None:continue
   row={'at':hex(pc),'value':hex(ref)}
   if ref in ss:row['resource_text']=ss[ref]
   if BASE<=ref<BASE+len(b):
    raw=b[ref-BASE:ref-BASE+300].split(b'\0')[0]
    if len(raw)>=5 and all(32<=c<127 for c in raw):row['ascii']=raw.decode()
   refs.append(row)
  index.append({'name':name,'file':name+'-thumb.txt','start':hex(BASE+lo),'end_exclusive':hex(BASE+hi),'source_range_sha256':hashlib.sha256(b[lo:hi]).hexdigest(),'literal_references':refs,'direct_call_candidates':calls})
 (O/'code-index.json').write_text(json.dumps({'scope':'Linear disassembly includes literal pools. Reference/call candidates are not a complete CFG or proven functions. Semantics require branch tracing.','ranges':index},ensure_ascii=False,indent=2)+'\n')
 print(len(index),'application disassembly ranges')
if __name__=='__main__':main()
