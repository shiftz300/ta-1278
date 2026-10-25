#!/usr/bin/env python3
"""Deterministic replica contract, NOT recovered Nokia source or a firmware emulator.
Native-supported transitions and deliberate replica policies are separated in
analysis/ui/apps/{sms,browser,music}-contract.md. All service data is synthetic
and injected; this module has no filesystem, network, SIM, USB or audio I/O.
"""
from dataclasses import dataclass,field
from typing import Optional
from native_app_rules import (sms_ui_count,sms_character_counts,sms_uses_ucs2,browser_field_config,browser_field_confirm,
 browser_select_event,browser_xhtml_selection)

def units(text):
 b=text.encode('utf-16-le','surrogatepass')
 return [b[i]|b[i+1]<<8 for i in range(0,len(b),2)]
def from_units(buf):
 return bytes(v for c in buf for v in (c&255,c>>8)).decode('utf-16-le','surrogatepass')

@dataclass
class Editor:
 """UTF-16 offsets. Multi-tap table and timeout event supplied by caller.
 Vertical moves need visual line spans from the renderer, not split('\n').
 Unknown native paired-range deletion is intentionally a separate policy hook.
 """
 buf:list=field(default_factory=list)
 cursor:int=0
 anchor:Optional[int]=None
 pending:Optional[tuple]=None
 @property
 def text(self):return from_units(self.buf)
 def commit(self):self.pending=None
 def select(self,a,b):
  self.commit()
  if not 0<=a<=len(self.buf) or not 0<=b<=len(self.buf):raise ValueError('selection bounds')
  self.anchor=a;self.cursor=b
 def replace(self,text):
  self.commit();a,b=sorted((self.cursor,self.anchor if self.anchor is not None else self.cursor))
  value=units(text);self.buf[a:b]=value;self.cursor=a+len(value);self.anchor=None
 def backspace(self):
  self.commit()
  if self.anchor is not None and self.anchor!=self.cursor:self.replace('')
  elif self.cursor:self.anchor=self.cursor-1;self.replace('')
 def move(self,delta):
  self.commit();self.cursor=max(0,min(len(self.buf),self.cursor+delta));self.anchor=None
 def vertical(self,direction,line_spans):
  self.commit()
  current=next((i for i,(a,b) in enumerate(line_spans) if a<=self.cursor<=b),None)
  if current is None:raise ValueError('cursor absent from renderer line spans')
  target=current+direction
  if 0<=target<len(line_spans):
   column=self.cursor-line_spans[current][0];a,b=line_spans[target];self.cursor=min(a+column,b)
  self.anchor=None
 def tap(self,slot,rows):
  candidates=rows[slot]
  if not candidates:return
  if self.pending and self.pending[0]==slot:
   _,idx,a,b=self.pending;idx=(idx+1)%len(candidates);self.select(a,b)
  else:
   self.commit();idx=0;a=min(self.cursor,self.anchor if self.anchor is not None else self.cursor)
  self.replace(candidates[idx]);self.pending=(slot,idx,a,self.cursor)

# Native SMS classification and widget arithmetic. The replica chooses
# alphabet_management=1 and sms_encode=0; current handset settings are unknown.
def sms_counter(text):
 counts=sms_character_counts(text,alphabet_management=1,sms_encode=0)
 encoding='UTF16' if sms_uses_ucs2(counts['ucs2_count'],counts['language_counts']) else 'GSM7'
 return {**sms_ui_count(**counts),'encoding':encoding,'policy':'A-classification-and-counter/P-selected-settings'}

@dataclass
class Message:
 id:int
 box:str
 recipients:list
 text:str
 unread:bool=False
 status:dict=field(default_factory=dict)

