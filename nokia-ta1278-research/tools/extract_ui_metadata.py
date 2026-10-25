#!/usr/bin/env python3
"""Extract native locale declarations, customization symbols, audio, GUI descriptors.
Descriptor fragments are callback-anchored evidence, not a complete UI graph.
"""
from pathlib import Path
import struct,json,re,hashlib,shutil,collections,subprocess
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui';RES=R/'analysis/extracted/resources';BASE=0x88000000
def save(name,obj):(O/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
def main():
 index=json.loads((R/'analysis/system/resource-index.json').read_text());symbols=[]
 src=next(x for x in index if x['kind']=='cscs');b=(RES/src['file']).read_bytes();p=4
 while p<len(b):
  at=p;kindgroup,ix,reserved=struct.unpack_from('<III',b,p);p+=12;e=b.index(0,p);name=b[p:e].decode('ascii');p=(e+4)&~3
  assert name and all(32<=ord(c)<127 for c in name) and not any(b[e:p]);symbols.append({'name':name,'kind':kindgroup&255,'group':kindgroup>>8,'index':ix,'reserved':reserved,'offset':src['offset']+at,'resource_id':hex(0x40000000+((kindgroup>>8)<<16)+ix+1) if kindgroup&255==0 else None})
 assert p==len(b);save('resource-symbols.json',{'source':src['file'],'layout':'u32 prefix; repeated u32 kind/group, u32 zero-based index, u32 reserved, NUL ASCII padded to 4','symbols':symbols})
 src=next(x for x in index if x['kind']=='lang');b=(RES/src['file']).read_bytes();assert len(b)==76
 locales=[]
 for i in range(3):
  p=4+24*i;locale=struct.unpack_from('<I',b,p)[0];name=b[p+4:p+24].split(b'\0')[0].decode();assert locale==i;locales.append({'locale':locale,'name':name,'offset':src['offset']+p})
 save('locales.json',locales)
 aud=O/'audio';aud.mkdir(exist_ok=True);audio=[]
 for row in index:
  if row['kind']!='sound':continue
  b=(RES/row['file']).read_bytes();ext='.mid' if b[:4]==b'MThd' else '.wav' if b[:4]==b'RIFF' else '.amr' if b.startswith(b'#!AMR') else '.mp3' if b[:3]==b'ID3' or b[:2]==b'\xff\xfb' else None
  item={'source':row['file'],'offset':row['offset'],'size':len(b),'sha256':hashlib.sha256(b).hexdigest(),'format':ext,'placeholder':b==b'is null temply'}
  if ext:
   path=aud/(Path(row['file']).stem+ext);path.write_bytes(b);item['file']=str(path.relative_to(O))
  audio.append(item)
 save('audio-index.json',audio)
 b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes();smsnames=[];p=0x21b2a6;i=1
 while b[p:p+6]==b'MMISMS':
  name=b[p:p+50].split(b'\0')[0].decode('ascii');smsnames.append({'id':hex(0x80000+i),'name':name,'offset':hex(p)});i+=1;p+=50
 names={int(x['id'],16):x['name'] for x in smsnames};save('sms-window-names.json',{'registration':'0x88ab0718','lookup':'0x885f237e..0x885f2392: base + (low16(ID)-1)*50','names':smsnames})
 if not (O/'window-name-registration-xrefs.txt').exists():
  argv=['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi','--mcpu=cortex-a5',str(O/'user.analysis-only.elf')]
  process=subprocess.Popen(argv,stdout=subprocess.PIPE,text=True)
  lines=[line for line in process.stdout if re.search(r'(?:b.w|bl).*0x885f2332',line)]
  assert process.wait()==0
  (O/'window-name-registration-xrefs.txt').write_text(''.join(lines))
 registries=[]
 for line in (O/'window-name-registration-xrefs.txt').read_text().splitlines():
  at=int(line.split(':')[0],16)-BASE
  first,second=struct.unpack_from('<HH',b,at-4)
  if first&0xff00!=0x4900 or second&0xff00!=0x2000:continue
  literal=(at&~3)+(first&255)*4
  start=struct.unpack_from('<I',b,literal)[0]-BASE
  if 0<=start<len(b):registries.append({'group':second&255,'base':start,'registration':hex(BASE+at-4)})
 all_names=[]
 for reg in registries:
  at=reg['base'];end=min([x['base'] for x in registries if x['base']>at]+[len(b)]);i=1
  while at+50<=end:
   raw=b[at:at+50];name=raw.split(b'\0')[0]
   if not re.fullmatch(rb'[A-Z][A-Z0-9_]{4,49}',name):break
   if any(raw[len(name):]):break
   item={'id':hex((reg['group']<<16)+i),'name':name.decode(),'offset':hex(at),'registration':reg['registration']};all_names.append(item);names[int(item['id'],16)]=item['name'];at+=50;i+=1
  reg['names']=i-1
 save('window-names.json',{'registries':registries,'names':all_names,'note':'Only literal-load/movs registration wrappers and bounded 50-byte names. Missing/dynamic registrations remain open.'})
 texts=json.loads((O/'strings.json').read_text());ss=collections.defaultdict(dict)
 for x in texts:ss[int(x['resource_id'],16)][str(x['locale'])]=x['text']
 windows=[]
 for m in re.finditer(re.escape(struct.pack('<I',0xf001)),b):
  at=m.start()
  if at%4 or at+16>len(b):continue
  handler,op,wid=struct.unpack_from('<III',b,at+4)
  if op!=0xf002 or not(0<wid>>16<0x46 and BASE<handler<BASE+len(b) and handler&1):continue
  end=None
  for q in range(at+16,min(at+1024,len(b)-4),4):
   if struct.unpack_from('<I',b,q)[0]==0xf032:end=q+4;break
  if end is None:continue
  words=struct.unpack_from('<'+str((end-at)//4)+'I',b,at);attrs=[];j=4
  while j<len(words)-1:
   op=words[j];args=[];j+=1
   while j<len(words)-1 and not(0xf000<=words[j]<=0xf1ff):args.append(words[j]);j+=1
   attrs.append({'opcode':hex(op),'arguments':[hex(x) for x in args], 'strings':{hex(x):ss[x] for x in args if x in ss}})
  windows.append({'offset':hex(at),'size':end-at,'sha256':hashlib.sha256(b[at:end]).hexdigest(),'window_id':hex(wid),'name':names.get(wid),'handler_thumb':hex(handler),'attributes':attrs,'words':[hex(x) for x in words]})
 save('window-descriptors.json',{'note':'Static callback-anchored fragments; prefix attributes may precede f001. Opcode argument boundaries inferred from f000..f1ff sentinels. No assertion of runtime reachability or full transitions.','count':len(windows),'sms_count':sum(int(x['window_id'],16)>>16==8 for x in windows),'windows':windows})
 print({'symbols':len(symbols),'locales':locales,'audio_files':sum(bool(x['format']) for x in audio),'sms_names':len(smsnames),'all_window_names':len(names),'window_fragments':len(windows)})
if __name__=='__main__':main()
