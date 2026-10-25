#!/usr/bin/env python3
"""Synthetic behavioral regression tests + independent native branch anchors.
Passing means reference contract consistency, not physical handset acceptance.
"""
import json,unittest,io,hashlib,struct
from pathlib import Path
from app_interaction_reference import Editor,SmsApp,BrowserApp,MusicApp,Page,sms_counter,units
from recover_event_routes import run
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
 def test_counter_extension_does_not_split_escape_pair(self):
  x=sms_counter('A'*152+'€'+'A'*152)
  self.assertEqual((x['units'],x['segments'],x['remaining']),(306,3,152))
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
class NativeEvidenceTests(unittest.TestCase):
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