class SmsApp:
 """Mailbox persistence is an in-memory contract; adapter supplies real storage.
 Native anchors: S1 focus-specific menus; S2 overlay Back; S3 close save branch;
 S4 send/SIM path. Mailbox sorting and service-result UI are replica policies.
 """
 def __init__(self):
  self.messages={};self.next_id=1;self.screen='folders';self.box='inbox';self.selected=None
  self.editor=Editor();self.recipients=[];self.focus='recipient';self.overlay=None
  self.draft_id=None;self.active_send=None;self.sequence=0;self.alert=None;self.sim=None
 def add_message(self,box,recipients,text,unread=False):
  i=self.next_id;self.next_id+=1;self.messages[i]=Message(i,box,list(recipients),text,unread);return i
 def list_ids(self,box):return sorted((i for i,m in self.messages.items() if m.box==box),reverse=True)
 def open_message(self,i):
  m=self.messages[i];self.selected=i;self.box=m.box
  if m.box=='drafts':self.compose(m.text,m.recipients,i)
  else:m.unread=False;self.screen='read'
 def compose(self,text='',recipients=(),draft_id=None):
  if self.active_send is not None:raise ValueError('send in progress')
  self.editor=Editor();self.editor.replace(text);self.recipients=list(recipients)
  self.draft_id=draft_id;self.screen='compose';self.overlay=None;self.alert=None;self.focus='recipient'
 def reply(self):self.compose('',self.messages[self.selected].recipients[:1])
 def forward(self):self.compose(self.messages[self.selected].text)
 def remove_selected(self,confirmed):
  if confirmed and self.selected in self.messages:del self.messages[self.selected];self.selected=None;self.screen='list'
 def options(self):
  self.editor.commit();self.overlay='recipient-options' if self.focus=='recipient' else 'body-options';return self.overlay
 def set_focus(self,focus):
  if focus not in ('recipient','body'):raise ValueError('unknown focus')
  self.editor.commit();self.focus=focus
 def center(self,available_sims=(0,),preferred_sim=None):
  if self.focus=='recipient':self.editor.commit();self.overlay='contact-picker';return None
  return self.send(available_sims,preferred_sim)
 def body_action(self,action,payload=None):
  self.editor.commit()
  if action==7:self.overlay='insert-options'
  elif action in (4,5,6):
   if payload is None:self.overlay={4:'smiley-picker',5:'symbol-picker',6:'template-picker'}[action]
   else:self.editor.replace(payload);self.overlay=None;self.focus='body'
  elif action==36:self.editor.select(0,len(self.editor.buf));self.editor.replace('');self.overlay=None
  elif action==32:self.overlay=None;self.save_draft();self.screen='list'
  elif action in (8,11,12,23):return {'adapter_required':'attachment','native_action':action}
  else:raise ValueError('unresolved body action')
  return None
 def save_draft(self):
  self.editor.commit()
  if not self.editor.buf:return None
  if self.draft_id in self.messages:
   m=self.messages[self.draft_id];m.text=self.editor.text;m.recipients=list(self.recipients)
  else:self.draft_id=self.add_message('drafts',self.recipients,self.editor.text)
  return self.draft_id
 def back(self):
  self.editor.commit()
  if self.overlay:self.overlay=None;return 'dismiss-overlay'
  if self.screen=='compose':self.save_draft();self.screen='list';return 'close-compose'
  if self.screen=='sending':return 'send-in-progress' # P-SMS-SERVICE: avoid ambiguous duplicate sends
  if self.screen=='read':self.screen='list'
  elif self.screen=='list':self.screen='folders'
  else:self.screen='home'
  return self.screen
 def send(self,available_sims=(0,),preferred_sim=None):
  if self.screen!='compose':return None
  self.editor.commit();self.alert=None
  if not self.recipients:self.focus='recipient';self.alert='recipient-required';return None
  if not self.editor.buf:self.focus='body';self.alert='body-required';return None
  # A deliberate local validation policy, not recovered network address grammar.
  if any(not r.lstrip('+').isdigit() for r in self.recipients):self.alert='invalid-recipient';return None
  if not available_sims:self.alert='no-sim';return None
  if preferred_sim not in available_sims and len(available_sims)>1:self.overlay='sim-picker';return None
  self.sim=preferred_sim if preferred_sim in available_sims else available_sims[0]
  self.sequence+=1;self.active_send=self.sequence;self.screen='sending';self.overlay=None
  self.pending_snapshot=(list(self.recipients),self.editor.text)
  return {'operation':self.active_send,'sim':self.sim,'recipients':list(self.recipients),'text':self.editor.text}
 def send_result(self,operation,results):
  if operation!=self.active_send or self.active_send is None:return False
  recipients,text=self.pending_snapshot
  if set(results)!=set(recipients):raise ValueError('need one result per recipient')
  ok=[r for r in recipients if results[r]=='sent'];bad=[r for r in recipients if results[r]!='sent']
  if ok:self.add_message('sent',ok,text)
  if self.draft_id in self.messages:del self.messages[self.draft_id]
  self.draft_id=None
  if bad:
   self.add_message('outbox',bad,text);self.alert='partial-failure' if ok else 'send-failed'
  else:self.alert='sent'
  self.active_send=None;self.screen='list';self.box='outbox' if bad else 'sent';return True

