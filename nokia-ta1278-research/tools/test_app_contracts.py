#!/usr/bin/env python3
"""Synthetic behavioral regression tests + independent native branch anchors.
Passing means reference contract consistency, not physical handset acceptance.
"""
import json,unittest,io,hashlib,struct
from pathlib import Path
from app_interaction_reference import Editor,SmsApp,BrowserApp,MusicApp,Page,sms_counter,units
from recover_event_routes import run
from native_app_rules import (sms_capacity,sms_segment_size,sms_retry_decision,RETRY_CAUSES,
 browser_address_bytes,browser_url_invalid,browser_submit_address,music_center_dispatch,
 sms_ui_count,sms_uses_ucs2,browser_input_format,browser_field_config,browser_field_confirm,
 browser_media_progress,music_back_plan,browser_select_event,browser_xhtml_selection,
 sms_success_plan,sms_write_confirm_plan,music_play_mode,music_index_step,music_end_guard,music_end_plan,
 browser_checkbox_click,browser_select_reset,sms_character_counts)
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps'
class EditorTests(unittest.TestCase):
 def test_multitap_in_middle_and_commit(self):
  e=Editor();e.replace('XY');e.move(-1);rows=['abc','de','']
  e.tap(0,rows);e.tap(0,rows);self.assertEqual(e.text,'XbY')
  e.commit();e.tap(0,rows);self.assertEqual(e.text,'XbaY')
  e.tap(1,rows);self.assertEqual(e.text,'XbadY')
 def test_selection_reverse_and_backspace(self):
  e=Editor();e.replace('abcde');e.select(4,1);e.backspace();self.assertEqual((e.text,e.cursor),('ae',1))
  e.select(0,0);e.backspace();self.assertEqual(e.text,'ae')
 def test_pending_replacement_of_selection(self):
  e=Editor();e.replace('01234');e.select(1,4);e.tap(0,['ab']);e.tap(0,['ab']);self.assertEqual(e.text,'0b4')
 def test_utf16_offsets(self):
  e=Editor();e.replace('中😀A');self.assertEqual(e.cursor,4)
  e.select(1,3);e.replace('文');self.assertEqual(e.text,'中文A')
 def test_vertical_uses_renderer_lines(self):
  e=Editor();e.replace('abcd\nxy\n1234');e.cursor=3;e.vertical(1,[(0,4),(5,7),(8,12)])
  self.assertEqual(e.cursor,7)
 def test_counter_alphabet_boundary(self):
  self.assertEqual(sms_counter('A'*160)['segments'],1)
  self.assertEqual(sms_counter('A'*161)['remaining'],145)
  self.assertEqual(sms_counter('中'*71)['remaining'],63)
  self.assertEqual(sms_counter('€')['units'],2)
 def test_ui_counter_is_not_wire_escape_segmentation(self):
  x=sms_counter('A'*152+'€'+'A'*152)
  self.assertEqual((x['units'],x['segments'],x['remaining'],x['text']),(306,2,0,'0/2'))
  self.assertEqual(sms_segment_size(b'A'*152+b'\x1b\x65'+b'A'*152,153,0),152)
 def test_selection_invariants(self):
  for a in range(6):
   for b in range(6):
    e=Editor();e.replace('12345');e.select(a,b);e.replace('XY')
    self.assertEqual(e.text,'12345'[:min(a,b)]+'XY'+'12345'[max(a,b):])
    self.assertEqual(e.cursor,min(a,b)+2)
