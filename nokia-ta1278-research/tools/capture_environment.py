#!/usr/bin/env python3
"""Read host metadata/USB registry only. Never opens phone devices or volumes."""
import datetime
import glob
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'analysis' / 'environment'
OUT.mkdir(parents=True, exist_ok=True)
STAMP = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).strftime('%Y%m%dT%H%M%S%z')
SENSITIVE = re.compile(r'serial|imei|uuid|unique|iSerialNumber', re.I)
USB_PROP = re.compile(r'\+-o|"(?:idVendor|idProduct|bDevice\w*|bInterface\w*|bcdUSB|bcdDevice|USB Product Name|USB Vendor Name|IOClass|IOProviderClass|CFBundleIdentifier\w*|Device Speed|bConfiguration\w*|bNumEndpoints|bNumConfigurations)"')

def run(argv, usb=False):
    p = subprocess.run(argv, capture_output=True, text=True, timeout=30)
    lines = [s for s in p.stdout.splitlines() if not SENSITIVE.search(s)]
    if usb:
        lines = [s for s in lines if USB_PROP.search(s)]
    return {'argv': argv, 'exit_code': p.returncode, 'stdout': '\n'.join(lines), 'stderr': p.stderr[:2000]}

commands = [
    run(['sw_vers']), run(['uname', '-m']),
    run(['sysctl', '-n', 'machdep.cpu.brand_string']), run(['sysctl', '-n', 'hw.memsize']),
    run(['df', '-h', str(ROOT)]),
    run(['ioreg', '-p', 'IOUSB', '-l', '-w', '0'], True),
    run(['ioreg', '-r', '-c', 'IOUSBHostDevice', '-l', '-w', '0'], True),
    run(['ioreg', '-r', '-c', 'IOUSBHostInterface', '-l', '-w', '0'], True),
    run(['system_profiler', 'SPUSBHostDataType', '-detailLevel', 'mini']),
]
names = 'python3 file strings xxd hexdump shasum 7z 7zz binwalk ghidra ghidraRun analyzeHeadless radare2 rizin llvm-objdump objdump unzip tar curl git brew clang make pkg-config adb fastboot'.split()
data = {'timestamp_Asia_Shanghai': STAMP, 'privacy': 'Serial/IMEI/UUID lines omitted before persistence; no device contents accessed.',
        'commands': commands, 'tools_on_PATH': {n: shutil.which(n) for n in names},
        'serial_nodes': sorted(glob.glob('/dev/cu.*') + glob.glob('/dev/tty.*')),
        'application_candidates': sorted(glob.glob('/Applications/*Ghidra*') + glob.glob('/Applications/*ghidra*')),
        'libusb_local_candidates': sorted(glob.glob('/opt/homebrew/opt/libusb/lib/libusb*.dylib'))}
dest = OUT / (STAMP + '.json')
dest.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
print(dest)
print(json.dumps(data, ensure_ascii=False, indent=2))