@dataclass
class Page:
 url:str
 title:str=''
 nodes:list=field(default_factory=list)
 focus:Optional[str]=None
 scroll:int=0

class BrowserApp:
 """Dorado UI contract. No HTML/JS engine or actual HTTP. Adapter provides Page
 nodes, navigation candidates, field types and request completion events.
 Spatial focus algorithm is deliberately a renderer responsibility.
 """
 def __init__(self):
  self.screen='start';self.history=[];self.index=-1;self.loading=False;self.pending=None
  self.serial=0;self.overlay=None;self.held=None;self.timer=None;self.engine_events=[]
  self.bookmarks=[];self.offline=[];self.home='about:blank';self.error=None;self.field=None
 @property
 def page(self):return self.history[self.index] if self.index>=0 else None
 def navigate(self,url,target=None):
  if not url.strip():return None
  self.serial+=1;self.pending=(self.serial,url,target);self.loading=True;self.error=None;self.screen='page';self.overlay=None
  return {'operation':self.serial,'url':url}
 def complete(self,operation,page=None,error=None):
  if self.pending is None or operation!=self.pending[0]:return False
  _,url,target=self.pending;self.pending=None;self.loading=False
  if error:self.error=error;return True
  page=page or Page(url)
  if target is None:self.history=self.history[:self.index+1]+[page];self.index=len(self.history)-1
  else:self.history[target]=page;self.index=target
  return True
 def back(self):
  if self.overlay=='field-select':return self.select_event(0xfa17)
  if self.overlay:self.overlay=None;self.field=None;return 'dismiss-overlay'
  if self.loading:self.pending=None;self.loading=False;return 'stop'
  if self.index>0:self.navigate(self.history[self.index-1].url,self.index-1);return 'history-back'
  self.overlay='close-current-page';return 'confirm-close'
 def close_answer(self,yes):
  if self.overlay!='close-current-page':return
  self.overlay=None
  if yes:self.release();self.pending=None;self.loading=False;self.screen='start'
 def forward(self):
  if self.index+1<len(self.history):return self.navigate(self.history[self.index+1].url,self.index+1)
 def refresh(self):
  if self.page:return self.navigate(self.page.url,self.index)
 def key_down(self,direction):
  code={'up':14,'down':15,'left':12,'right':13}[direction]
  self.held=direction;self.timer=500;self.engine_events.append(code)
 def repeat_tick(self,focused=True):
  self.timer=None
  if not self.held or not focused:return
  self.engine_events.append({'up':18,'down':19,'left':16,'right':17}[self.held]);self.timer=100
 def release(self):self.held=None;self.timer=None
 def activate(self):
  self.engine_events.append(20)
  node=next((n for n in (self.page.nodes if self.page else []) if n['id']==self.page.focus),None)
  if not node:return None
  kind=node['type']
  if kind=='link':return self.navigate(node['url'])
  if kind in ('text','password','select'):
   self.overlay='field-'+kind;self.field=node
   if kind=='select':self.selection=[bool(x) for x in node.get('selected',[])]
   return 'edit-field'
  if kind=='checkbox':node['checked']=not node.get('checked',False)
  if kind=='submit':return {'form':node['form']} # adapter decides method/encoding; no network in this model
 def save_field(self,value):
  if not self.overlay or not self.overlay.startswith('field-'):raise ValueError('no field')
  node=self.field
  if node['type']=='select':
   if len(value)!=len(self.selection):raise ValueError('option count mismatch')
   self.selection=list(value)
   return self.select_event(0xfa1b)
  else:
   kind=node.get('markup','xhtml')
   config=browser_field_config(kind,format_bytes=node.get('format'),maxlength=node.get('maxlength',0),password=node['type']=='password')
   # The UI should enforce this while editing. Reject invalid adapter input
   # instead of silently truncating a user's edit on confirmation (P).
   if len(units(value))>config['limit']:raise ValueError('field exceeds native edit limit')
   effects=browser_field_confirm(kind,context_valid=node.get('context_valid',True),layout_valid=node.get('layout_valid',True),guard_8f=node.get('guard_8f',False),guard_90=node.get('guard_90',False),changed=value!=node.get('value',''))
   if any(x in effects for x in ('set-wml-value','set-xhtml-value')):node['value']=value
  self.field=None;self.overlay=None
  return effects
 def select_event(self,event):
  if self.overlay!='field-select':raise ValueError('no select')
  node=self.field;kind=node.get('markup','xhtml');multiple=node.get('multiple',False)
  old=node.get('selected',[])
  result=browser_xhtml_selection(old,self.selection,multiple,node.get('selected_index',0))
  effects=browser_select_event(kind,event,multiple,context_valid=node.get('context_valid',True),layout_valid=node.get('layout_valid',True),changed=result['changed'])
  if 'commit-widget-selection' in effects:
   node['selected']=list(result['selected'])
   if kind=='xhtml':node['selected_index']=result['selected_index']
   # WML task/value refresh and XHTML JS execution belong to the adapter.
  if 'close-select' in effects:self.field=None;self.overlay=None
  return effects
 def bookmark(self):
  if self.page:self.bookmarks.append((self.page.title,self.page.url))
 def save_offline(self):
  if self.page:self.offline.append(Page(self.page.url,self.page.title,[dict(n) for n in self.page.nodes]))
 def options(self):self.release();self.overlay='page-menu' if self.screen=='page' else 'start-menu'
 def menu_action(self,action):
  self.release()
  overlays={5:'open-menu',28:'navigation-menu',29:'save-menu',14:'address-editor',8:'bookmarks',9:'history',11:'offline-pages',10:'downloads',23:'page-details',3:'settings',13:'close-current-page'}
  if action in overlays:self.overlay=overlays[action];return None
  self.overlay=None
  if action==15:return self.navigate(self.home)
  if action==16:self.pending=None;self.loading=False
  elif action==17:return self.forward()
  elif action==18:
   if self.index>0:return self.navigate(self.history[self.index-1].url,self.index-1)
  elif action==19:return self.refresh()
  elif action==20:
   if self.page:self.home=self.page.url
  elif action==21:self.bookmark()
  elif action==22:self.save_offline()
  elif action==7:return {'adapter_required':'compose-sms','text':self.page.url if self.page else ''}
  elif action==31:return {'adapter_required':'save-image'}
  else:raise ValueError('unresolved browser action')
  return None

