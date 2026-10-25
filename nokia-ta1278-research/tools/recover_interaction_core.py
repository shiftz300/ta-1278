#!/usr/bin/env python3
"""Record reviewed native UI rules with exact byte anchors; no firmware execution.
Semantic statements are manual branch reconstructions, not a decompiler output.
"""
from pathlib import Path
import hashlib,json,struct
from recover_event_routes import parse,run
from native_app_rules import SMS_R8_ONE,SMS_R8_TWO,SMS_LANG_0,SMS_LANG_2,SMS_LANG_4,SMS_DEFAULT
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps';BASE=0x88000000
def main():
 b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
 sha=hashlib.sha256(b).hexdigest();assert sha=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
 def region(lo,hi):return {'start':hex(BASE+lo),'end_exclusive':hex(BASE+hi),'sha256':hashlib.sha256(b[lo:hi]).hexdigest()}
 def word(at):return struct.unpack_from('<I',b,at)[0]
 # Independently enumerate only bounded integer predicate trees. No calls,
 # stack, stores or mutable RAM are evaluated. A true branch visits mov r1,1.
 ins=parse(O/'sms-character-classification-thumb.txt');predicates={}
 for name,start,yes,expected in [('r8_one',0x3a483a,0x3a4850,SMS_R8_ONE),('r8_two',0x3a4856,0x3a48d6,SMS_R8_TWO),('language_0',0x3a4aac,0x3a4ad8,SMS_LANG_0),('language_2',0x3a4a60,0x3a4aa2,SMS_LANG_2),('language_4',0x3a4a34,0x3a4a56,SMS_LANG_4)]:
  found=[]
  for c in range(65536):
   _,reason,path=run(ins,b,BASE+start,'r0',c)
   assert reason=='side-effect boundary: bx',(name,c,reason)
   if BASE+yes in path:found.append(c)
  assert set(found)==expected,(name,found)
  predicates[name]={'entry':hex(BASE+start),'true_branch':hex(BASE+yes),'codepoints':found}
 tables={}
 for name,literal in [('default',0xacb574),('extension',0xacb570)]:
  at=word(literal)-BASE;values=list(struct.unpack_from('<128H',b,at))
  tables[name]={'offset':hex(at),'sha256':hashlib.sha256(b[at:at+256]).hexdigest(),'codepoints':values}
 assert set(tables['default']['codepoints'])==SMS_DEFAULT
 at=word(0x13bba4)-BASE;values=struct.unpack_from('<434H',b,at);target=word(0x13bba8)-BASE
 assert max(values[::2])<128
 pairs=[{'unicode':c,'default_index':i,'replacement':struct.unpack_from('<H',b,target+2*i)[0]} for i,c in zip(values[::2],values[1::2])]
 alphabet={'source_sha256':sha,'predicates':predicates,'tables':tables,
  'conversion':{'offset':hex(at),'sha256':hashlib.sha256(b[at:at+868]).hexdigest(),'entries':pairs},
  'count_entry':'0x883a492c','language_count_entry':'0x883a4ade','editor_update_entry':'0x880b0f0e',
  'method':'Original tables plus exhaustive bounded comparison-tree evaluation for each of 65536 UTF16 units, independently checked against manually transcribed sets. No firmware execution.',
  'rules':'SMS UCS2 predicate: code>127 and absent from R8 one/two, default and extension tables; if member of 217-entry conversion table and alphabet_management==0, clear UCS2. Priority UCS2 > R8 two > R8 one > default extension (unless sms_encode==2). Language counters 0/2/4 classify independently; 1/3 remain zero in this build.',
  'limits':'Current alphabet_management (global+0xc60) and editor sms_encode (+0x244) are not read from private data. Conversion table describes available replacement, not proof every send path applies it. Full wire packing, text edits and settings changes remain separate.'}
 (O/'sms-character-tables.json').write_text(json.dumps(alphabet,ensure_ascii=False,indent=2)+'\n')
 assert b[0x352308:0x35230e]==b'%d/%d\0'
 assert b[0x3522d4:0x3522d8]==bytes.fromhex('b5fbf0f1') # UDIV, visible with +hwdiv
 assert b[0x351b80:0x351b84]==bytes.fromhex('95fbf0f0') # SDIV
 assert word(0xb490e8)==0x60065 and word(0xb490ec)==0x60015
 assert word(0xacfa38)==0xe0017 and word(0xacfa3c)==0x8820311c
 data={'source_sha256':sha,'method':'Manual native branch tracing, literal checks and hashed code ranges. A: numbers/conditions/effects. B: public framework field names. No target code was run.',
 'sms':{'regions':[region(0x351b12,0x351c0c),region(0x352232,0x352306),region(0xb0754,0xb0784),region(0x6240c8,0x624216)],
  'format':{'offset':'0x352308','value':'%d/%d','arguments':['remaining','page_count']},
  'fields':{'str_len':'+0x94 u16','str_max_len':'+0x96 u16','is_sms':'+0x240 byte','is_pb_encode':'+0x241 byte','ucs2_count':'+0x248 u16','default_ext':'+0x24a u16','r8_one':'+0x24c u16','r8_two':'+0x24e u16','language_counts':'+0x250..0x258, five u16'},
  'page_fields':{'ucs2':['+0x262','+0x264'],'default':['+0x266','+0x268'],'r8_one':['+0x26a','+0x26e'],'r8_two':['+0x26c','+0x270']},
  'initializer':'0x886240c8 calls capacity(alphabet=0 or 2, extra_header=0) and capacity(alphabet=2, extra_header=0); explicitly shifts the UCS2 byte outputs right by one. Settings 1/2 use alphabet 2 for the default slots. Others use alphabet 0. Current device setting was not read.',
  'ordinary_page_values':{'ucs2':[70,67],'default':[160,153],'r8_one':[155,149],'r8_two':[155,146]},
  'max_length_fields':{'settings_1_2':[402,804,803,802],'other_settings':[402,918,894,876]},
  'formula':'used=str_len for UCS2, otherwise uint16(str_len+default_ext+r8_two). page=1 when used<=single or multi=0, else floor((used-1)/multi)+1. remaining=uint16(single-used) on single/zero-multi branch, else (multi-used%multi)%multi.',
  'important_difference':'When r8_two>0, page count uses r8_two capacities only if language fields +250/+254/+256 are nonzero; remaining count always uses r8_two. Preserve both branches. UI division does not split ESC pairs; send segmentation is separate.',
  'scope':'Shared SMS TextEdit widget, called by SMS body menu at 0x883989d4. Per-character classification is recorded in sms-character-tables.json; actual selected encoding/shift configuration remains external. Phonebook post-adjustment at 351bb2 is excluded from the SMS model.'},
 'browser_fields':{'regions':[region(0xb488b4,0xb4896e),region(0xb4896e,0xb48b1e),region(0xb48cac,0xb48fb8),region(0x629754,0x6297f2),region(0x629848,0x629878),region(0x629bae,0x629bde)],
  'window':'0x60015','control':'0x60065','handlers':{'wml':'0x88b4896e','xhtml':'0x88b48a76'},
  'confirm_events':['0xe001','0xe004','0xfa16','0xfa1b'],'cancel_events':['0xe002','0xfa17'],'close_event':'0xf022',
  'format_parser':{'entry':'0x88b488b4','default_limit':1024,'default_mask':63,'default_mode':1,'tokens':{'A/a':[16,16],'N':[8,8],'n/X/x':[24,16],'M/m':[63,1],'unknown':[63,4]},'digits':'accumulate uint32 decimal across format bytes; star replaces accumulator with 1024; final zero or >1024 becomes 1024; masks OR except unknown resets mask'},
  'constructors':{'wml':'0x88b48cfa','xhtml':'0x88b48e48','maxlength':'positive attribute overrides format-derived limit, min(attribute,1024)','password':'edit_type=4, otherwise 1; no format pointer and password sets mask=56, mode=16','initial_value':'copy min(UTF16_length, limit) units','xhtml_format_getter':'0x88629bc2 returns zero; XHTML uses default parser branch'},
  'confirmation':'WML: validate context and layout, read control, setter 629764, close. XHTML: same context checks, bytes +8f/+90 block commit; otherwise compare via 629848, setter 6297c0, then changed value calls 629858 -> event 7 if callback exists. All reviewed confirm paths close, including invalid/guarded context.',
  'cancel':'close editor without calling value setter; f022 calls window-context cleanup 8838d8a8',
  'unknown':['semantic names of XHTML +8f/+90 guards','full JS callback effects','complete form serialization and validation','all allocation failures']},
 'browser_progress':{'regions':[region(0xb65986,0xb659d4),region(0xb62c68,0xb62cc6)],
  'page_received':'store payload[0] as total media count; reset current media and byte progress; clear request handle; if total==0 clear loading and set stopped parameters',
  'media_received':'increment current. If current<total, display(current+total,2*total); otherwise clear loading and stop parameters/UI via 0x88b62c02.',
  'page_stopped':'query request state; set stopped parameters. States 1/2/3 preserve current loading, others clear it and refresh stopped UI. Do not turn every stop notification into cancellation of the latest request.'},
 'music_back':{'regions':[region(0xacf6f6,0xacf752),region(0x600ea6,0x600ec6),region(0x3f12a0,0x3f12b8),region(0x3f25fc,0x3f2696)],
  'order':'call 61999c(0). If main player state !=0 AND review-open AND review state in {0,1}: call 601092(0), 3f25fc(), 3f2646(), 601092(1). Then if ab8438 context flag true close current; else close e0017 if present and open descriptor 8820311c.',
  'review_name_evidence':'native string at user+3f154c: MMIAP_IsReviewOpen',
  'limits':'Review-helper media effects and ab8438 flag meaning remain unresolved. Numeric effect plan is recovered; unconditional continue-playing is still a replica policy.'}}
 data['browser_select']={
  'regions':[region(0xb482c2,0xb488b4),region(0x6296ea,0x629754),region(0x629878,0x629898)],
  'handlers':{'wml':'0x88b48350','xhtml':'0x88b487c4'},'control':'0x60064',
  'multiple_getters':{'wml':{'entry':'0x886296fa','field':'+0x38'},'xhtml':{'entry':'0x88629738','field':'+0x7e'}},
  'confirm':'e001/e004/fa16/fa1b: multiple stays open, single commits widget selection and closes. XHTML dispatches callback event 7 only if flags changed.',
  'back':'e002/fa17: single closes without commit; multiple commits and closes. XHTML multi Back does NOT call 629878 change dispatch.',
  'invalid_context':'missing object or invalid layout closes without commit; close f022 frees window context',
  'xhtml_commit':'b48534 iterates all options; write option+5a from widget item checked flag. If single, each checked item writes select+70=index (last checked wins if inconsistent state). changed is OR of old/new differences. Always call 62974c -> bfd58 after loop.',
  'wml_commit':'b482c2 writes option+4 from widget. Single selected option goes to affaee; otherwise affb92. WML task/value side effects remain delegated.',
  'limits':['generic widget toggling/navigation occurs before these window handlers','DOM/JS callback execution and refresh internals not emulated']}
 data['browser_form_actions']={
  'regions':[region(0xbed86,0xbf088),region(0xbea84,0xbebc6)],
  'input_type_table':{'offset':'0xbee1c','targets':[hex(BASE+0xbee1c+2*x) for x in b[0xbee1c:0xbee26]],'interpretation':{'0/1':'text/password editor','2':'checkbox','3':'radio','4':'submit with callback and validation','5':'reset','6/8':'return','7':'submit flag=1','9':'file picker'}},
  'checkbox':'type2 with guards node+8f/+90 zero toggles node+8e BEFORE click callback node+24 event3; callback false or changed type returns without local rollback. Passed checks redraw via bf06a.',
  'radio':'guards +8f/+90; iterate form list or layout list, candidate tag must equal 0x000a0255 and type3; compare names using b276a6/b27782 then clear selected peer flags, select current and redraw. Exact string comparison semantics/DOM lifetime remain unresolved.',
  'submit':'type4 checks form callback+130 (event3), then walks linked form controls. Type0/1 validation uses null-value guard at input+114 and helper bebc6; failure b46656. Non-input or other input type takes beff0 submit exit, so do not claim exhaustive validation of every later field. Submit b5562a(form+4c,layout,node,flag=0). Type7 submits flag=1 directly.',
  'reset':'type5 checks form callback+134 event3; if permitted calls bea84(form+4c,layout), clears layout+21a and redraws. Text/password free current value and copy default from node+68 to node+60; checkbox/radio copy default node+91 to checked+8e. Select guarded by +7d, options copy +5c to +5a and checked options set select+70=index. If nonmultiple, size<=1 and index==0, set first option selected. This is exact numeric fallback, not inferred DOM-standard behavior.',
  'limits':['full encoding/HTTP request body generation at b5562a not decoded','validator bebc6 and JS callback effects not fully reconstructed','file picker results and allocation failure recovery incomplete']}
 data['sms_send_storage']={
  'regions':[region(0x3c3986,0x3c3a76),region(0x3c435e,0x3c44a0)],
  'fields':{'+4':'dual_sys','+8':'is_finished byte','+c':'send_result','+10':'send_status','+14':'save_storage','+18':'save_record_id u16','+1c':'send_content, first byte is_need_save','+1030':'update_dest_addr','+14c0':'current operation','+1508':'backsend_status'},
  'names':'B: fixed public mmisms_send.c corroborates native calls/offset usage. Numeric branches and stores are A.',
  'success_entry':'0x883c3986',
  'wait':'need_save != 0 and save_storage == 0 -> send_status=1 and return without advancing segment or destination',
  'saved':'need_save != 0 and save_storage != 0 -> b51c90(dual,storage,id,4,1,0,0,ctx,5), backsend_status=5, send_result=0, storage=0; then continue',
  'continue':'next segment first: sequence uint8(current+1) -> 3c3296. Otherwise clear successful recipient, move to next destination if possible, initialize result, call 3c37dc. No destination/move failure/return 4 -> 3c2be2(dual,1,1).',
  'write_continuation_entry':'0x883c435e',
  'write_status':{'1':'finished=1,storage=0; next segment or clear recipient and move to next destination','3/4':'finished=1,storage=0; skip remaining segments and move to next destination','5':'if more segments: set global and current operation=12; b51936 saves next segment. Error 8 opens No-space continuation; other error shows Error and closes send window(0). If no more segments move to next destination.','other':'no continuation'},
  'write_exit':'all normal branches clear send_status=0; missing context returns without store',
  'limits':['this is continuation after write-confirm processing, not the entire modem callback','storage physical writes, complete folder/index logic and all error handlers remain outside these rules','high-level synthetic SmsApp retains explicit policy state and is not a native modem emulator']}
 assert b[0x6000dc:0x6000e1]==bytes.fromhex('03c0c0c0c0')
 data['music_queue']={
  'regions':[region(0x68629c,0x68630a),region(0x600036,0x6002fa),region(0x6002fa,0x600424),region(0x619b68,0x619c10),region(0x61b3c0,0x61b55c),region(0x6010b6,0x6010d2)],
  'settings':'repeat 0/1/2 = Off/All/One, shuffle 0/1; non-boolean values retain native fallback',
  'manual_mode':{'off':[4,1],'all':[3,2],'one':[3,0]},
  'end_mode':{'off':[4,1],'all':[3,2],'one':[0,0]},
  'mode_array_order':'shuffle=0,shuffle=1',
  'callback':'600066 native log MMIAPIMP3_PlayMp3CallBack. Main states 2/3 only; set audio substate=3, update UI. review_flag from copied context+14 suppresses normal queue handling.',
  'end_dispatch_table':{'offset':'0x6000dc','targets':[hex(BASE+0x6000dc+2*x) for x in b[0x6000dc:0x6000e1]]},
  'mode0':'main state 2: guard 600036; if false call 5ff8d8(0) without queue advance, if true call 5fff3a. Main state 3 skips both.',
  'mode1_4':'guard 600036; if false advance via 61b452(0), failure -> 5fff3a; success sends target2 event81, callback result!=2 -> 416a26(0), then main state2 -> 5ff8d8(0).',
  'guard':'600036: nonempty library and (mode2/3: unsupported_count >= count; mode0: unsupported_count !=0). Other modes false. Native count log at 60148c GetNoSupportNumber num=%d.',
  'index':'619b68 supports signed step and wraps modulo count; count=0 fails. Caller 61b452 rejects wrapped index0 when stored mode is1/4, while manual 61b3c0 has different boundary guards at619b96/619bd8. Stored mode3 wrap triggers619a78.',
  'final':'notify window e000e if present; notify target3/4 event2000. After stop helper and mode0/1/4 call5fffb2.',
  'limits':['helper addresses preserve unresolved media side effects','shuffle permutation generation and saved ordering not yet decoded','synchronous callbacks may change state; pure model uses stable snapshots and cannot prove reentrancy behavior','no real audio/handset acceptance']}
 (O/'interaction-core-rules.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
 print('SMS UI counter/config; browser input format/commit/progress; guarded music Back: byte anchors verified')
if __name__=='__main__':main()
