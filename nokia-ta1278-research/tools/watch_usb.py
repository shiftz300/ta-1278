#!/usr/bin/env python3
"""Passive macOS registry polling; never opens USB devices or volumes."""
import argparse
import datetime
import json
from pathlib import Path
import re
import subprocess
import time

out = Path(__file__).resolve().parents[1] / 'analysis' / 'environment'
out.mkdir(parents=True, exist_ok=True)
zone = datetime.timezone(datetime.timedelta(hours=8))
stamp = lambda: datetime.datetime.now(zone).isoformat(timespec='milliseconds')
dest = out / ('usb-watch-' + datetime.datetime.now(zone).strftime('%Y%m%dT%H%M%S%z') + '.jsonl')
allow = re.compile(r'\+-o|"(?:idVendor|idProduct|bDevice\w*|bInterface\w*|bcdUSB|bcdDevice|USB Product Name|USB Vendor Name|IOClass|IOProviderClass|CFBundleIdentifier\w*|Device Speed|bConfiguration\w*|bNumEndpoints|bNumConfigurations)"')
deny = re.compile(r'serial|imei|uuid|unique', re.I)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--seconds', type=int, default=180, choices=range(1, 601), metavar='1..600')
args = parser.parse_args()
previous = None
deadline = time.monotonic() + args.seconds
print('Passive USB watch started:', dest, flush=True)
with dest.open('x') as f:
    f.write(json.dumps({'event': 'watch_start', 'time_Asia_Shanghai': stamp(), 'duration_seconds': args.seconds}) + '\n')
    f.flush()
    while time.monotonic() < deadline:
        p = subprocess.run(['ioreg', '-p', 'IOUSB', '-l', '-w', '0'], capture_output=True, text=True, timeout=5)
        lines = [s for s in p.stdout.splitlines() if allow.search(s) and not deny.search(s)]
        # Retain counts change independently of connection state.
        lines = [re.sub(r'retain \d+', 'retain [omitted]', s) for s in lines]
        current = '\n'.join(lines)
        if current != previous:
            event = {'time_Asia_Shanghai': stamp(), 'exit': p.returncode, 'usb_filtered': current, 'stderr': p.stderr[:500]}
            f.write(json.dumps(event, ensure_ascii=False) + '\n'); f.flush()
            print(json.dumps(event, ensure_ascii=False), flush=True)
            previous = current
        time.sleep(0.25)
    f.write(json.dumps({'event': 'watch_end', 'time_Asia_Shanghai': stamp()}) + '\n')
print('Passive USB watch finished.', flush=True)
