#!/usr/bin/env python3
"""Verify downloaded resources and record the actual font fallback coverage.
fontTools is only needed for this developer check, not for running the app.
"""
from pathlib import Path
import hashlib,json
from fontTools.ttLib import TTFont
root=Path(__file__).resolve().parents[1]
for row in json.loads((root/'assets/sources.json').read_text()):
 p=root/'assets'/row['file'];assert p.stat().st_size==row['size'];assert hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256'],row['file']
paths=['nokia-ta1278-research/analysis/ui/fonts/font-0-latn.otf','nokia-ta1278-research/analysis/ui/fonts/font-1-hani.otf','nokia-ta1278-research/analysis/ui/fonts/font-2-hani.otf','assets/fonts/NotoSansCJKsc-Regular.otf','assets/fonts/NotoSansSymbols2-Regular.ttf','assets/fonts/NotoEmoji.ttf']
all_chars=set();rows=[];before=0
for i,path in enumerate(paths):
 f=TTFont(root/path);chars=set(f.getBestCmap());rows.append({'file':path,'characters':len(chars),'added':len(chars-all_chars)});all_chars|=chars
 if i==1:before=len(all_chars)
sample='你好繁體漢字髙龘喆𠮷こんにちはカタカナ안녕하세요→←✓★♫😀🙂🤔👍✨'
assert all(ord(c) in all_chars for c in sample)
report={'method':'Unicode cmap union in renderer fallback order; not full shaping or emoji sequence support','before':before,'after':len(all_chars),'added':len(all_chars)-before,'fonts':rows}
(root/'assets/font-coverage.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print('Resource SHA-256 checks passed; coverage %s -> %s codepoints.'%(before,len(all_chars)))
