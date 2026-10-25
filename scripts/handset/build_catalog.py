#!/usr/bin/env python3
"""Seed a coverage ledger from compiled registrations, never claim runtime visibility."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
source = ROOT / 'nokia-ta1278-research/analysis/ui/mainmenu-registry.json'
registry = json.loads(source.read_text())
facets = ['entry_and_exit', 'page_layout_and_resources', 'focus_and_scrolling',
          'softkeys_and_context_menus', 'dialogs_and_cancel', 'settings_and_persistence',
          'empty_and_error_states', 'background_and_resume']
apps = []
for row in registry['rows']:
    apps.append({'registry_slot': row['slot'], 'app_id': row['app_id'],
                 'label': row['labels'].get('1', row['labels'].get('0')),
                 'handler_thumb': row['handler_thumb'],
                 'runtime_visibility': 'unknown', 'scope': 'include_if_present',
                 'cases': {f: {'status': 'not_sampled', 'artifacts': [], 'note': ''} for f in facets}})
extra = {
    'home_and_lock': ['startup', 'status_bar', 'idle_softkeys', 'lock_unlock', 'shortcuts', 'interrupt_and_resume'],
    'input_and_sms': ['recipient', 'body', 'input_mode_menu', 'chinese_spelling', 'candidate_paging',
                      'english_prediction', 'multitap', 'symbols', 'cursor_and_selection', 'short_delete',
                      'long_delete', 'newlines', 'character_counts', 'scrolling', 'insert_menus',
                      'save_reopen', 'discard_cancel', 'drafts', 'outbox', 'send_confirmation_without_send'],
    'browser': ['entry_engine', 'home', 'address_edit', 'links', 'scroll', 'forms', 'history', 'bookmarks',
                'load_cancel', 'errors', 'options', 'exit_resume'],
    'music': ['empty_library', 'library', 'play_pause', 'track_change', 'seek', 'volume', 'repeat_shuffle',
              'options', 'background_resume', 'end_of_track'],
}
result = {
    'schema': 1,
    'scope': 'All handset functions except owner-excluded third-party services. Compiled entries are candidates, not a runtime menu.',
    'owner_exclusions': ['喜马拉雅', '支付宝'],
    'exclusion_rule': 'Other apps remain included until explicitly excluded or proven absent. No silent exclusions.',
    'source': str(source.relative_to(ROOT)), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
    'statuses': ['not_sampled', 'partially_sampled', 'captured', 'transition_verified', 'implemented', 'replica_compared', 'blocked', 'not_present', 'excluded_by_owner'],
    'completion_rule': 'Every runtime-visible page/action needs handset evidence, a transition result and replica comparison. Blocked and firmware-only cases do not count as sampled. No global percentage from registry counts.',
    'blocked_actions': ['real_call', 'real_message_send', 'payment', 'erase_or_factory_reset', 'delete_existing_user_data'],
    'applications': apps,
    'cross_cutting_cases': {group: {case: {'status':'not_sampled', 'artifacts':[], 'note':''} for case in cases}
                            for group,cases in extra.items()},
}
out = ROOT/'docs/handset-coverage.json'
if out.exists():
    raise SystemExit('Refusing to overwrite an existing coverage ledger')
out.write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
print(f'Created {len(apps)} candidate app rows and {sum(map(len,extra.values()))} cross-cutting cases; zero claimed handset samples.')
