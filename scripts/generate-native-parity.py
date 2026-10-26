#!/usr/bin/env python3
"""Generate test-only cases from the independently traced SMS reference rules."""
import json
from pathlib import Path
import sys
import zlib

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'nokia-ta1278-research/tools'))
from native_app_rules import sms_character_counts, sms_ui_count, sms_uses_ucs2


def result(text):
    counts = sms_character_counts(text)
    ui = sms_ui_count(**counts)
    return dict(remaining=ui['remaining'], segments=ui['segments'],
                ucs2=sms_uses_ucs2(counts['ucs2_count'], counts['language_counts']))


def main():
    out = ROOT / 'crates/phone-core/tests/fixtures'
    out.mkdir(parents=True, exist_ok=True)
    bmp = bytearray()
    for code in range(65536):
        if 0xd800 <= code <= 0xdfff:
            continue
        r = result(chr(code))
        assert r['segments'] == 1
        bmp.extend((r['remaining'], r['ucs2']))
    (out / 'sms-native-bmp.z').write_bytes(zlib.compress(bmp, 9))
    cases = []
    for pattern in ['A', '中', '€', '^', '\x1b', 'ª', 'Á', 'Ş', 'Şª', 'Áª', '😀', 'A€中']:
        for length in [0, 1, 69, 70, 71, 145, 146, 149, 153, 155, 160, 161, 306, 1024]:
            cases.append({'pattern': pattern, 'repeat': length, **result(pattern * length)})
    cases.append({'text': 'A' * 152 + '€' + 'A' * 152,
                  **result('A' * 152 + '€' + 'A' * 152)})
    (out / 'sms-native-boundaries.json').write_text(json.dumps(cases, ensure_ascii=False) + '\n')
    print(f'Generated {len(bmp)//2} BMP cases and {len(cases)} boundary/mixed-language cases. Test-only assets.')


if __name__ == '__main__':
    main()
