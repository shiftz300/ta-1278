#!/usr/bin/env python3
"""Recover runtime diagnostic entry evidence from the original system image.

Offline only: no device access, writes or execution of firmware code.
"""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
EXPECTED = '8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
BASE = 0x88000000


def main():
    data = SOURCE.read_bytes()
    assert hashlib.sha256(data).hexdigest() == EXPECTED
    u32 = lambda off: struct.unpack_from('<I', data, off)[0]
    table = []
    for off in range(0x6f8d94, 0x6f8ed4, 16):
        ptr, option, control, handler = struct.unpack_from('<4I', data, off)
        start = ptr - BASE
        assert 0 <= start < len(data)
        code = data[start:data.index(b'\0', start)].decode('ascii')
        table.append({'offset': hex(off), 'code': code, 'option': option,
                      'control': control, 'handler': hex(handler)})
    entry = next(row for row in table if row['code'] == '#*8378#8#')
    assert entry['handler'] == '0x882ddc3d'
    assert u32(0x2de038) == BASE + 0x6f90a0
    assert u32(0x6f90a4) == BASE + 0x2df903
    assert u32(0x6d20c) - 0x2d == 0x130006
    assert '#*8378#1#' not in [row['code'] for row in table]
    # Thumb ADR uses Align(PC+4, 4); LLVM's printed ADR target is misleading here.
    fixture = {}
    for label, instruction, expected_offset, expected_text in [
        ('number', 0x5eaa46, 0x5ead98, '123%04d'),
        ('name', 0x5eaa80, 0x5eada0, 'a%04d'),
    ]:
        halfword = struct.unpack_from('<H', data, instruction)[0]
        assert halfword & 0xf800 == 0xa000
        offset = ((instruction+4) & ~3) + (halfword & 255)*4
        text = data[offset:data.index(b'\0', offset)].decode('ascii')
        assert offset == expected_offset and text == expected_text
        fixture[label] = {'adr_instruction': hex(instruction), 'offset': hex(offset),
                          'format': text}

    ranges = [
        ('runtime-entry-open', 0x2ddc3c, 0x2ddc4a),
        ('runtime-test-setting', 0x2df902, 0x2dfa78),
        ('runtime-usblog-nv', 0x6d152, 0x6d186),
        ('runtime-usblog-window', 0x6d31c, 0x6d39c),
        ('runtime-contact-fill', 0x5ea9c4, 0x5eaaca),
    ]
    # Minimal ELF wrapper, used only by llvm-objdump.
    names = b'\0.text\0.shstrtab\0'
    shoff = (256 + len(data) + len(names) + 3) & ~3
    header = struct.pack('<16sHHIIIIIHHHHHH',
                         b'\x7fELF\x01\x01\x01' + bytes(9), 2, 40, 1,
                         BASE, 0, shoff, 0x5000000, 52, 0, 0, 40, 3, 2)
    elf = (header.ljust(256, b'\0') + data + names).ljust(shoff, b'\0') + bytes(40)
    elf += struct.pack('<10I', 1, 1, 6, BASE, 256, len(data), 0, 0, 4, 0)
    elf += struct.pack('<10I', 7, 3, 0, 0, 256 + len(data), len(names), 0, 0, 1, 0)
    out = ROOT / 'docs/evidence'
    out.mkdir(exist_ok=True)
    records = []
    with tempfile.TemporaryDirectory(prefix='ta1278-entry-') as temporary:
        path = Path(temporary) / 'user.analysis-only.elf'
        path.write_bytes(elf)
        for name, start, end in ranges:
            command = ['xcrun', 'llvm-objdump', '-d', '--triple=thumbv7a-none-eabi',
                       '--mcpu=cortex-a5', f'--start-address={BASE+start}',
                       f'--stop-address={BASE+end}', str(path)]
            text = subprocess.check_output(command, text=True)
            (out / (name + '.txt')).write_text(text.replace(str(path), path.name))
            records.append({'name': name, 'start': hex(start), 'end_exclusive': hex(end),
                            'sha256': hashlib.sha256(data[start:end]).hexdigest()})
    result = {'source': str(SOURCE.relative_to(ROOT)), 'source_sha256': EXPECTED,
              'dial_table': table, 'usblog_nv_id': '0x130006', 'ranges': records,
              'generated_contact_formats': fixture,
              'status': 'Static entry and side effects recovered. This script does not test runtime USB; see docs/handset-observations.md for separate live results.',
              'limits': 'Opening the menu only constructs a list. Selecting Open also changes sound, call restrictions, lock/backlight and fills contacts. Close is not a complete rollback. No device command was sent by this script.'}
    (out / 'runtime-entry.json').write_text(json.dumps(result, indent=2) + '\n')
    print('Verified runtime entry table and bounded native paths; no device access.')


if __name__ == '__main__':
    main()