class SmsTests(unittest.TestCase):
 def prepared(self):
  s=SmsApp();s.compose('测试', ['10000']);return s
 def test_back_dismisses_overlay_before_draft_save(self):
  s=self.prepared();s.overlay='recipient-candidates';self.assertEqual(s.back(),'dismiss-overlay')
  self.assertEqual(s.screen,'compose');self.assertFalse(s.messages)
  s.back();self.assertEqual(len(s.list_ids('drafts')),1)
 def test_empty_compose_no_draft(self):
  s=SmsApp();s.compose();s.back();self.assertFalse(s.messages)
 def test_resumed_draft_replaces_existing(self):
  s=self.prepared();i=s.save_draft();s.back();s.open_message(i);s.editor.replace('!');s.back()
  self.assertEqual(s.messages[i].text,'测试!');self.assertEqual(len(s.messages),1)
 def test_focus_specific_options(self):
  s=self.prepared();self.assertEqual(s.options(),'recipient-options');s.focus='body';self.assertEqual(s.options(),'body-options')
 def test_native_body_actions_insert_clear_save(self):
  s=self.prepared();s.set_focus('body');s.options();s.body_action(7);s.body_action(5);self.assertEqual(s.overlay,'symbol-picker')
  s.body_action(5,'!');self.assertEqual(s.editor.text,'测试!');s.body_action(36);self.assertEqual(s.editor.text,'')
  s.body_action(6,'模板');s.body_action(32);self.assertEqual(s.screen,'list');self.assertEqual(s.messages[s.draft_id].text,'模板')
 def test_center_recipient_does_not_send(self):
  s=self.prepared();self.assertIsNone(s.center());self.assertEqual(s.overlay,'contact-picker');self.assertIsNone(s.active_send)
  s.back();s.set_focus('body');self.assertIsNotNone(s.center())
 def test_send_validation_and_sim_selection(self):
  s=SmsApp();s.compose('x');self.assertIsNone(s.send());self.assertEqual(s.focus,'recipient')
  s.recipients=['+10000'];self.assertIsNone(s.send(()));self.assertEqual(s.alert,'no-sim')
  self.assertIsNone(s.send((0,1)));self.assertEqual(s.overlay,'sim-picker')
  request=s.send((0,1),1);self.assertEqual(request['sim'],1)
 def test_partial_send_never_resends_successful_recipient(self):
  s=self.prepared();s.recipients=['10000','10001'];token=s.send()['operation']
  self.assertIsNone(s.send());self.assertEqual(s.back(),'send-in-progress')
  s.send_result(token,{'10000':'sent','10001':'network-error'})
  self.assertEqual(s.messages[s.list_ids('outbox')[0]].recipients,['10001'])
  self.assertEqual(s.messages[s.list_ids('sent')[0]].recipients,['10000'])
  self.assertFalse(s.send_result(token,{'10000':'sent','10001':'sent'}))
 def test_receive_read_reply_forward_delete(self):
  s=SmsApp();i=s.add_message('inbox',['10000'],'Hello',True);s.open_message(i)
  self.assertFalse(s.messages[i].unread);s.reply();self.assertEqual((s.recipients,s.editor.text),(['10000'],''))
  s.open_message(i);s.forward();self.assertEqual((s.recipients,s.editor.text),([],'Hello'))
  s.open_message(i);s.remove_selected(False);self.assertIn(i,s.messages);s.remove_selected(True);self.assertNotIn(i,s.messages)
class BrowserTests(unittest.TestCase):
 def test_multiselect_back_commits_after_confirm_stays(self):
  b=self.loaded();node={'id':'s','type':'select','multiple':True,'selected':[True,False]}
  b.page.nodes=[node];b.page.focus='s';b.activate()
  self.assertEqual(b.save_field([False,True]),['keep-select-open'])
  self.assertEqual(node['selected'],[True,False]);self.assertEqual(b.overlay,'field-select')
  self.assertEqual(b.back(),['commit-widget-selection','close-select'])
  self.assertEqual(node['selected'],[False,True]);self.assertIsNone(b.overlay)
 def loaded(self):
  b=BrowserApp();r=b.navigate('https://example.invalid/a');b.complete(r['operation'],Page(r['url'],'A'));return b
 def test_back_stop_history_close_are_distinct(self):
  b=self.loaded();r=b.navigate('https://example.invalid/b');self.assertEqual(b.back(),'stop')
  self.assertFalse(b.complete(r['operation']));self.assertEqual(b.page.title,'A')
  r=b.navigate('https://example.invalid/b');b.complete(r['operation']);self.assertEqual(b.back(),'history-back')
  b.complete(b.pending[0]);self.assertEqual(b.index,0);self.assertEqual(b.back(),'confirm-close')
  b.close_answer(False);self.assertEqual(b.screen,'page');b.back();b.close_answer(True);self.assertEqual(b.screen,'start')
 def test_stale_result_does_not_overwrite_new_navigation(self):
  b=self.loaded();old=b.navigate('old')['operation'];new=b.navigate('new')['operation']
  self.assertFalse(b.complete(old));b.complete(new);self.assertEqual(b.page.url,'new')
 def test_history_branch_discards_forward_entries(self):
  b=self.loaded();b.complete(b.navigate('B')['operation']);b.back();b.complete(b.pending[0]);b.complete(b.navigate('C')['operation'])
  self.assertEqual([p.url for p in b.history],['https://example.invalid/a','C']);self.assertIsNone(b.forward())
 def test_failure_preserves_previous_page(self):
  b=self.loaded();b.complete(b.navigate('bad')['operation'],error='network');self.assertEqual(b.page.title,'A');self.assertEqual(b.error,'network')
 def test_direction_codes_and_repeat_cancel(self):
  for key,initial,repeated in [('up',14,18),('down',15,19),('left',12,16),('right',13,17)]:
   b=self.loaded();b.key_down(key);self.assertEqual(b.timer,500);b.repeat_tick();self.assertEqual(b.timer,100)
   b.release();b.repeat_tick();self.assertEqual(b.engine_events,[initial,repeated])
 def test_field_cancel_link_and_checkbox(self):
  b=self.loaded();b.page.nodes=[{'id':'q','type':'text','value':'old'},{'id':'c','type':'checkbox'},{'id':'l','type':'link','url':'B'}]
  b.page.focus='q';b.activate();self.assertEqual(b.back(),'dismiss-overlay');self.assertEqual(b.page.nodes[0]['value'],'old')
  b.activate();b.save_field('new');self.assertEqual(b.page.nodes[0]['value'],'new')
  b.page.focus='c';b.activate();self.assertTrue(b.page.nodes[1]['checked']);b.page.focus='l';self.assertEqual(b.activate()['url'],'B')
 def test_bookmark_and_offline_copy(self):
  b=self.loaded();b.page.nodes=[{'id':'q','type':'text','value':'a'}];b.bookmark();b.save_offline();b.page.nodes[0]['value']='b'
  self.assertEqual(b.offline[0].nodes[0]['value'],'a');self.assertEqual(b.bookmarks[0][0],'A')
 def test_native_menu_actions(self):
  b=self.loaded();b.options();b.menu_action(5);self.assertEqual(b.overlay,'open-menu');b.menu_action(14);self.assertEqual(b.overlay,'address-editor')
  b.back();b.menu_action(20);self.assertEqual(b.home,b.page.url);b.menu_action(21);self.assertEqual(len(b.bookmarks),1)
  self.assertEqual(b.menu_action(7)['text'],b.page.url);b.navigate('pending');b.menu_action(16);self.assertFalse(b.loading)
