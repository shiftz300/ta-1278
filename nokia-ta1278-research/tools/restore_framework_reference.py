#!/usr/bin/env python3
"""Restore only hash-identified public reference texts from the local archive.
No private partitions, network, firmware execution or overwrite of different files.
"""
from pathlib import Path
import hashlib,json,os,shutil
R=Path(__file__).resolve().parents[1]
repo=R.parent
archive=Path(os.environ.get('TA1278_SOURCE',str(repo.parent/'ta-1278-source')))
ref=Path('tools/upstream-reference/mmi-framework')
manifest=json.loads((R/ref/'provenance.json').read_text())
for item in manifest['files']:
 dst=R/ref/item['file']
 src=archive/'workspace'/R.name/ref/item['file']
 chosen=dst if dst.exists() else src
 digest=hashlib.sha256(chosen.read_bytes()).hexdigest()
 if digest!=item['local_sha256']:raise ValueError('Reference mismatch: '+item['file'])
 if not dst.exists():dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
print('Verified/restored %d public reference texts; no private data restored.'%len(manifest['files']))
