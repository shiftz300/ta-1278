#!/usr/bin/env python3
"""Small, manually reconstructed rules from TA-1278 17.00.17.02 system code.
Not a CPU emulator. No device, filesystem, network or firmware execution.
Only the stated function boundaries are modeled; see apps/native-algorithms.md.
"""
RETRY_CAUSES=frozenset((11,17,21,22,27,36,38,41,42,47,61,81,95,96,97,98,99,111,255))
# Literal comparisons at 3a483a/3a4856/3a4a34/3a4a60/3a4aac.
SMS_R8_ONE=frozenset(map(ord,'ª∞º`'))
SMS_R8_TWO=frozenset(map(ord,'çÁÍÓÚáíóúêÔôÊÀÃÕÂãõâ\x1bŞşıĞğİ'))
SMS_LANG_0=frozenset(map(ord,'ŞşĞğİı'))
SMS_LANG_2=frozenset(map(ord,'ÀÂêÔôÊÃãÕõâº∞ª`'))
SMS_LANG_4=frozenset(map(ord,'ÚúÍíÓóÁá'))
SMS_EXTENSION=frozenset(map(ord,'^{}\\[~]|€')) # native predicate excludes FF
SMS_DEFAULT=frozenset(map(ord,'@£$¥èéùìòÇ\nØø\rÅåΔ_ΦΓΛΩΠΨΣΘΞ\x1bÆæßÉ !"#¤%&\'()*+,-./0123456789:;<=>?¡ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÑÜ§¿abcdefghijklmnopqrstuvwxyzäöñüà'))

