#!/usr/bin/env python3
"""Fixed-scope RAM loader + two 32 KiB BootROM reads. No flash commands."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--fresh', action='store_true', help='Wait for existing BootROM USB device to disappear before opening a new connection')
args = parser.parse_args()

root = Path(__file__).resolve().parents[1]
stamp = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).strftime('%Y%m%dT%H%M%S%z')
loader = root / 'tools/loaders/t117_fdl1.bin'
expected = '062d5e1a298c0e378eeefcc67dddba6fd4d6be10a683511c849ec59c17fb7414'
if hashlib.sha256(loader.read_bytes()).hexdigest() != expected:
    raise SystemExit('Loader digest mismatch')
work = root / 'analysis/work' / ('bootrom-' + stamp)
work.mkdir(parents=True, exist_ok=False)
first, second = work / 'read1.partial', work / 'read2.partial'
argv = [str(root / 'tools/build/spd_dump'), '--wait', '180', '--verbose', '1',
        'fdl', str(loader), '0x6200',
        'read_mem', '0xffff0000', '0x8000', str(first),
        'read_mem', '0xffff0000', '0x8000', str(second)]
record = {'time_Asia_Shanghai': stamp, 'argv': argv, 'loader_sha256': expected,
          'scope': 'BootROM 0xffff0000 + 0x8000, twice; custom loader in RAM; no NAND access'}
print('WORK', work, flush=True)
if args.fresh:
    print('Waiting for existing 1782:4d00 to disconnect; unplug for 2 seconds, then reconnect with boot key held.', flush=True)
    deadline = time.monotonic() + 180
    while True:
        scan = subprocess.run(['ioreg', '-p', 'IOUSB', '-l', '-w', '0'], capture_output=True, text=True, timeout=5)
        if scan.returncode:
            raise SystemExit('Registry query failed; no device commands sent')
        if '"idVendor" = 6018' not in scan.stdout or '"idProduct" = 19712' not in scan.stdout:
            break
        if time.monotonic() >= deadline:
            raise SystemExit('No disconnect observed; no device commands sent')
        time.sleep(0.1)
with (work / 'acquisition.log').open('x') as log:
    with subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True) as p:
        for line in p.stdout:
            log.write(line); log.flush()
            print(line, end='', flush=True)
        record['exit_code'] = p.wait()
checks = []
for path in [first, second]:
    if path.exists():
        data = path.read_bytes()
        checks.append({'file': path.name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
record['reads'] = checks
valid = (record['exit_code'] == 0 and len(checks) == 2 and
         all(x['bytes'] == 0x8000 for x in checks) and checks[0]['sha256'] == checks[1]['sha256'])
record['two_reads_match'] = valid
if valid:
    original = root / 'firmware/original' / ('bootrom-' + stamp + '.bin')
    original.parent.mkdir(parents=True, exist_ok=True)
    with original.open('xb') as f:
        f.write(first.read_bytes())
    original.chmod(0o444)
    record['original'] = str(original.relative_to(root))
    # Keep one work copy; verification hash for the second is retained above.
    first.rename(work / 'bootrom.bin')
    second.unlink()
(work / 'acquisition.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(record, ensure_ascii=False, indent=2))
raise SystemExit(0 if valid else 1)
