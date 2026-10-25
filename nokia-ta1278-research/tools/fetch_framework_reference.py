#!/usr/bin/env python3
"""Explicitly fetch pinned reference text; never execute it or install software."""
from pathlib import Path
import hashlib,json,urllib.request
S=Path(__file__).resolve().parent/'upstream-reference/mmi-framework'
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 info=json.loads((S/'provenance.json').read_text())
 for row in info['files']:
  dst=S/row['file'];expected=row['local_sha256']
  if dst.exists():
   if sha(dst.read_bytes())!=expected:raise ValueError('Existing reference differs: '+row['file'])
   print('verified',row['file']);continue
  url=row['url'].replace('https://github.com/','https://raw.githubusercontent.com/').replace('/blob/','/')
  req=urllib.request.Request(url,headers={'User-Agent':'ta1278-research-reference-fetch'})
  with urllib.request.urlopen(req,timeout=60) as res:raw=res.read(2*1024*1024+1)
  if len(raw)>2*1024*1024:raise ValueError('Unexpected reference size')
  candidates=[raw]
  for enc in ('utf-8','gb18030'):
   try:s=raw.decode(enc).replace('\r\n','\n')
   except UnicodeDecodeError:continue
   candidates.extend(x.encode('utf-8') for x in (s,s+'\n',s.rstrip('\n')+'\n'))
  data=next((b for b in candidates if sha(b)==expected),None)
  if data is None:raise ValueError('Pinned reference hash mismatch: '+row['file'])
  dst.write_bytes(data);print('downloaded and verified',row['file'])
if __name__=='__main__':main()
