#!/usr/bin/env python3
"""Read only the known wallpaper file from the archived FAT12 volume.

No mounting, device access or other user-file export. Outputs stay local.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def extract(source, output):
    data = source.read_bytes()
    if hashlib.sha256(data).hexdigest() != '163318c2a98d1feb1d40d6eae4dea7be11886602457968233869241ace51da9a':
        raise ValueError('Not the verified handset storage capture')
    u16 = lambda at: struct.unpack_from('<H', data, at)[0]
    bps, spc, reserved, fats = u16(11), data[13], u16(14), data[16]
    roots, sectors, fat_sectors = u16(17), u16(19), u16(22)
    if sectors == 0:
        sectors = struct.unpack_from('<I', data, 32)[0]
    if (data[510:512] != b'\x55\xaa' or bps not in (512, 1024, 2048, 4096)
            or not spc or spc & (spc - 1) or not reserved or not fats or not fat_sectors):
        raise ValueError('Unsupported FAT header')
    root_sector = reserved + fats * fat_sectors
    root_sectors = (roots * 32 + bps - 1) // bps
    first_data = root_sector + root_sectors
    clusters = (sectors - first_data) // spc
    if not 0 < clusters < 4085 or sectors * bps > len(data):
        raise ValueError('Expected a complete FAT12 volume')
    fat = data[reserved*bps:(reserved+fat_sectors)*bps]

    def chain(start):
        chunks, seen = [], set()
        while start < 0xff8:
            if start < 2 or start > clusters + 1 or start in seen:
                raise ValueError('Invalid or cyclic cluster chain')
            seen.add(start)
            offset = (first_data + (start-2)*spc)*bps
            chunks.append(data[offset:offset+spc*bps])
            at = start + start//2
            pair = int.from_bytes(fat[at:at+2], 'little')
            start = pair >> 4 if start & 1 else pair & 0xfff
        return b''.join(chunks)

    def find(directory, wanted):
        parts = {}
        for at in range(0, len(directory), 32):
            entry = directory[at:at+32]
            if len(entry) < 32 or entry[0] == 0:
                break
            if entry[0] == 0xe5:
                parts = {}
                continue
            if entry[11] == 15:
                if entry[0] & 0x40:
                    parts = {}
                parts[entry[0] & 31] = entry[1:11]+entry[14:26]+entry[28:32]
                continue
            short = entry[:8].decode('ascii', errors='replace').strip()
            ext = entry[8:11].decode('ascii', errors='replace').strip()
            name = short + ('.'+ext if ext else '')
            if parts:
                name = b''.join(parts[i] for i in sorted(parts)).decode('utf-16le').split('\0')[0].rstrip('\uffff')
            parts = {}
            if not entry[11] & 8 and name.casefold() == wanted.casefold():
                return (entry[11], int.from_bytes(entry[26:28], 'little'),
                        int.from_bytes(entry[28:32], 'little'))
        raise FileNotFoundError(wanted)

    output.mkdir(parents=True, exist_ok=False)
    report = {'source_sha256': hashlib.sha256(data).hexdigest(), 'files': []}
    for path in ('wallpaper.dat',):
        directory = data[root_sector*bps:root_sector*bps+roots*32]
        try:
            components = path.split('/')
            for index, component in enumerate(components):
                attr, cluster, size = find(directory, component)
                if index < len(components)-1:
                    if not attr & 16:
                        raise ValueError('Expected a directory')
                    directory = chain(cluster)
                else:
                    if attr & 16 or not 0 < size <= 4*1024*1024:
                        raise ValueError('Invalid wallpaper file size/type')
                    content = chain(cluster)
                    if len(content) < size:
                        raise ValueError('Truncated wallpaper chain')
                    content = content[:size]
                    target = output / path.replace('/', '-')
                    target.write_bytes(content)
                    target.chmod(0o600)
                    report['files'].append({'path': path, 'bytes': size,
                                            'sha256': hashlib.sha256(content).hexdigest()})
                    if hashlib.sha256(content).hexdigest() != 'c308c0d219dd48e488ca96143053a8a02f3f7c41038df0f913ebf4e5dd13994e':
                        raise ValueError('Wallpaper differs from the verified cache')
                    # The matching capture establishes width/height and LE565.
                    # The 11-byte trailer is not image data and is not exported.
                    from PIL import Image
                    if len(content) != 232 * 316 * 2 + 11:
                        raise ValueError('Unexpected cache dimensions')
                    rgb = bytearray()
                    for (value,) in struct.iter_unpack('<H', content[:-11]):
                        r, g, b = value >> 11, (value >> 5) & 63, value & 31
                        rgb.extend(((r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2)))
                    Image.frombytes('RGB', (232, 316), bytes(rgb)).save(output/'wallpaper.png')
        except FileNotFoundError:
            report['files'].append({'path': path, 'missing': True})
    (output/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    extract(args.source, args.output)
