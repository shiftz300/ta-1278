#!/usr/bin/env python3
"""Compare equal-content public handset SMS samples with synthetic app frames."""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
PAIRS = [
    ('sms-delete-newline', 'sms-native-one', 'One Chinese character; Abc mode'),
    ('sms-newline', 'sms-native-newline', 'Chinese character and newline; Abc mode'),
    ('sms-unicode-70', 'sms-native-wrap', 'One Chinese character and 69 digits; numeric mode'),
    ('sms-numeric-161', 'sms-native-scrolled', '161 digits; last nine rows visible'),
]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--replica', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    records = []
    for native, replica, state in PAIRS:
        source = ROOT / 'docs/handset-samples/20261025' / (native + '.png')
        frame = args.replica / (replica + '.png')
        sheet = Image.new('RGB', (496, 348), '#303030')
        draw = ImageDraw.Draw(sheet)
        for x, label, path in [(0, 'Handset 17.00.17.02', source), (256, 'Replica', frame)]:
            with Image.open(path) as image:
                if image.size != (240, 320):
                    raise ValueError(f'Unexpected dimensions: {path}')
                sheet.paste(image.convert('RGB'), (x, 28))
            draw.text((x + 6, 7), label, fill='white')
        target = args.out / (replica + '-comparison.png')
        sheet.save(target)
        records.append({'state': state, 'handset': str(source.relative_to(ROOT)),
                        'handset_sha256': sha(source), 'replica': frame.name,
                        'replica_sha256': sha(frame), 'comparison': target.name,
                        'comparison_sha256': sha(target)})
    (args.out / 'sms-composer-index.json').write_text(json.dumps({
        'firmware': '17.00.17.02', 'pairs': records,
        'geometry': {'bubble_bottom': 254, 'single_row_top': 220, 'two_rows_top': 194,
                     'full_height_top': 24, 'last_editor_row_y': 224, 'caret_y': [225, 251],
                     'count_ink_y': [261, 274], 'softkey_text_y': 289},
        'limits': ['Whole-pixel advances, 20px Latin and 24px CJK are fitted from these samples.',
                   'Original SFON outlines are used through derived OTF/fonts; original hinting is not restored.',
                   'Native JPEG is lossy. Status icons differ; no all-pixel equivalence claimed.',
                   'Six-row bubble top and some glyph edges still differ by about 1-2 pixels.',
                   'Pencil and delete-key outlines remain local renderings, not recovered resources.'],
    }, indent=2) + '\n')
    print(f'Created {len(records)} equal-content comparisons.')

if __name__ == '__main__':
    main()
