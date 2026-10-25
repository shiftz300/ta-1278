#!/usr/bin/env python3
"""Compare reviewed handset PNGs with replica framebuffers; never read raw sessions."""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
PAIRS = [
    ('music-root-empty-selection', 'music-root'),
    ('browser-live-form', 'browser-native-form'),
    ('browser-links', 'browser-native-links'),
    ('browser-root-overlay', 'browser-root-popup'),
    ('browser-page-options', 'browser-page-popup'),
    ('browser-font-size-picker', 'browser-font-picker'),
    ('browser-clear-picker', 'browser-clear-picker'),
    ('browser-restore-focus', 'browser-settings-tail'),
    ('browser-url-editor', 'browser-url-input'),
    ('browser-exit-resampled', 'browser-exit'),
    ('sms-new', 'sms-empty-recipient'),
    ('sms-body-focus', 'sms-empty'),
    ('sms-recipient-focus', 'sms-forward'),
    ('snake-game', 'snake-native-matched'),
    ('snake-start', 'snake-menu'),
    ('snake-level-seven', 'snake-level'),
    ('snake-sound', 'snake-sound'),
    ('alarm-inline-form', 'alarm-form'),
    ('alarm-repeat-days', 'alarm-days'),
    ('alarm-monday-checked', 'alarm-days-selected'),
    ('alarm-snooze-picker', 'alarm-snooze-picker'),
    ('alarm-snooze-preview', 'alarm-snooze-preview'),
]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--replica', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    native = ROOT / 'docs/handset-samples/20261025'
    args.out.mkdir(parents=True, exist_ok=True)
    records = []
    for original, replica in PAIRS:
        sheet = Image.new('RGB', (496, 348), '#303030')
        draw = ImageDraw.Draw(sheet)
        hashes = []
        for x, label, path in [(0, 'Handset', native / (original + '.png')),
                                (256, 'Replica', args.replica / (replica + '.png'))]:
            with Image.open(path) as img:
                if img.size != (240, 320):
                    raise ValueError(f'Unexpected dimensions: {path.name}')
                sheet.paste(img.convert('RGB'), (x, 28))
            draw.text((x + 6, 7), label, fill='white')
            hashes.append(hashlib.sha256(path.read_bytes()).hexdigest())
        name = replica + '-comparison.png'
        sheet.save(args.out / name)
        records.append({'file': name, 'handset': original + '.png', 'replica': replica + '.png',
                        'handset_sha256': hashes[0], 'replica_sha256': hashes[1],
                        'sha256': hashlib.sha256((args.out / name).read_bytes()).hexdigest()})
    (args.out / 'index.json').write_text(json.dumps({
        'note': 'Original pixels. Different radio/status, input values and host fixture URLs are intentional. This is a visual review aid, not an equivalence score.',
        'pairs': records,
    }, indent=2) + '\n')
    print(f'Created {len(records)} labeled comparisons.')

if __name__ == '__main__':
    main()
