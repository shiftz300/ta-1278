#!/usr/bin/env python3
"""Check the Rust Latin multi-tap table against the public firmware extraction."""
from pathlib import Path
import re,json
root=Path(__file__).resolve().parents[1]
source=(root/'crates/phone-core/src/editor.rs').read_text()
body=re.search(r'pub const TAP:.*?=\s*\[(.*?)\];',source,re.S).group(1)
actual=[json.loads(s) for s in re.findall(r'"(?:\\.|[^"\\])*"',body)]
data=json.loads((root/'nokia-ta1278-research/analysis/ui/multitap-tables.json').read_text())
rows={row['key']:row['text'] for row in data['tables'][0]['rows']}
assert len(actual)==10
for digit,text in enumerate(actual):assert text==rows[str(digit)],digit
print('10 Rust multi-tap rows match the extracted table at user+0x21d592.')
