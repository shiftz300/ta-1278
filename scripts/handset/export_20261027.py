#!/usr/bin/env python3
"""Curate non-personal UI evidence from existing sessions; never contacts USB."""
import argparse
import hashlib
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser()
parser.add_argument('--sessions', type=Path, default=Path('/private/tmp'))
args = parser.parse_args()
rows = [
    (6, 'files-root', 'files-root'),
    (8, 'files-folder-options', 'folder-options'),
    (12, 'folder-move-picker', 'delete-selection'),
    (17, 'move-folder-selection', 'move-destination'),
    (21, 'new-folder-editor', 'folder-name'),
    (30, 'other-root', 'other-root'),
    (31, 'timer-form', 'timer-empty'),
    (34, 'timer-abc', 'timer-abc'),
    (34, 'timer-name-a', 'timer-named'),
    (35, 'timer-running', 'timer-started'),
    (36, 'timer-active-page', 'timer-menu'),
    (38, 'timer-stopped-before', 'timer-expired'),
    (39, 'timer-alert-stop', 'timer-stopped'),
    (41, 'timer-after-stop', 'timer-reopened'),
]
out = ROOT / 'docs/handset-samples/20261027'
out.mkdir(parents=True, exist_ok=True)
frames = []
for session, source, name in rows:
    folder = args.sessions / f'ta1278-pages-{session:03d}'
    events = [json.loads(line) for line in (folder / 'events.jsonl').read_text().splitlines()]
    assert any(e.get('event') == 'version_verified' and e['version'] == '17.00.17.02' for e in events)
    capture = next(e for e in events if e.get('event') == 'capture' and e['name'] == source)
    src = folder / f'{source}.png'
    assert hashlib.sha256(src.read_bytes()).hexdigest() == capture['png_sha256']
    shutil.copyfile(src, out / f'{name}.png')
    frame = {'name': name, 'source_session': folder.name, 'source_frame': source,
             'png': f'{name}.png', 'capture': capture}
    if name == 'delete-selection':
        frame['correction'] = 'The original action name was wrong: a missed navigation press opened Delete selection. Nothing was selected or deleted; it was cancelled.'
    if name == 'timer-expired':
        frame['correction'] = 'The timer expired before the planned stop key; screenshot precondition failed and that key was not sent. The next session stopped the alert.'
    frames.append(frame)
(out / 'index.json').write_text(json.dumps({'firmware': '17.00.17.02', 'date_local': '2026-10-27', 'frames': frames}, ensure_ascii=False, indent=2) + '\n')
print(f'Exported {len(frames)} existing UI captures and verified PNG hashes; no device commands.')
