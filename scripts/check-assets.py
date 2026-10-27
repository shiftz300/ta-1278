#!/usr/bin/env python3
"""Verify checked-in resources using only the Python standard library.
Use --fonts locally after changing fonts to regenerate the coverage report.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FONTS = [
    'assets/fonts/device/font-0-latn.otf',
    'assets/fonts/device/font-1-hani.otf',
    'assets/fonts/device/font-2-hani.otf',
    'assets/fonts/subsets/FallbackCJK.otf',
    'assets/fonts/subsets/FallbackSymbols.ttf',
    'assets/fonts/subsets/FallbackEmoji.ttf',
]


def read(path):
    return json.loads((ROOT / path).read_text(encoding='utf-8'))


def digest(path):
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def verify(path, expected, size=None):
    if size is not None and (ROOT / path).stat().st_size != size:
        raise ValueError(f'Resource size mismatch: {path}')
    if digest(path) != expected:
        raise ValueError(f'Resource SHA-256 mismatch: {path}')


def coverage():
    # Developer-only dependency. Normal checks must work without site-packages.
    from fontTools.ttLib import TTFont

    chars, rows, before = set(), [], 0
    for i, path in enumerate(FONTS):
        with TTFont(ROOT / path) as font:
            current = set(font.getBestCmap())
        rows.append({'file': path, 'sha256': digest(path),
                     'characters': len(current), 'added': len(current - chars)})
        chars.update(current)
        if i == 1:
            before = len(chars)
    sample = '你好繁體漢字髙龘喆𠮷こんにちはカタカナ안녕하세요→←✓★♫😀🙂🤔👍✨'
    if not all(ord(c) in chars for c in sample):
        raise ValueError('Required sample characters are missing')
    report = {'method': 'Unicode cmap union in renderer fallback order; not full shaping or emoji sequence support',
              'before': before, 'after': len(chars), 'added': len(chars) - before, 'fonts': rows}
    (ROOT / 'assets/font-coverage.json').write_bytes(
        (json.dumps(report, ensure_ascii=False, indent=2) + '\n').encode('utf-8'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fonts', action='store_true', help='Regenerate font coverage locally (requires fontTools)')
    args = parser.parse_args()
    for row in read('assets/runtime.json')['files']:
        verify(row['file'], row['sha256'], row['bytes'])
    wallpaper = read('assets/ui/owner-wallpaper.json')
    verify(wallpaper['file'], wallpaper['sha256'], wallpaper['bytes'])
    for row in read('assets/sources.json'):
        verify('assets/' + row['file'], row['sha256'], row['size'])
    for row in read('assets/fonts/subsets/manifest.json')['fonts']:
        verify(row['file'], row['sha256'], row['bytes'])
        verify(row['source'], row['source_sha256'], row['source_bytes'])
    if args.fonts:
        coverage()
    report = read('assets/font-coverage.json')
    if [row['file'] for row in report['fonts']] != FONTS:
        raise ValueError('Font coverage order is stale; run --fonts locally')
    for row in report['fonts']:
        verify(row['file'], row['sha256'])
    print('Resource SHA-256 checks passed; recorded coverage %s -> %s codepoints.' %
          (report['before'], report['after']))


if __name__ == '__main__':
    main()