class MusicTests(unittest.TestCase):
 def test_short_press_skips_only_on_release(self):
  p=MusicApp([100,200]);p.toggle();p.down('right');self.assertEqual(p.track,0);p.release('right');self.assertEqual(p.track,1)
 def test_long_seek_does_not_skip_on_release(self):
  p=MusicApp([100,200]);p.toggle();p.down('right');p.long('right');p.seek_tick();p.release('right')
  self.assertEqual((p.track,p.position),(0,5));self.assertFalse(p.seeking)
 def test_seek_bounds_and_focus_loss(self):
  p=MusicApp([3]);p.toggle();p.down('right');p.long('right');p.seek_tick();self.assertEqual(p.position,3)
  p.lose_focus();p.seek_tick();self.assertFalse(p.seeking)
 def test_repeat_modes(self):
  p=MusicApp([10,20]);p.toggle();p.set_repeat_native(2);p.finish();self.assertEqual((p.track,p.position),(0,0))
  p.repeat='off';p.finish();self.assertEqual(p.track,1);p.finish();self.assertEqual(p.state,'stopped')
  p.repeat='all';p.toggle();p.finish();self.assertEqual(p.track,0)
 def test_shuffle_preserves_track_and_rejects_missing_tracks(self):
  p=MusicApp([10,20,30]);p.set_shuffle_order([2,0,1]);self.assertEqual(p.track,0)
  with self.assertRaises(ValueError):p.set_shuffle_order([0,0,1])
 def test_volume_and_error(self):
  p=MusicApp([10]);p.toggle()
  for _ in range(40):p.down('up')
  self.assertEqual(p.volume,p.max_volume);p.playback_error('unsupported-codec');self.assertEqual(p.state,'stopped')
 def test_empty_library(self):
  p=MusicApp();p.toggle();p.finish();p.down('left');p.release('left');self.assertEqual(p.error,'empty-library')
