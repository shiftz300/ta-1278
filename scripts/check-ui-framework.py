#!/usr/bin/env python3
"""Verify native control descriptors and the scoped desktop UI refactor."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
EXPECTED = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
INDEX = ROOT / 'nokia-ta1278-research/analysis/ui/window-semantics.json'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path)
    parser.add_argument('--frames', type=Path)
    args = parser.parse_args()
    data = SOURCE.read_bytes()
    assert sha(data) == EXPECTED, 'Firmware hash mismatch'
    index = json.loads(INDEX.read_text())
    controls = Counter()
    selected = []
    for window in index['windows']:
        offset = int(window['offset'], 16)
        assert sha(data[offset:offset+window['size']]) == window['sha256'], window['offset']
        for attribute in window['attributes']:
            if attribute.get('framework_macro'):
                controls[attribute['framework_macro']] += 1
        if (window.get('name') or '').startswith(('SNAKEX_', 'MMISMS_CHATBOX_CHILD')):
            selected.append({key: window[key] for key in ['name','offset','size','sha256','attributes']})
    result = {
        'firmware_sha256': EXPECTED, 'index': str(INDEX.relative_to(ROOT)),
        'index_sha256': sha(INDEX.read_bytes()), 'verified_native_windows': len(index['windows']),
        'macro_counts': dict(sorted(controls.items())), 'selected_descriptors': selected,
        'runtime_controls': {
            'WIN_SOFTKEY': 'ui/softkeys.rs: typed left/center/right actions and common geometry',
            'CREATE_LISTBOX_CTRL': 'ui/list.rs: list rows, selection, radio and check markers',
            'CREATE_POPMENU_CTRL': 'ui/list.rs: same rows with popup geometry',
            'CREATE_EDIT_CTRL': 'ui/editor.rs and ui/text.rs: one edit renderer with metric profiles',
            'WIN_TITLE / MMK_CREATE_STATUSBAR': 'ui/chrome.rs: title and status control',
        },
        'limits': ['Macro names are correlated from a different public MMK reference version.',
                   'Native descriptor bytes are verified; this does not prove every attribute meaning.',
                   'The Rust controls are a local implementation; no reference C code is compiled.',
                   'Host geometry is fitted to reviewed handset captures; no complete MMK interpreter.',
                   'App models retain their own actions and native layout differences.'],
    }
    if args.frames or args.baseline:
        assert args.frames and args.baseline, 'Supply both --frames and --baseline'
        from PIL import Image, ImageChops
        frames = []
        exceptions = {'home.png': 'Live clock/date',
            'sms-incoming-options.png': 'Popup labels use the common 6px inset',
            'sms-read-options.png': 'Popup labels use the common 6px inset',
            'sms-thread-bubble-menu.png': 'Popup labels use the common 6px inset',
            'sms-thread-number-menu.png': 'Popup labels use the common 6px inset'}
        for current in sorted(args.frames.glob('*.png')):
            baseline = args.baseline / current.name
            assert baseline.exists(), current.name
            with Image.open(baseline) as a, Image.open(current) as b:
                assert a.size == b.size == (240,320)
                difference = ImageChops.difference(a.convert('RGB'),b.convert('RGB'))
                bbox = difference.getbbox()
                body_bbox = difference.crop((0,0,240,284)).getbbox()
            assert not body_bbox or current.name in exceptions, f'Unexpected body change: {current.name}'
            frames.append({'scene': current.name, 'before_sha256': sha(baseline.read_bytes()),
                'after_sha256': sha(current.read_bytes()), 'bbox': bbox, 'body_bbox': body_bbox,
                'exception': exceptions.get(current.name) if body_bbox else None})
        result['refactor_frames'] = frames
        result['summary'] = {'scenes':len(frames), 'identical':sum(not r['bbox'] for r in frames),
            'unchanged_body':sum(not r['body_bbox'] for r in frames)}
    target = ROOT / 'docs/evidence/shared-ui-framework.json'
    target.write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n')
    print(f'Verified {len(index["windows"])} native window slices; wrote shared UI evidence.')

if __name__ == '__main__':
    main()
