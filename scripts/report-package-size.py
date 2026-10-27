#!/usr/bin/env python3
"""Measure the signed macOS bundle and locate embedded packed bytes in Mach-O."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
ASSETS = {
    'original_fonts': [('LATIN', 'assets/fonts/device/font-0-latn.otf'),
                       ('HANI1', 'assets/fonts/device/font-1-hani.otf'),
                       ('HANI2', 'assets/fonts/device/font-2-hani.otf')],
    'fallback_fonts': [('CJK', 'assets/fonts/subsets/FallbackCJK.otf'),
                       ('SYMBOLS', 'assets/fonts/subsets/FallbackSymbols.ttf'),
                       ('EMOJI', 'assets/fonts/subsets/FallbackEmoji.ttf')],
    't9': [('T9_CODE', 'assets/input/t9/engine.arm'),
           ('T9_TABLES', 'assets/input/t9/character-tables.bin'),
           ('T9_ENGLISH', 'assets/input/t9/english.ldb'),
           ('T9_CHINESE', 'assets/input/t9/chinese.ldb')],
    'native_soundfont': [('SOUNDFONT', 'assets/audio/gm1_A21.sf2')],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, default=ROOT/'target/packed-build-size.json')
    args = parser.parse_args()
    bundle = ROOT/'target/TA1278.app'
    binary = (bundle/'Contents/MacOS/phone-desktop').read_bytes()
    groups, resources = {}, []
    for group, entries in ASSETS.items():
        groups[group] = 0
        for name, path in entries:
            raw = (ROOT/path).read_bytes()
            matches = {}
            for candidate in (ROOT/'target/release/build').glob(f'phone-*/out/{name}.z'):
                data = candidate.read_bytes()
                if data in binary:
                    matches[data] = candidate
            assert len(matches) == 1, (name, 'Expected exactly one embedded packed representation')
            data = next(iter(matches))
            assert binary.count(data) == 1 and raw not in binary, name
            assert zlib.decompress(data) == raw, name
            resources.append({'source': path, 'raw_bytes': len(raw), 'packed_bytes': len(data),
                              'raw_sha256': hashlib.sha256(raw).hexdigest(),
                              'packed_sha256': hashlib.sha256(data).hexdigest(),
                              'executable_offset': binary.index(data)})
            groups[group] += len(data)
    # Mach-O LC_SEGMENT_64: count machine-code sections separately from constants.
    assert struct.unpack_from('<I', binary)[0] == 0xfeedfacf
    commands = struct.unpack_from('<I', binary, 16)[0]
    at, code = 32, 0
    for _ in range(commands):
        command, size = struct.unpack_from('<II', binary, at)
        if command == 0x19:
            count = struct.unpack_from('<I', binary, at + 64)[0]
            for i in range(count):
                section = at + 72 + i * 80
                name = binary[section:section+16].split(b'\0')[0]
                if name in (b'__text', b'__stubs'):
                    code += struct.unpack_from('<Q', binary, section + 40)[0]
        at += size
    groups['machine_code_including_static_dependencies'] = code
    groups['other_executable_data_and_metadata'] = len(binary) - sum(groups.values())
    helper = bundle/'Contents/MacOS/ta1278-media'
    groups['native_media_helper'] = helper.stat().st_size
    files = [p for p in bundle.rglob('*') if p.is_file()]
    total = sum(p.stat().st_size for p in files)
    groups['notices_and_bundle_metadata'] = total-len(binary)-helper.stat().st_size
    assert sum(groups.values()) == total and min(groups.values()) >= 0
    report = {'bundle_bytes': total, 'executable_bytes': len(binary),
              'executable_sha256': hashlib.sha256(binary).hexdigest(),
              'groups_bytes': groups, 'packed_resources': resources,
              'method': 'Logical file sizes, embedded byte search, independent Python zlib roundtrip, Mach-O __text/__stubs sizes. Other executable data includes small images/audio, strings, tables, alignment and signing metadata; it is not all code.',
              'profile': {'opt-level': 'z', 'lto': 'fat', 'codegen-units': 1, 'strip': 'symbols', 'panic': 'abort'}}
    args.out.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({'bundle_bytes':total, 'groups_bytes':groups}, indent=2))


if __name__ == '__main__':
    main()
