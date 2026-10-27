#!/usr/bin/env python3
"""Remove fallback glyphs already covered by earlier fonts; preserve the CJK/symbol cmap union; Emoji fallback is excluded.
Developer-only fontTools transform. Generated fonts are checked in for Rust-only builds.
"""
from pathlib import Path
import hashlib
import json
from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]


def main():
    covered = set()
    for name in ('font-0-latn.otf', 'font-1-hani.otf', 'font-2-hani.otf'):
        with TTFont(ROOT/'assets/fonts/device'/name) as font:
            covered.update(font.getBestCmap())
    out = ROOT/'assets/fonts/subsets'
    out.mkdir(exist_ok=True)
    rows = []
    for source, name in [('NotoSansCJKsc-Regular.otf', 'FallbackCJK.otf'),
                         ('NotoSansSymbols2-Regular.ttf', 'FallbackSymbols.ttf')]:
        path = ROOT/'assets/fonts'/source
        font = TTFont(path, recalcTimestamp=False)
        original = set(font.getBestCmap())
        keep = original - covered
        options = subset.Options()
        options.layout_features = []  # fontdue does not perform OpenType shaping.
        options.recalc_timestamp = False
        worker = subset.Subsetter(options=options)
        worker.populate(unicodes=keep)
        worker.subset(font)
        # Distinguish the modified fonts from their upstream names (OFL).
        family = 'TA1278 ' + Path(name).stem
        font['name'].names = [r for r in font['name'].names if r.nameID not in (1, 2, 3, 4, 6, 16, 17)]
        for platform, encoding, language in [(3, 1, 0x409), (1, 0, 0)]:
            for ident, text in [(1, family), (2, 'Regular'), (3, family + ' subset 1'),
                                (4, family + ' Regular'), (6, family.replace(' ', '') + '-Regular')]:
                font['name'].setName(text, ident, platform, encoding, language)
        if 'CFF ' in font:
            cff = font['CFF '].cff
            cff.fontNames = [family.replace(' ', '') + '-Regular']
            cff.topDictIndex[0].FamilyName = family
            cff.topDictIndex[0].FullName = family + ' Regular'
        target = out/name
        font.save(target)
        font.close()
        with TTFont(target) as restored:
            actual = set(restored.getBestCmap())
        assert keep <= actual <= original, source  # Some alias mappings are retained.
        assert covered | actual == covered | original, source
        rows.append({'source': str(path.relative_to(ROOT)), 'file': str(target.relative_to(ROOT)),
                     'source_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                     'sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
                     'source_bytes': path.stat().st_size, 'bytes': target.stat().st_size,
                     'source_codepoints': len(original), 'retained_codepoints': len(actual)})
        covered.update(original)
    report = {'method': 'Subset supplemental fonts in renderer fallback order; preserve Unicode cmap union. Remove unused layout shaping features. Rename derived font families; preserve copyright/license records. Original source fonts remain unchanged. Emoji fallback is omitted; unsupported characters render as a square.',
              'coverage_codepoints': len(covered), 'fonts': rows}
    (out/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