class NativeRuleTests(unittest.TestCase):
 def test_sms_ui_single_and_multi_boundaries(self):
  for n,left,pages in [(0,160,1),(160,0,1),(161,145,2),(306,0,2),(307,152,3)]:
   x=sms_ui_count(n);self.assertEqual((x['remaining'],x['segments'],x['text']),(left,pages,f'{left}/{pages}'))
  for n,left,pages in [(0,70,1),(70,0,1),(71,63,2),(134,0,2)]:
   x=sms_ui_count(n,ucs2_count=1);self.assertEqual((x['remaining'],x['segments']),(left,pages))
 def test_native_language_classification_is_not_any_language(self):
  for values,wanted in [((0,0,0,0,0),False),((1,0,0,0,0),False),((1,1,0,0,0),True),((0,1,1,0,0),True),((0,0,1,0,0),False),((0,0,1,1,0),True),((1,0,0,0,1),True)]:
   self.assertEqual(sms_uses_ucs2(0,values),wanted)
  self.assertTrue(sms_uses_ucs2(1))
 def test_native_national_shift_page_left_branch_difference(self):
  x=sms_ui_count(297,r8_two=1)
  self.assertEqual((x['units'],x['remaining'],x['segments']),(298,140,2))
  self.assertEqual((x['remaining_category'],x['page_category']),('r8_two','r8_one'))
  x=sms_ui_count(297,r8_two=1,language_counts=(1,0,0,0,0))
  self.assertEqual(x['segments'],3)
 def test_native_uint16_and_zero_multi_behavior(self):
  x=sms_ui_count(200,pages={'default':(160,0)})
  self.assertEqual((x['remaining'],x['segments']),(65496,1))
  self.assertEqual(sms_ui_count(65535,default_ext=1)['units'],0)
  with self.assertRaises(ValueError):sms_ui_count(65536)
 def test_browser_format_character_masks_and_numeric_limit(self):
  for f,limit,mask,mode in [(None,1024,63,1),(b'12N',12,8,8),(b'3A',3,16,16),(b'10n',10,24,16),(b'*M',1024,63,1),(b'4?',4,63,4),(b'2N3a',23,24,16),(b'12',12,63,1),(b'0N',1024,8,8),(b'4294967297N',1,8,8),(b'8N\0M',8,8,8)]:
   self.assertEqual(browser_input_format(f),dict(limit=limit,mask=mask,mode=mode))
 def test_browser_maxlength_overrides_format_and_password(self):
  self.assertEqual(browser_field_config('wml',format_bytes=b'3N',maxlength=20)['limit'],20)
  self.assertEqual(browser_field_config('xhtml',format_bytes=b'3N')['limit'],1024)
  x=browser_field_config(password=True,maxlength=4096)
  self.assertEqual(x,dict(limit=1024,mask=56,mode=16,edit_type=4))
  # A non-null but empty WML format pointer is distinct from no pointer.
  self.assertEqual(browser_field_config('wml',format_bytes=b'',password=True)['mask'],63)
 def test_browser_commit_guard_and_changed_callback(self):
  self.assertEqual(browser_field_confirm('xhtml',guard_8f=True),['close-editor'])
  self.assertEqual(browser_field_confirm('xhtml',guard_90=True),['close-editor'])
  self.assertEqual(browser_field_confirm('wml',layout_valid=False),['close-editor'])
  self.assertNotIn('dispatch-change-event-7-if-callback',browser_field_confirm('xhtml'))
  self.assertIn('dispatch-change-event-7-if-callback',browser_field_confirm('xhtml',changed=True))
 def test_field_model_preserves_value_on_cancel_guard_or_limit_error(self):
  b=BrowserApp();b.complete(b.navigate('fixture')['operation'],Page('fixture',nodes=[{'id':'q','type':'text','value':'old','maxlength':3}],focus='q'))
  b.activate()
  with self.assertRaises(ValueError):b.save_field('long')
  self.assertEqual(b.field['value'],'old');self.assertEqual(b.overlay,'field-text')
  b.back();self.assertIsNone(b.field);b.page.nodes[0]['guard_90']=True;b.activate();b.save_field('new')
  self.assertEqual(b.page.nodes[0]['value'],'old');self.assertIsNone(b.overlay)
  b.page.nodes[0]['guard_90']=False;b.activate();effects=b.save_field('new')
  self.assertEqual(b.page.nodes[0]['value'],'new');self.assertIn('dispatch-change-event-7-if-callback',effects)
 def test_browser_media_progress_has_document_half(self):
  self.assertEqual(browser_media_progress(0,4),dict(current=1,display=(5,8),stop=False))
  self.assertEqual(browser_media_progress(3,4),dict(current=4,display=None,stop=True))
  self.assertTrue(browser_media_progress(0,0)['stop'])
 def test_music_back_guards_and_window_branch(self):
  for state in range(5):
   for review in (False,True):
    for substate in range(4):
     plan=music_back_plan(state,review,substate)
     self.assertEqual('0x883f25fc()' in plan,state!=0 and review and substate in (0,1))
  self.assertEqual(music_back_plan(0,False,0,context_flag=True),['0x8861999c(0)','close-current-window'])
  self.assertEqual(music_back_plan(0,False,0,library_window_exists=True)[1:],['close-window-0xe0017','open-descriptor-0x8820311c'])
 def test_ui_rule_native_anchors(self):
  b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
  data=json.loads((O/'interaction-core-rules.json').read_text())
  self.assertEqual(data['sms']['ordinary_page_values']['ucs2'],[70,67])
  self.assertEqual(b[0x352308:0x35230e],b'%d/%d\0')
  self.assertEqual(b[0x624148:0x62414a],bytes.fromhex('4008')) # native UCS2 divide by two
  self.assertEqual(b[0x6241a8:0x6241aa],bytes.fromhex('4008'))
  self.assertEqual(struct.unpack_from('<I',b,0xb490e8)[0],0x60065)
 def test_all_capacity_branches_match_rom_immediates(self):
  data=json.loads((O/'sms-encoding-rules.json').read_text())
  for row in data['capacity_rows']:
   self.assertEqual(sms_capacity(row['alphabet'],row['extra_header'],row['language_shift'],row['single_shift']),(row['single'],row['multipart']))
  self.assertEqual(sms_capacity(0,True,True,True),(154,146))
  self.assertEqual(sms_capacity(0,False,False,True),(160,153))
  with self.assertRaises(ValueError):sms_capacity(3)
 def test_ucs2_byte_capacity_not_silently_rounded(self):
  self.assertEqual(sms_capacity(2,True),(133,128))
  self.assertEqual(sms_capacity(1,True),(133,127))
 def test_encoded_segment_escape_boundary(self):
  data=b'A'*152+b'\x1b\x65'+b'A'
  self.assertEqual(sms_segment_size(data,153,0),152)
  self.assertEqual(sms_segment_size(data,153,1),152)
  self.assertEqual(sms_segment_size(data,153,2),153)
  self.assertEqual(sms_segment_size(b'',153,0),0)
  self.assertEqual(sms_segment_size(b'A\x1b',153,0),1)
 def test_retry_allowlist_from_integer_branch_tree(self):
  data=json.loads((O/'sms-retry-rules.json').read_text())
  self.assertEqual(set(data['accepted_rl_causes']),RETRY_CAUSES)
  self.assertEqual(len(RETRY_CAUSES),19)
  for cause in range(256):
   result=sms_retry_decision(1,cause,2)
   self.assertEqual(result['action']=='retry-continuation',cause in RETRY_CAUSES)
 def test_retry_priority_and_exhaustion(self):
  for cause in (4,9):
   self.assertEqual(sms_retry_decision(cause,17,2),{'action':'finish-without-retry','retries':2,'finish_flag':False,'send_result':2})
  self.assertEqual(sms_retry_decision(0,17,2)['action'],'ignore-success-in-error-handler')
  self.assertEqual(sms_retry_decision(1,17,1)['retries'],0)
  self.assertEqual(sms_retry_decision(1,17,0)['action'],'finish-without-retry')
 def test_url_copy_keeps_native_narrowing(self):
  self.assertEqual(browser_address_bytes('http://example.invalid/中文'),'http://example.invalid/'.encode())
  self.assertEqual(browser_address_bytes('中http://x'),b'')
  self.assertEqual(browser_address_bytes('A\0B'),b'A')
  self.assertEqual(browser_address_bytes('A'*513),b'A'*512)
 def test_url_check_is_not_generic_url_validation(self):
  for s in (None,b'',b'\x80abc',b'abc\rdef',b'abc\ndef'):self.assertTrue(browser_url_invalid(s))
  for s in (b'http://',b'plain name',b'A\xff',b'\x7f'):self.assertFalse(browser_url_invalid(s))
  self.assertFalse(browser_url_invalid(b'A\0\n'))
 def test_url_submit_retains_or_closes_editor(self):
  self.assertTrue(browser_submit_address('')['keep_editor'])
  self.assertEqual(browser_submit_address('https://x')['url_bytes'],b'https://x')
  self.assertTrue(browser_submit_address('http://')['close_editor'])
 def test_async_jump_table_and_reference_names(self):
  j=json.loads((O/'browser-async-routes.json').read_text());rows=j['rows']
  self.assertEqual(len(rows),22)
  expect={0x304:0x88b62bb0,0x305:0x88b62bd2,0x306:0x88b62bd2,0x30e:0x88b62b8e,0x30f:0x88b62b96,0x315:0x88b62cd8}
  for msg,dest in expect.items():self.assertEqual(rows[msg-0x300]['target'],hex(dest))
  self.assertEqual(rows[14]['framework_name_B'],'BRW_MSG_TYPE_DISPLAY_READY_IND')
 def test_music_callback_route_recovered(self):
  j=json.loads((O/'event-routes.json').read_text())
  row=next(x for x in j['applications'] if x['name']=='music-view-callback')
  ev=next(x for x in row['nondefault_routes'] if x['event']=='0xfa1b')
  self.assertEqual(ev['boundary_pc'],'0x8868aca6')
  self.assertEqual(len(row['nondefault_routes']),4)
 def test_music_numeric_states_preserve_guards(self):
  self.assertIsNone(music_center_dispatch(2,enabled=False))
  self.assertEqual(music_center_dispatch(0),'delay-play')
  self.assertEqual(music_center_dispatch(1,1),'prepare-and-pause')
  self.assertEqual(music_center_dispatch(1,2),'resume')
  self.assertIsNone(music_center_dispatch(1,0))
  self.assertEqual(music_center_dispatch(2),'pause')
  self.assertEqual(music_center_dispatch(3),'resume')
  self.assertIsNone(music_center_dispatch(4))
 def test_native_rule_regions_and_literal_bytes(self):
  b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
  def walk(x):
   if isinstance(x,dict):
    if all(k in x for k in ('start','end_exclusive','sha256')):
     a=int(x['start'],16)-0x88000000;z=int(x['end_exclusive'],16)-0x88000000
     self.assertEqual(hashlib.sha256(b[a:z]).hexdigest(),x['sha256'])
    for v in x.values():walk(v)
   elif isinstance(x,list):
    for v in x:walk(v)
  for name in ['sms-encoding-rules','sms-retry-rules','browser-address-auth','browser-async-routes','music-center-route','interaction-core-rules']:walk(json.loads((O/(name+'.json')).read_text()))
  self.assertEqual(b[0x3b49d8:0x3b49dc],b'\r\n\0\0')
  self.assertEqual(struct.unpack_from('<I',b,0x68ad84)[0],0x8868ac45)
  self.assertEqual(b[0x3b4742:0x3b4744],bytes.fromhex('2978')) # ldrb r1,[r5], not indexed current byte