class MusicApp:
 """Press/long/release distinguishes track skip from seek (native M1).
 Queue wrapping, shuffle permutation and seek seconds are explicit policies.
 No codec or audio hardware calls are performed.
 """
 def __init__(self,durations=()):
  self.durations=list(durations);self.queue=list(range(len(durations)));self.index=0
  self.state='stopped';self.position=0;self.volume=5;self.max_volume=10
  self.repeat='off';self.held=None;self.seeking=False;self.screen='player';self.error=None
 @property
 def track(self):return self.queue[self.index] if self.queue else None
 def toggle(self):
  if self.track is None:self.error='empty-library';return
  self.state='paused' if self.state=='playing' else 'playing'
 def down(self,key):
  if key in ('left','right'):self.held=key;self.seeking=False
  elif key in ('up','down'):self.volume=max(0,min(self.max_volume,self.volume+(1 if key=='up' else -1)))
 def long(self,key):
  if self.held==key and self.track is not None and self.state in ('playing','paused'):self.seeking=True
 def seek_tick(self,seconds=5):
  if self.seeking:
   self.position=max(0,min(self.durations[self.track],self.position+seconds*(1 if self.held=='right' else -1)))
 def release(self,key):
  if self.held!=key:return
  if not self.seeking:self.skip(1 if key=='right' else -1)
  self.held=None;self.seeking=False
 def lose_focus(self):self.held=None;self.seeking=False
 def skip(self,direction):
  if not self.queue:return
  target=self.index+direction
  if 0<=target<len(self.queue):self.index=target;self.position=0
  elif self.repeat=='all':self.index=target%len(self.queue);self.position=0
 def finish(self):
  if self.track is None:return
  if self.repeat=='one':self.position=0;return
  if self.index+1<len(self.queue) or self.repeat=='all':self.skip(1)
  else:self.state='stopped';self.position=self.durations[self.track]
 def set_shuffle_order(self,order):
  if sorted(order)!=list(range(len(self.durations))):raise ValueError('queue must be a permutation')
  old=self.track;self.queue=list(order);self.index=self.queue.index(old) if old is not None else 0
 def set_repeat_native(self,value):
  if value not in (0,1,2):raise ValueError('invalid native repeat row')
  self.repeat={0:'off',1:'all',2:'one'}[value]
 def playback_error(self,error):self.error=error;self.state='stopped';self.lose_focus()
 def back(self):self.lose_focus();self.screen='library' # background play is policy, not fully proven native behavior
