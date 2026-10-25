#!/usr/bin/env python3
"""Annotate native tables with explicitly version-bounded public framework evidence.
Parses integer expressions with an AST allowlist, never executes upstream code.
"""
from pathlib import Path
import ast,hashlib,json,re,struct,collections
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui';S=R/'tools/upstream-reference/mmi-framework'
def save(name,obj):(O/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
def stripped(s):return re.sub(r'//[^\n]*|/\*.*?\*/','',s,flags=re.S)
def value(expr,known):
 def walk(n):
  if isinstance(n,ast.Constant) and isinstance(n.value,int):return n.value
  if isinstance(n,ast.Name):return known[n.id]
  if isinstance(n,ast.UnaryOp) and isinstance(n.op,ast.USub):return -walk(n.operand)
  if isinstance(n,ast.BinOp):
   a,b=walk(n.left),walk(n.right)
   if isinstance(n.op,ast.Add):return a+b
   if isinstance(n.op,ast.Sub):return a-b
   if isinstance(n.op,ast.BitOr):return a|b
   if isinstance(n.op,ast.LShift):return a<<b
  raise ValueError(expr)
 return walk(ast.parse(expr.strip(),mode='eval').body)
def defines(files):
 raw={};loc={}
 for f in files:
  for i,line in enumerate(stripped((S/f).read_text()).splitlines(),1):
   m=re.match(r'\s*#define\s+(\w+)\s+(.+)',line)
   if m:raw[m[1]]=m[2].strip();loc[m[1]]=f
 known={}
 for _ in range(30):
  n=len(known)
  for k,v in raw.items():
   if k in known:continue
   try:known[k]=value(v,known)
   except (ValueError,KeyError,SyntaxError,TypeError):pass
  if len(known)==n:break
 return known,loc
def enums(file,tags):
 s=stripped((S/file).read_text());known={}
 for tag in tags:
  body=re.search(r'typedef enum\s+'+tag+r'\s*\{(.*?)\}',s,re.S)[1];n=-1
  for line in body.split(','):
   line=line.strip()
   if not line:continue
   if '=' in line:name,expr=map(str.strip,line.split('=',1));n=value(expr,known)
   else:name=line;n+=1
   assert re.fullmatch(r'\w+',name);known[name]=n
 return known
def main():
 b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
 assert hashlib.sha256(b).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
 known,loc=defines(['caf.h','mmk_msg.h','mmi_appmsg.h']);event_names=collections.defaultdict(list)
 for k,v in known.items():
  if k.startswith(('MSG_','MMI_')):event_names[v].append(k)
 interaction=json.loads((O/'interaction-map.json').read_text())
 for row in interaction['compose_dispatch']['branches']:
  row['framework_names']=sorted(event_names[int(row['event'],16)])
  row['name_confidence']='framework-correlated' if row['framework_names'] else 'unresolved'
 interaction['compose_dispatch']['event_names']='Public caf.h/mmk_msg.h/mmi_appmsg.h matches; firmware IDs/branch targets verified, names inferred across versions. Preprocessor branches are not a recovered build configuration.'
 ops=enums('window_parse.h',['parse_win_tag','parse_child_ctrl_tag']);opnames={v:k for k,v in ops.items() if k not in ('FIRST_PARSE','CHILD_CTRL_FIRST','LAST_PARSE','LAST_CHILD')}
 # Independent anchors: all native streams use these exact tag values.
 assert ops['MMK_WINFUNC']==0xf001 and ops['MMK_WINID']==0xf002
 # Native terminal tag is f032; public revision ends at f031. Do not silently
 # transplant late enum members across this demonstrated version difference.
 opnames.pop(0xf031,None);opnames[0xf032]='NATIVE_END_WIN'
 assert ops['CREATE_CHILD_FORM']==0xf110 and ops['CREATE_CHILD_EDIT_TEXT']==0xf101
 macro_s=stripped((S/'window_parse.h').read_text()).replace('\\\n',' ');schemas={}
 for m in re.finditer(r'#define\s+(\w+)\(([^)]*)\)\s+([^\n]+)',macro_s):
  operands=[x.strip() for x in m[3].split(',')];tag=operands[0]
  if tag in ops and len(operands)>1:schemas[ops[tag]]={'macro':m[1],'operands':operands[1:]}
 windows=json.loads((O/'window-descriptors.json').read_text())['windows'];annotated=[]
 for w in windows:
  row={k:v for k,v in w.items() if k!='words'}
  for a in row['attributes']:
   n=int(a['opcode'],16);a['framework_opcode']=opnames.get(n,'unresolved')
   if n in schemas:
    sc=schemas[n]
    a['framework_macro']=sc['macro'];a['operand_schema_match']=len(sc['operands'])==len(a['arguments'])
    if a['operand_schema_match']:a['named_arguments']=dict(zip(sc['operands'],a['arguments']))
  annotated.append(row)
 save('window-semantics.json',{'confidence':'framework-correlated; 751 native byte slices verified separately; conditional/dynamic attributes remain unresolved','version_difference':'public END_WIN=f031; native terminal=f032. Native f031 is deliberately unnamed.','windows':annotated,'opcode_map':{hex(v):k for v,k in opnames.items()}})
 for node in interaction['compose_layout']['nodes']:
  op=int(node['opcode'],16);node['type']=opnames[op];sc=schemas.get(op)
  if sc:node['named_arguments']=dict(zip(sc['operands'],node['parameters']+[node['node'],node['parent']]))
 interaction['compose_layout']['interpretation']='Child types/operand names correlated with window_parse.h. FORM layout 0=vertical order, 1=side by side. Capacity is construction-time, not proven final SMS limit.'
 interaction['cursor_directions']={'1':'up','2':'down','3':'left','4':'right','confidence':'native four-way dispatch and error literal match BASEFLEX_MoveCursor; source conditionals differ','native':'0x882ffe3c','source':'ctrlbaseflex_cursor.c'}
 interaction['known_gaps']=[x for x in interaction['known_gaps'] if x!='Event ID names and complete handler side effects']+['All handler side effects not fully recovered; cross-version event names require runtime acceptance']
 save('interaction-map.json',interaction)
 # This exact contiguous native sequence matches the public indexed table.
 at=0x4acb58;keytable=b[at:at+0x69]
 remap_at=struct.unpack_from('<I',b,0x135344)[0]-0x88000000
 assert remap_at-0x69==at
 remaps=dict(struct.unpack_from('<BB',b,remap_at+i*2) for i in range(12))
 assert keytable[:8]==bytes([0,4,5,6,7,1,14,0]) and keytable[8:20]==b'123456789*0#'
 rows=[]
 for name,k in known.items():
  if name.startswith('KEY_') and 0<k<len(keytable) and keytable[k]:
   effective=remaps.get(k,k)
   rows.append({'framework_key':name,'ui_key_code':hex(k),'effective_ui_key_code':hex(effective),'native_signal_value':hex(keytable[k]),'native_table_offset':hex(at+k),'down':hex(0xfa00|effective),'up':hex(0xfb00|effective),'long_down':hex(0xfc00|effective),'long_up':hex(0xfd00|effective),'repeat':hex(0xfe00|effective)})
 kernel=(R/'analysis/work/system-20261025T054301+0800/kernel.bin').read_bytes();matrix=struct.unpack_from('<64H',kernel,0xb401cc);physical=[]
 for i,v in enumerate(matrix):
  if v==0xffff:continue
  normalized={0x24:0x0a,0x25:0x0b,0x66:0x5f}.get(v,v)&255;code=keytable.index(normalized) if normalized in keytable else 0;code=remaps.get(code,code);assert code!=0
  physical.append({'matrix_row':i//8,'matrix_column':i%8,'scan_value':hex(v),'ui_key_code':hex(code),'down_event':hex(0xfa00+code)})
 assert len(physical)==19
 save('key-event-map.json',{'confidence':'native converter and caller traced; key names correlated with public CAF definitions; center/power electrical paths not inspected','converter':'0x88135052','caller':'0x882e2e9c..0x882e2ea4','secondary_remap':'0x8813502e','table_offset':hex(at),'raw_hex':keytable.hex(),'normalization':{'0x24':'0x0a','0x25':'0x0b','0x66':'0x5f'},'qwerty_to_numeric_remaps':{hex(k):hex(v) for k,v in remaps.items()},'rows':sorted(rows,key=lambda x:int(x['ui_key_code'],16)),'physical_matrix':physical,'notes':['Do not confuse kernel scan codes with CAF key codes.','Native converter maps signal 0x0d to KEY_WEB (fa1b), 0x02 to RED (fa18); association with physical center/power needs wiring or observation.','Native key dispatch uses fa00/fb00/fc00/fd00/fe00 prefixes; long-press/repeat threshold is not recovered.']})
 # Audit all 77 non-ASCII single-byte resource records without replacing bytes.
 ss=json.loads((O/'strings.json').read_text());odd=[]
 for x in ss:
  if x.get('encoding')!='8-bit-latin1-provisional':continue
  raw=bytes.fromhex(x['raw_hex']);a=raw.decode('latin1');c=raw.decode('cp1252',errors='replace')
  odd.append({'resource_id':x['resource_id'],'locale':x['locale'],'offset':x['offset_hex'],'raw_hex':x['raw_hex'],'latin1':a,'windows1252':c,'both_decoders_identical':a==c,'question_mark_bytes':raw.count(b'?')})
 save('single-byte-text-audit.json',{'records':odd,'identical_latin1_windows1252':sum(x['both_decoders_identical'] for x in odd),'irrecoverable_original_text_records':[x for x in odd if x['raw_hex']=='3f3fea3fed3f3f3f'],'note':'The three ??ê?í??? records already contain literal 0x3f bytes. Intended lost characters cannot be inferred uniquely from this firmware. Codepage name remains unproven.'})
 # Inventory per app family, not a fabricated complete transition graph.
 families={}
 for family,pat in [('messages','SMS|MMS'),('browser','BROWSER|WWW|WAP'),('home','IDLE|MAINMENU'),('settings','SET|PROFILE'),('input','IM_|INPUT|PINYIN')]:
  ww=[x for x in annotated if re.search(pat,x.get('name') or '')]
  families[family]={'count':len(ww),'windows':[{'id':x['window_id'],'name':x.get('name'),'handler':x['handler_thumb'],'offset':x['offset']} for x in ww]}
 save('application-window-index.json',{'scope':'name-based inventory; overlaps are intentional and names do not prove feature availability','families':families})
 save('framework-semantics-validation.json',{'status':'passed','named_compose_events':sum(bool(x['framework_names']) for x in interaction['compose_dispatch']['branches']),'compose_events':25,'window_count':len(annotated),'annotated_attributes':sum('framework_opcode' in a for w in annotated for a in w['attributes']),'keytable_bytes':len(keytable),'physical_matrix_keys_mapped':len(physical),'single_byte_records':len(odd),'firmware_executed':False})
 print('Framework annotation:',len(annotated),'windows;',sum(bool(x['framework_names']) for x in interaction['compose_dispatch']['branches']),'of 25 events named')
if __name__=='__main__':main()