class NativeEvidenceTests(unittest.TestCase):
 def test_sms_native_classification_and_language_mix(self):
  c=sms_character_counts('A€Şª中😀\f')
  self.assertEqual((c['length'],c['ucs2_count'],c['default_ext'],c['r8_one'],c['r8_two']),(8,3,1,1,1))
  self.assertEqual(c['language_counts'],(1,0,1,0,0))
  self.assertEqual(sms_counter('Ş')['encoding'],'GSM7')
  self.assertEqual(sms_counter('Şª')['encoding'],'UTF16')
  self.assertEqual(sms_counter('\f')['units'],1) # native extension predicate excludes FF
  self.assertEqual(sms_counter('\x1b')['units'],2) # ESC is R8-two in this build
 def test_sms_conversion_configuration_is_explicit(self):
  tables=json.loads((O/'sms-character-tables.json').read_text())
  convertible={x['unicode'] for x in tables['conversion']['entries']}
  self.assertEqual(len(convertible),217)
  self.assertEqual(sms_character_counts('Ā',alphabet_management=1)['ucs2_count'],1)
  self.assertEqual(sms_character_counts('Ā',alphabet_management=0,convertible=convertible)['ucs2_count'],0)
  self.assertEqual(sms_character_counts('€',sms_encode=2)['default_ext'],0)
  with self.assertRaises(ValueError):sms_character_counts('Ā',alphabet_management=0)
 def test_checkbox_veto_does_not_locally_undo_toggle(self):
  a=browser_checkbox_click(False,callback_allows=False)
  self.assertTrue(a['checked']);self.assertNotIn('redraw-input',a['effects'])
  self.assertEqual(a['effects'][0],'write-checked-before-click')
  self.assertFalse(browser_checkbox_click(False,guard_8f=True)['checked'])
  self.assertNotIn('redraw-input',browser_checkbox_click(True,type_unchanged=False)['effects'])
 def test_select_reset_preserves_native_index_condition(self):
  self.assertEqual(browser_select_reset([0,0],0)['selected'],(True,False))
  self.assertEqual(browser_select_reset([0,0],1)['selected'],(False,False))
  self.assertEqual(browser_select_reset([0,1],0)['selected_index'],1)
  self.assertEqual(browser_select_reset([0,0],0,multiple=True)['selected'],(False,False))
  self.assertIsNone(browser_select_reset([1],0,guard_7d=True))
 def test_select_commit_branches(self):
  for kind in ('wml','xhtml'):
   for event in (0xe001,0xe004,0xfa16,0xfa1b):
    self.assertEqual(browser_select_event(kind,event,True),['keep-select-open'])
    self.assertEqual(browser_select_event(kind,event,False),['commit-widget-selection','close-select'])
   for event in (0xe002,0xfa17):
    self.assertEqual(browser_select_event(kind,event,False),['close-select'])
    self.assertEqual(browser_select_event(kind,event,True,changed=True),['commit-widget-selection','close-select'])
  self.assertEqual(browser_select_event('xhtml',0xfa1b,False,changed=True),['commit-widget-selection','dispatch-change-event-7-if-callback','close-select'])
 def test_select_stale_context_and_option_flags(self):
  for context,layout in ((False,True),(True,False)):
   self.assertEqual(browser_select_event('xhtml',0xfa1b,True,context_valid=context,layout_valid=layout),['close-select'])
  a=browser_xhtml_selection([1,0,0],[0,1,1],False,0)
  self.assertEqual((a['selected'],a['selected_index'],a['changed']),((False,True,True),2,True))
  self.assertEqual(browser_xhtml_selection([1,0],[1,0],True,7)['selected_index'],7)
  self.assertFalse(browser_xhtml_selection([1,0],[1,0],True,7)['changed'])
 def test_sms_success_waits_for_storage(self):
  a=sms_success_plan(need_save=True,saved_storage=0,next_segment=True)
  self.assertEqual(a,['backsend-status=3','send-status=1','wait-write-confirm'])
  a=sms_success_plan(need_save=True,saved_storage=2,next_segment=True,sequence=3)
  self.assertEqual(a[-1],'send-segment=4');self.assertIn('backsend-status=5',a)
  self.assertNotIn('wait-write-confirm',sms_success_plan(need_save=False,saved_storage=0))
 def test_sms_recipient_continuation(self):
  args=dict(need_save=False,saved_storage=0,next_destination=True)
  self.assertEqual(sms_success_plan(**args,send_result=0)[-1],'send-next-destination')
  self.assertEqual(sms_success_plan(**args,send_result=4)[-1],'finish(success=1,finished=1)')
  self.assertNotIn('send-next-destination',sms_success_plan(**args,move_ok=False))
 def test_sms_write_success_continues_and_clears_wait(self):
  a=sms_write_confirm_plan(1,next_segment=True,sequence=255)
  self.assertEqual(a,['finished=1','save-storage=0','send-segment=0','send-status=0'])
  for status in (3,4):
   a=sms_write_confirm_plan(status,next_segment=True,next_destination=True)
   self.assertNotIn('send-segment=2',a);self.assertEqual(a[-2: ],['send-next-destination','send-status=0'])
  self.assertEqual(sms_write_confirm_plan(99),['send-status=0'])
 def test_sms_failed_long_message_saves_remainder(self):
  for result,tail in ((0,[]),(8,['show-no-space-with-continuation']),(1,['show-error','close-send-window(result=0)'])):
   a=sms_write_confirm_plan(5,next_segment=True,save_result=result)
   self.assertEqual(a,['global-operation=12','current-operation=12','save-segment=2']+tail+['send-status=0'])
  self.assertEqual(sms_write_confirm_plan(1,context_valid=False),['ignore-missing-context'])
 def test_music_modes_separate_manual_and_eof(self):
  self.assertEqual([[music_play_mode(r,s) for s in (0,1)] for r in (0,1,2)],[[4,1],[3,2],[3,0]])
  self.assertEqual([[music_play_mode(r,s,natural_end=True) for s in (0,1)] for r in (0,1,2)],[[4,1],[3,2],[0,0]])
 def test_music_index_boundaries_and_failure_guard(self):
  self.assertEqual(music_index_step(0,3,-1),2);self.assertEqual(music_index_step(2,3,1),0)
  self.assertIsNone(music_index_step(0,0,1));self.assertEqual(music_index_step(0,1,1),0)
  for m in (2,3):
   self.assertFalse(music_end_guard(m,2,3));self.assertTrue(music_end_guard(m,3,3))
  self.assertTrue(music_end_guard(0,1,3));self.assertFalse(music_end_guard(4,9,3))
  self.assertFalse(music_end_guard(0,1,0))
 def test_music_end_callback_guards_and_queue_effects(self):
  self.assertEqual(music_end_plan(0,3),[])
  self.assertNotIn('advance-queue(direction=0)',music_end_plan(2,3,review_flag=True))
  a=music_end_plan(2,0);self.assertIn('0x885ff8d8(0)',a);self.assertNotIn('advance-queue(direction=0)',a)
  a=music_end_plan(2,0,failures=1);self.assertIn('0x885fff3a()',a);self.assertIn('0x885fffb2()',a)
  a=music_end_plan(2,3,advance_ok=False);self.assertIn('0x885fff3a()',a);self.assertNotIn('0x885fffb2()',a)
  a=music_end_plan(3,4,advance_ok=True,callback_result=2)
  self.assertIn('advance-queue(direction=0)',a);self.assertNotIn('0x885ff8d8(0)',a);self.assertNotIn('0x88416a26(0)',a)
 def test_continuation_native_anchors(self):
  b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
  self.assertEqual(b[0x6000dc:0x6000e1],bytes.fromhex('03c0c0c0c0'))
  self.assertEqual(b[0x3c3a62:0x3c3a64],bytes.fromhex('2061')) # write wait status
  self.assertEqual(b[0x3c447a:0x3c447c],bytes.fromhex('2561')) # clear wait status
  self.assertEqual(b[0x60148c:].split(b'\0',1)[0],b'GetNoSupportNumber num=%d')
 def test_independent_branch_anchors(self):
  data=json.loads((O/'event-routes.json').read_text());a={x['name']:x for x in data['applications']}
  for name,ev,target in [('browser-page','0xfa17','0x88b62486'),('browser-page','0xfa01','0x88b62364'),('music-player','0xfc03','0x88acf788'),('music-player','0xfd03','0x88acf840'),('sms-options','0xf021','0x8839bfec')]:
   row=next(x for x in a[name]['nondefault_routes'] if x['event']==ev);self.assertEqual(row['boundary_pc'],target)
 def test_menu_bytes_and_native_action_anchors(self):
  b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes();data=json.loads((O/'native-menus.json').read_text())
  for m in data['menus']:
   at=int(m['offset'],16);self.assertEqual(hashlib.sha256(b[at:at+len(m['rows'])*8]).hexdigest(),m['sha256'])
  switch=data['action_switches'][0]['rows'];targets={x['action']:x['target'] for x in switch}
  self.assertEqual(targets[16],'0x88b6375c');self.assertEqual(targets[18],'0x88b637c8')
  self.assertEqual([x['action'] for x in data['menus'][0]['rows']],[5,28,29,10,7,23,3,13])
  sms=next(t for t in data['action_switches'] if t['name']=='sms-body-actions');self.assertEqual(len(sms['rows']),137)
  self.assertEqual(sms['rows'][32]['target'],'0x8839c632');self.assertEqual(sms['rows'][36]['target'],'0x8839c702')
 def test_native_error_and_repeat_resources(self):
  rows=json.loads((O/'sms-operation-status.json').read_text())['rows'];self.assertEqual(rows[0]['text']['0'],'Sending…');self.assertEqual(rows[4]['text']['0'],'Phone number is empty');self.assertEqual(rows[7]['text']['0'],'No space')
  rows=json.loads((O/'music-repeat-settings.json').read_text())['rows'];self.assertEqual([x['text']['0'] for x in rows],['Off','Repeat all','Repeat one'])
 def test_unknown_carry_stops_instead_of_inventing_flag(self):
  ins={0:(2,'movs','r0, #1',''),2:(2,'bhs','0x8',''),4:(2,'bl','0x100','')}
  stop,reason,_=run(ins,b'',0,'r1',0);self.assertEqual(stop,2);self.assertIn('unknown operand',reason)
 def test_movs_preserves_carry(self):
  ins={0:(2,'cmp','r1, #1',''),2:(2,'movs','r0, #0',''),4:(2,'bhs','0x8',''),6:(2,'bl','0x100',''),8:(2,'bl','0x200','')}
  self.assertEqual(run(ins,b'',0,'r1',2)[0],8)

def main():
 suite=unittest.defaultTestLoader.loadTestsFromModule(__import__(__name__));stream=io.StringIO();result=unittest.TextTestRunner(stream=stream,verbosity=2).run(suite)
 report={'status':'passed' if result.wasSuccessful() else 'failed','tests':result.testsRun,'output':stream.getvalue(),'scope':'Synthetic replica contract tests plus native byte/branch anchors. No handset, modem, browser rendering, network, persistence adapter or audio playback acceptance. Policy choices are not newly proven firmware behavior.'}
 (O/'contract-validation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(stream.getvalue());return 0 if result.wasSuccessful() else 1
if __name__=='__main__':raise SystemExit(main())
