#!/usr/bin/env python3
"""Extract native tables and bounded comparison results; annotate reviewed branches.
This is not an automatic decompiler. Semantic annotations retain A/B boundaries.
"""
from pathlib import Path
import hashlib,json,re,struct
from recover_event_routes import parse,run
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps';BASE=0x88000000
SHA='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
def main():
 b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes();assert hashlib.sha256(b).hexdigest()==SHA
 def region(lo,hi):return {'start':hex(BASE+lo),'end_exclusive':hex(BASE+hi),'file_offset':hex(lo),'sha256':hashlib.sha256(b[lo:hi]).hexdigest()}
 def imm(pc,register):
  h=struct.unpack_from('<H',b,pc)[0];assert h>>8==0x20+register,(hex(pc),hex(h));return h&255
 def write(name,data):(O/name).write_text(json.dumps({'source_sha256':SHA,**data},ensure_ascii=False,indent=2)+'\n')
 # Literal MOVS operands, checked independently of the executable reference model.
 c={hex(pc):imm(pc,reg) for pc,reg in [(0xb50d3e,6),(0xb50d40,4),(0xb50d42,5),(0xb50d6a,0),(0xb50d78,4),(0xb50d7c,0),(0xb50d8c,0),(0xb50d98,0),(0xb50d9e,0),(0xb50da2,0),(0xb50daa,0)]}
 v=lambda pc:c[hex(pc)]
 rows=[]
 for alphabet,extra,lang,shift,single,multi in [
  (0,False,False,False,v(0xb50d9e),v(0xb50da2)),
  (0,False,True,False,v(0xb50d8c),v(0xb50d98)),
  (0,False,True,True,v(0xb50d8c),v(0xb50d78)),
  (0,True,None,None,v(0xb50d7c),v(0xb50d78)),
  (1,False,None,None,v(0xb50d40),v(0xb50d42)),
  (1,True,None,None,v(0xb50d3e),v(0xb50daa)),
  (2,False,None,None,v(0xb50d40),v(0xb50d42)),
  (2,True,None,None,v(0xb50d3e),v(0xb50d6a))]:
  rows.append(dict(alphabet=alphabet,extra_header=extra,language_shift=lang,single_shift=shift,single=single,multipart=multi))
 write('sms-encoding-rules.json',{'evidence':'A numeric branches; B alphabet and national-shift names from content conversion and related framework. No UI counter format claimed.','capacity_function':region(0xb50d34,0xb50db0),'immediates':c,'capacity_rows':rows,'flag_locations':{'extra_header':'r3, caller send context +0x153a','language_shift':'[0x81d16394+0x2c]','single_shift':'[0x81d16394+0x2d]'},'segment_tail_rule':{'range':region(0xb50ea2,0xb50ed0),'alphabets':[0,1],'last_byte':27,'action':'decrease chosen encoded segment length by one'},'content_conversion':{'entry':'0x883d92f4','alphabet_field':'+0x10','length_field':'+0x0 (u16)','ucs2_copy_length':'min(input UTF-16 units * 2, 804)','caveat':'Byte capacity is not the screen character count. Extra-header UCS2 single capacity 133 is odd; do not silently replace it with a generic SMS formula.'}})
 ins=parse(O/'sms-retry-causes-thumb.txt');accepted=[];traces={}
 for cause in range(65536):
  pc,why,trace=run(ins,b,0x883c5e68,'r0',cause)
  assert pc==0x883c5eca and why=='side-effect boundary: bx',(cause,hex(pc),why)
  if 0x883c5ec8 in trace:accepted.append(cause);traces[str(cause)]=[hex(x) for x in trace]
 write('sms-retry-rules.json',{'evidence':'A: pure integer comparison tree exhaustively evaluated over 0..65535. It does not call code or touch handset/RAM. B names use matching native log strings.','function':region(0x3c5e68,0x3c5ecc),'accepted_rl_causes':accepted,'comparison_paths':traces,'error_handler':region(0x3c407e,0x3c40e0),'error_handler_continuation':region(0x3c421a,0x3c4242),'decision_order':[{'condition':'cause == 0','action':'log and return'},{'condition':'cause in {4,9}','action':'send_result=2; no-retry helper with finish_flag=0'},{'condition':'retry byte > 0 and allowlist(rl_cause)','action':'decrement retry byte; result=2; send_status=2; call 0x883c4030'},{'condition':'otherwise','action':'result=2; no-retry helper with finish_flag=1'}],'continuation':'0x883c4030 requires active context, context+0x1508 == 3 and non-null 0x883c242a result before 0x880c8a02. This branch is not an unconditional 5000-unit timer.','unknown':['initial retry budget for all send modes','save/send confirmation race and full mailbox transitions','all cancellation causes and user-visible outcomes']})
 d=R/'tools/upstream-reference/mmi-framework';p=json.loads((d/'provenance.json').read_text());f='brw_control_api.h';item=next(x for x in p['files'] if x['file']==f);raw=(d/f).read_bytes();assert hashlib.sha256(raw).hexdigest()==item['local_sha256']
 enum=raw.decode().split('BRW_MSG_INIT_CNF=0,',1)[1].split('}BRW_EXPORT_MSG_E;',1)[0]
 names=['BRW_MSG_INIT_CNF']+re.findall(r'^\s*(BRW_MSG_[A-Z0-9_]+)\s*,',enum,re.M)
 assert len(names)>=22
 # The last target extends beyond the main return; include the complete dispatcher.
 handlers={0xb62c0e:'filename request',0xb62bb0:'authentication',0xb62bd2:'URL/new-page update',0xb62c48:'progress',0xb62bea:'error',0xb62c50:'download OK',0xb62c58:'download error',0xb62c60:'page received',0xb62c68:'subresource received',0xb62b8e:'display ready',0xb62b96:'repaint ready',0xb62c18:'certificate prompt',0xb62c92:'page stopped',0xb62cc6:'OMA download confirmation',0xb62cce:'layout destroy',0xb62cd4:'return true / no operation',0xb62cd8:'HTTP refresh'}
 table=b[0xb62b78:0xb62b8e];rows=[]
 for i,x in enumerate(table):
  target=0xb62b78+2*x;rows.append({'message':hex(0x300+i),'index':i,'table_byte':x,'target':hex(BASE+target),'framework_name_B':names[i],'reviewed_role':handlers[target]})
 write('browser-async-routes.json',{'evidence':'A table/targets and native calls; B symbolic names from a different-product header. Page history, JS/DOM and transport internals not recovered.','function':region(0xb62b56,0xb62ce0),'table':region(0xb62b78,0xb62b8e),'reference':item,'rows':rows})
 bad=b[0x3b49d8:0x3b49dc];assert bad==b'\r\n\0\0'
 write('browser-address-auth.json',{'evidence':'A manually traced native conditions; field names associated with native resources and public framework.','address_submit':region(0xb65c54,0xb65cbe),'narrow_copy':region(0x3a51dc,0x3a522c),'precheck':region(0x3b4724,0x3b4768),'invalid_bytes':list(bad[:2]),'invalid_bytes_offset':'0x3b49d8','address_max_utf16_units':512,'address_initial_text':'http://','invalid_resource':'0x40000872','copy_behavior':'copy <=512 UTF-16 units to one-byte buffer; stop/zero-fill at NUL or first unit >255; no punycode or UTF-8 call in this entry','precheck_behavior':'reject empty, first byte >127, any CR/LF; the >127 test reads byte zero, not each indexed byte','success':'StartWeb(entry=1, narrowed bytes, null); close address window','auth':{'window':'0x60008','handler':'0x88b62ce0','username_window':'0x60016','password_window':'0x60017','field_limit':20,'selection_rows':{'2':'username','4':'password'},'submit':'require username-set flag [context+8]; invoke challenge callback(instance, username, password, 1), clear challenge and close','missing_username':'show 0x40060056 Enter username','cancel':'callback(instance, null, null, 0) if available; otherwise StopPageReq, clear request and refresh; close','empty_username_editor_confirm':'show Enter username and retain editor','empty_password_editor_confirm':'remain in editor; no credential setter or close in this branch','close':'clear stored challenge','limits':'Credentials were never read from phone. Full encoding, allocation-failure handling and network retry remain outside this model.'}})
 write('music-center-route.json',{'evidence':'A callback installation, integer conditions and calls; pause/resume names confirmed by native logs.','installation':region(0xace712,0xace7bc),'vtable_init':region(0x68ace4,0x68acf6),'callback_pointer_slot':{'offset':'0x68ad84','value':hex(struct.unpack_from('<I',b,0x68ad84)[0])},'callback':region(0x68ac44,0x68acd2),'event':'0xfa1b','route':['0x88acf2de','0x88ad0cfe returns active view+0x14','indirect call through view callback +4','0x8868ac44','0x8868aca6','0x88acec7a','0x883f0ff4(5) enable query','0x883f125a','0x883f120c'],'branches':[{'player_state':0,'action':'0x8860105a(0), 0x885ff838 (delay play)'},{'player_state':1,'audio_state':1,'action':'0x885ff048(0,1), 0x885ffaae (pause)'},{'player_state':1,'audio_state':2,'action':'0x885ff968 (resume)'},{'player_state':2,'action':'0x885ffaae (pause)'},{'player_state':3,'action':'0x885ff968 (resume)'},{'player_state':'other','action':'return'}],'caveat':'Only this installed view callback is established. Physical center/WEB key identity and runtime player-state labels need device validation. Global handler remains a fallback when child returns false.'})
 print('5 native algorithm evidence files; 8 capacity rows;',len(accepted),'retry causes; 22 browser async entries; music callback chain')
if __name__=='__main__':main()
