#!/usr/bin/env python3
"""Check reviewed browser anchors against published firmware; no device/network I/O."""
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[1]
RESEARCH = ROOT / 'nokia-ta1278-research'
APPS = RESEARCH / 'analysis/ui/apps'


def main():
    manifest = json.loads((RESEARCH / 'firmware/public-system-manifest.json').read_text())
    source = next(row for row in manifest['files'] if row['component'] == 'user')
    data = (RESEARCH / source['file']).read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    assert len(data) == source['size'] and digest == source['sha256']
    rules = json.loads((APPS / 'interaction-core-rules.json').read_text())
    assert digest == rules['source_sha256']
    regions = []
    for section in ('browser_fields', 'browser_select', 'browser_form_actions', 'browser_progress'):
        for region in rules[section]['regions']:
            start = int(region['start'], 16) - 0x88000000
            end = int(region['end_exclusive'], 16) - 0x88000000
            assert hashlib.sha256(data[start:end]).hexdigest() == region['sha256']
            regions.append({'section': section, **region})
    # Verify the actual reviewed instruction listings, not just prose or symbol names.
    listings = {}
    for name in ('browser-field-input-thumb.txt', 'browser-select-thumb.txt',
                 'browser-form-reset-thumb.txt', 'browser-input-click-thumb.txt',
                 'browser-page-thumb.txt', 'browser-key-timers-thumb.txt'):
        count = 0
        for line in (APPS / name).read_text().splitlines():
            match = re.match(r'^([0-9a-f]{8}): ((?:[0-9a-f]{4} ){1,2})\s', line)
            if not match:
                continue
            at = int(match[1], 16) - 0x88000000
            raw = b''.join(bytes.fromhex(word)[::-1] for word in match[2].split())
            assert data[at:at + len(raw)] == raw, (name, match[1])
            count += 1
        assert count > 0, name
        listings[name] = count
    report = {
        'source': source['file'], 'source_sha256': digest,
        'verified_regions': regions, 'listing_instructions_checked': listings,
        'scope': 'Byte validation of previously reviewed Dorado branches. No firmware execution, handset access, engine-selection proof or complete browser equivalence.',
        'correction': {
            'native': 'XHTML initial field copy at 0x88b48fa0..0x88b48fb2 uses min(value UTF16 length, field limit).',
            'replica': 'Limit initial editor text as well as later input; cancel preserves the original page value.',
            'host_difference': 'A split UTF16 surrogate pair is omitted rather than leaving an unpaired high surrogate.'
        }
    }
    target = ROOT / 'docs/evidence/browser-code-audit.json'
    target.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Verified {len(regions)} regions and {sum(listings.values())} listed instructions against user.bin.')


if __name__ == '__main__':
    main()
