#!/usr/bin/env python3
"""Compare existing CPU-rendered UI frames. No window, phone or audio access."""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw

parser = argparse.ArgumentParser()
parser.add_argument('baseline', type=Path)
parser.add_argument('current', type=Path)
parser.add_argument('--report', type=Path, required=True)
args = parser.parse_args()
files = sorted(p.name for p in args.baseline.glob('*.png'))
assert files, 'No baseline PNGs'
assert set(files) == {p.name for p in args.current.glob('*.png')}, 'Frame sets differ'
rows = []
for name in files:
    before, after = args.baseline / name, args.current / name
    a, b = Image.open(before).convert('RGB'), Image.open(after).convert('RGB')
    assert a.size == b.size == (240, 320), name
    difference = ImageChops.difference(a, b)
    bounds = difference.getbbox()
    mask = [0, 42, 240, 96] if name == 'configured-home.png' else None
    if mask:
        ImageDraw.Draw(difference).rectangle((mask[0], mask[1], mask[2]-1, mask[3]-1), fill=(0, 0, 0))
    rows.append({'frame': name, 'before_sha256': hashlib.sha256(before.read_bytes()).hexdigest(),
                 'after_sha256': hashlib.sha256(after.read_bytes()).hexdigest(),
                 'difference_bounds': bounds, 'live_clock_mask': mask,
                 'unchanged_outside_clock': difference.getbbox() is None})
report = {'baseline_kind': 'Current working tree before this framework refactor, including earlier uncommitted corrections.',
          'baseline': str(args.baseline), 'current': str(args.current), 'frames': rows,
          'exact_matches': sum(r['difference_bounds'] is None for r in rows),
          'passed': all(r['unchanged_outside_clock'] for r in rows),
          'boundary': 'CPU frames only. This checks regression against the prior implementation, not new native or GPU acceptance.'}
args.report.write_text(json.dumps(report, indent=2)+'\n')
print(f"{len(rows)} frames: {report['exact_matches']} exact; all outside the live-clock mask: {report['passed']}")
raise SystemExit(0 if report['passed'] else 1)