def sms_character_counts(text, *, alphabet_management=1, sms_encode=0, convertible=None):
 """3a492c/3a4ade for SMS=true. Defaults are explicit replica settings, not
 the handset's current settings. Management 0 needs the extracted 217-entry
 conversion table membership; it suppresses UCS2 classification, not the text.
 UTF-16 units (including NUL and surrogate halves) are classified independently.
 """
 if alphabet_management==0 and convertible is None:raise ValueError('conversion table required for management 0')
 raw=text.encode('utf-16-le','surrogatepass')
 if len(raw)//2>65535:raise ValueError('native length exceeds uint16')
 counts=dict(ucs2_count=0,default_ext=0,r8_one=0,r8_two=0)
 lang=[0]*5
 for i in range(0,len(raw),2):
  c=raw[i]|raw[i+1]<<8
  ucs2=c>127 and c not in SMS_R8_TWO|SMS_R8_ONE|SMS_DEFAULT|SMS_EXTENSION
  if alphabet_management==0 and c in convertible:ucs2=False
  if ucs2:counts['ucs2_count']+=1
  elif c in SMS_R8_TWO:counts['r8_two']+=1
  elif c in SMS_R8_ONE:counts['r8_one']+=1
  elif c in SMS_EXTENSION and sms_encode!=2:counts['default_ext']+=1
  if c in SMS_LANG_0:lang[0]+=1
  elif c in SMS_LANG_2:lang[2]+=1
  elif c in SMS_LANG_4:lang[4]+=1
 return {'length':len(raw)//2,**counts,'language_counts':tuple(lang)}

def sms_uses_ucs2(ucs2_count, language_counts=(0,0,0,0,0)):
 """880b0754 numeric predicate; callers supply classified character counts.
 Language fields are kept in native +250,+252,+254,+256,+258 order.
 This does not decode characters, T9 dictionaries, or language configuration.
 """
 a,b,c,d,e=language_counts
 if ucs2_count:return True
 if a:return bool(b or c or e)
 if b:return bool(c)
 if c:return bool(d)
 return False

def sms_ui_count(length, *, ucs2_count=0, default_ext=0, r8_one=0, r8_two=0,
                 language_counts=(0,0,0,0,0), pages=None):
 """351b12 + 352296 + 352232: SMS widget count (not send segmentation).
 pages holds (single,multi) for ucs2/default/r8_one/r8_two; the default is the
 observed ordinary initializer with setting outside {1,2} and no shift flags.
 Does not model the optional phonebook-encoding post-adjustment at 351bb2.
 """
 if any(not 0<=v<=65535 for v in (length,ucs2_count,default_ext,r8_one,r8_two,*language_counts)):
  raise ValueError('native counter fields must be uint16')
 pages=pages or {'ucs2':(70,67),'default':(160,153),'r8_one':(155,149),'r8_two':(155,146)}
 if sms_uses_ucs2(ucs2_count,language_counts):
  used=length;left_kind=page_kind='ucs2'
 else:
  used=(length+default_ext+r8_two)&65535
  left_kind='r8_two' if r8_two else 'r8_one' if r8_one else 'default'
  # The native page-number and remaining-number functions differ here.
  page_kind=('r8_two' if any(language_counts[i] for i in (0,2,3)) else 'r8_one') if r8_two else left_kind
 single,multi=pages[left_kind]
 remaining=((single-used)&65535) if used<=single or multi==0 else (multi-used%multi)%multi
 single,multi=pages[page_kind]
 count=1 if used<=single or multi==0 else (used-1)//multi+1
 return {'units':used,'remaining':remaining,'segments':count,'text':f'{remaining}/{count}',
         'remaining_category':left_kind,'page_category':page_kind}

def browser_input_format(value):
 """88b488b4: format byte parser, masks kept numeric (not guessed IM names).
 The parser accumulates all decimal digits, '*' sets 1024, unknown bytes reset
 masks to 0x3f/4. It is not a regex or a validator of the eventual field text.
 """
 value=(value or b'').split(b'\0',1)[0]
 if not value:return {'limit':1024,'mask':0x3f,'mode':1}
 count=mask=mode=0
 for c in value:
  if c==ord('*'):count=1024
  elif c in b'Aa':mask|=0x10;mode=0x10
  elif c==ord('N'):mask|=8;mode=8
  elif c in b'nXx':mask|=0x18;mode=0x10
  elif c in b'Mm':mask|=0x3f;mode=1
  elif 48<=c<=57:count=(count*10+c-48)&0xffffffff
  else:mask=0x3f;mode=4
 if not count or count>1024:count=1024
 if not mask or not mode:mask=0x3f;mode=1
 return {'limit':count,'mask':mask,'mode':mode}

def browser_field_config(kind='xhtml', *, format_bytes=None, maxlength=0, password=False):
 """b48cfa WML / b48e48 XHTML constructors. XHTML format getter returns 0.
 Explicit positive maxlength overrides the parsed limit, bounded to 1024.
 """
 if kind not in ('wml','xhtml') or not 0<=maxlength<=0xffffffff:raise ValueError('field arguments')
 fmt=format_bytes if kind=='wml' else None
 result=browser_input_format(fmt)
 if maxlength:result['limit']=min(maxlength,1024)
 result['edit_type']=4 if password else 1
 if password and fmt is None:result.update(mask=0x38,mode=0x10)
 return result

def browser_field_confirm(kind, *, context_valid=True, layout_valid=True,
                          guard_8f=False, guard_90=False, changed=False):
 """Native effect plan only; JS/event execution and allocation are external.
 XHTML guard names are offsets, because full struct semantics are unresolved.
 """
 if kind not in ('wml','xhtml'):raise ValueError('field kind')
 actions=[]
 if context_valid and layout_valid:
  if kind=='wml':actions+=['read-edit-text','set-wml-value']
  elif not guard_8f and not guard_90:
   actions+=['read-edit-text','compare-old-value','set-xhtml-value']
   if changed:actions+=['dispatch-change-event-7-if-callback']
 return actions+['close-editor']

def browser_media_progress(current,total):
 """b62c68, after incrementing current by one; wraps like unsigned 32-bit.
 Returns the native progress arguments, not a percentage or timing estimate.
 """
 current=(current+1)&0xffffffff;total&=0xffffffff
 if current<total:return {'current':current,'display':((current+total)&0xffffffff,(2*total)&0xffffffff),'stop':False}
 return {'current':current,'display':None,'stop':True}

def browser_select_event(kind, event, multiple, *, context_valid=True, layout_valid=True, changed=False):
 """b48350/b487c4 window handlers, after list-widget processing.
 Multi-select Confirm stays open; Back commits. This is not text-field Cancel.
 changed is the result of comparing widget flags against XHTML option flags.
 """
 if kind not in ('wml','xhtml'):raise ValueError('select kind')
 if event==0xf022:return ['free-window-context']
 confirm=event in (0xe001,0xe004,0xfa16,0xfa1b)
 cancel=event in (0xe002,0xfa17)
 if not (confirm or cancel):return []
 if not (context_valid and layout_valid):return ['close-select']
 if confirm and multiple:return ['keep-select-open']
 actions=[]
 if confirm or multiple:
  actions.append('commit-widget-selection')
  if kind=='xhtml' and confirm and changed:actions.append('dispatch-change-event-7-if-callback')
 return actions+['close-select']

def browser_xhtml_selection(old, checked, multiple, selected_index):
 """b48534: all option flags are written; last checked index wins for single.
 This models even inconsistent widget snapshots without inventing normalization.
 """
 if len(old)!=len(checked):raise ValueError('option count mismatch')
 flags=tuple(bool(x) for x in checked)
 for i,flag in enumerate(flags):
  if flag and not multiple:selected_index=i
 return {'selected':flags,'selected_index':selected_index,
         'changed':any(bool(a)!=b for a,b in zip(old,flags)),
         'refresh':'0x8862974c -> 0x880bfd58'}

def browser_checkbox_click(checked, *, guard_8f=False, guard_90=False, callback_allows=True,
                           type_unchanged=True):
 """bed86: checkbox flag toggles BEFORE onclick; no local rollback on veto.
 Callback arguments and side effects are external; this records only local effects.
 """
 effects=[]
 if not guard_8f and not guard_90:checked=not checked;effects+=['write-checked-before-click']
 effects+=['dispatch-click-event-3-if-callback']
 if callback_allows and type_unchanged and not guard_8f and not guard_90:effects+=['redraw-input']
 return {'checked':bool(checked),'effects':effects}

def browser_select_reset(defaults, selected_index, *, multiple=False, size=1, guard_7d=False):
 """bea84 select branch: copy option+5c defaults into +5a. Preserve the exact
 index==0 fallback; do not reinterpret it as 'no selection'. Allocation/DOM
 failures and renderer refresh are excluded; defaults is an existing option list.
 """
 if guard_7d:return None
 flags=[bool(x) for x in defaults]
 for i,flag in enumerate(flags):
  if flag:selected_index=i
 if not multiple and size<=1 and selected_index==0 and flags:flags[0]=True
 return {'selected':tuple(flags),'selected_index':selected_index}

def _sms_next_destination(next_destination, move_ok, send_result):
 actions=[]
 if next_destination and move_ok:
  actions+=['init-send-save-result','send-next-destination']
  if send_result!=4:return actions
 return actions+['finish(success=1,finished=1)']

def sms_success_plan(*, need_save, saved_storage, next_segment=False, next_destination=False,
                     move_ok=True, send_result=0, sequence=1, update_address_empty=True, context_valid=True):
 """883c3986: successful send confirmation, including write-confirm race.
 Storage and next-send outcomes are supplied; no modem or persistent state.
 Public names corroborate offsets but every branch is checked on this image.
 """
 if not context_valid:return ['ignore-missing-context']
 actions=[]
 if next_segment or next_destination:actions+=['backsend-status=3']
 if need_save:
  if not saved_storage:return actions+['send-status=1','wait-write-confirm']
  actions+=['save-order-record(status=4,backsend=5)','backsend-status=5','send-result=0','save-storage=0']
 if next_segment:return actions+[f'send-segment={(sequence+1)&255}']
 if update_address_empty:actions+=['save-update-address(false)']
 actions+=['clear-successful-destination']
 return actions+_sms_next_destination(next_destination,move_ok,send_result)

def sms_write_confirm_plan(status, *, next_segment=False, next_destination=False, move_ok=True,
                           send_result=0, save_result=0, sequence=1, update_address_empty=True,
                           context_valid=True):
 """883c435e: continuation AFTER write-confirm processing. Not raw write callback.
 Status 5 saves unsent remainder, not retries; errors keep native numeric codes.
 """
 if not context_valid:return ['ignore-missing-context']
 actions=[]
 if status in (1,3,4):actions+=['finished=1','save-storage=0']
 if status==1 and next_segment:actions+=[f'send-segment={(sequence+1)&255}']
 elif status==5 and next_segment:
  actions+=['global-operation=12','current-operation=12',f'save-segment={(sequence+1)&255}']
  if save_result==8:actions+=['show-no-space-with-continuation']
  elif save_result:actions+=['show-error','close-send-window(result=0)']
 elif status in (1,3,4,5):
  if status==1:
   if update_address_empty:actions+=['save-update-address(false)']
   actions+=['clear-successful-destination']
  actions+=_sms_next_destination(next_destination,move_ok,send_result)
 return actions+['send-status=0']

def music_play_mode(repeat, shuffle, *, natural_end=False):
 """68629c/6862d4: persisted repeat/shuffle -> internal playlist mode.
 Repeat 2 deliberately differs between manual navigation and natural end.
 """
 mode=1
 if repeat==0:
  if not shuffle:mode=4
 elif repeat==1:
  if not shuffle:mode=3
  elif shuffle==1:mode=2
 elif repeat==2:mode=0 if natural_end or shuffle else 3
 return mode

def music_index_step(index, count, delta):
 """619b68 arithmetic for valid u16 indices/small signed direction.
 Zero count fails; caller decides stop-on-wrap vs circular traversal.
 """
 if not 0<=index<=65535 or not 0<=count<=65535 or delta not in (-1,0,1):raise ValueError('index arguments')
 if not count:return None
 if delta==0:return index
 return (abs(index+count+delta) if delta<0 else index+delta)%count

def music_end_guard(mode, failures, count):
 """600036: prevent endless retry when all tracks fail; count must be nonzero."""
 return bool(count and ((mode in (2,3) and failures>=count) or (mode==0 and failures!=0)))

def music_end_plan(player_state, mode, *, review_flag=False, failures=0, count=1,
                   advance_ok=True, callback_result=0):
 """600066 PlayMp3CallBack decision/effect plan with stable external snapshot.
 advance_ok comes from 61b452; callback_result from notification 0xc0e2c.
 Helpers kept as addresses where their complete side effects are not recovered.
 """
 if player_state not in (2,3):return []
 actions=['audio-state=3','0x883f300a(1)','0x883f303e(1)',
          '0x88acf8ec-if-0x885ff952']
 ended=False
 if not review_flag:
  actions+=['0x885ffede()']
  if mode==0 and player_state==2:
   if music_end_guard(mode,failures,count):ended=True
   else:actions+=['0x885ff8d8(0)']
  elif mode in (1,2,3,4):
   if music_end_guard(mode,failures,count):ended=True
   else:
    actions+=['advance-queue(direction=0)']
    if not advance_ok:ended=True
    else:
     actions+=['notify(target=2,event=0x81)']
     if callback_result!=2:actions+=['0x88416a26(0)']
     if player_state==2:actions+=['0x885ff8d8(0)']
 if ended:actions+=['0x885fff3a()']
 actions+=['notify-e000e-if-open(event=0x89f)','notify(target=3,event=0x2000)',
           'notify(target=4,event=0x2000)']
 if ended and mode in (0,1,4):actions+=['0x885fffb2()']
 return actions

def music_back_plan(player_state, review_open, review_state, *, context_flag=False, library_window_exists=False):
 """acf6f6..acf750. Does not invent media effects of the two review helpers.
 Service state queries are supplied as a stable snapshot by the caller.
 """
 actions=['0x8861999c(0)']
 if player_state!=0 and review_open and review_state in (0,1):
  actions+=['0x88601092(0)','0x883f25fc()','0x883f2646()','0x88601092(1)']
 if context_flag:actions+=['close-current-window']
 else:
  if library_window_exists:actions+=['close-window-0xe0017']
  actions+=['open-descriptor-0x8820311c']
 return actions

def sms_capacity(alphabet, extra_header=False, language_shift=False, single_shift=False):
 """Return (single, multipart) encoded units from 88b50d34.
 Alphabet 0 uses unpacked default-alphabet units; 1/2 use bytes.
 Flags correspond to r3 and context+2c/+2d; not UI settings.
 """
 if alphabet==0:
  if extra_header:return 154,146
  if language_shift:return 155,146 if single_shift else 149
  return 160,153
 if alphabet==1:return (133,127) if extra_header else (140,134)
 if alphabet==2:return (133,128) if extra_header else (140,134)
 raise ValueError('unresolved native alphabet')

def sms_segment_size(remaining, capacity, alphabet):
 """88b50ea2..88b50ed0: one encoded segment, not UI character counting.
 Caller chooses capacity/offset. No UDH construction, GSM packing or charset conversion.
 """
 if alphabet not in (0,1,2) or capacity<=0:raise ValueError('invalid segment arguments')
 n=min(len(remaining),capacity)
 if alphabet in (0,1) and n and remaining[n-1]==0x1b:n-=1
 return n

def sms_retry_decision(cause, rl_cause, retries):
 """Decision portion of 883c407e. Output effects, not a modem service.
 0: invalid entry to error handler; 4/9 bypass retry; otherwise use cause allowlist.
 Native initial retry budget and save-completion race remain external state.
 """
 if not 0<=retries<=255:raise ValueError('native retry byte out of range')
 if cause==0:return {'action':'ignore-success-in-error-handler','retries':retries}
 if cause in (4,9):return {'action':'finish-without-retry','retries':retries,'finish_flag':False,'send_result':2}
 if retries and rl_cause in RETRY_CAUSES:
  return {'action':'retry-continuation','retries':retries-1,'send_result':2,'send_status':2}
 return {'action':'finish-without-retry','retries':retries,'finish_flag':True,'send_result':2}

def browser_address_bytes(text):
 """b65c54 -> 3a51dc: bounded UTF-16-to-byte copy, NOT UTF-8/IDNA.
 With the zeroed destination in this caller, copying ends at NUL or a code unit >255.
 """
 raw=text.encode('utf-16-le','surrogatepass');out=bytearray()
 for i in range(0,min(len(raw),512*2),2):
  code=raw[i]|raw[i+1]<<8
  if code==0 or code>255:break
  out.append(code)
 return bytes(out)

def browser_url_invalid(value):
 """883b4724 precheck only; deliberately preserves the first-byte check.
 Later parser/network validation is outside this recovered function.
 """
 if value is None:return True
 value=value.split(b'\0',1)[0]
 return not value or value[0]>127 or b'\r' in value or b'\n' in value

def browser_submit_address(text):
 value=browser_address_bytes(text)
 if browser_url_invalid(value):return {'action':'show-url-error','resource':'0x40000872','keep_editor':True}
 return {'action':'start-web','entry':1,'url_bytes':value,'close_editor':True}

def music_center_dispatch(player_state,audio_state=None,enabled=True):
 """Callback chain 68ac44 -> acec7a -> 3f125a -> 3f120c.
 Integer states are retained; semantic names only where native logs support them.
 """
 if not enabled:return None
 if player_state==0:return 'delay-play'
 if player_state==1:
  if audio_state==1:return 'prepare-and-pause'
  if audio_state==2:return 'resume'
  return None
 if player_state==2:return 'pause'
 if player_state==3:return 'resume'
 return None
