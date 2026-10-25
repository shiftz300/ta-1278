#!/usr/bin/env python3
"""Verify published system originals and prepare working copies; no USB/network."""
from pathlib import Path
import hashlib,json,shutil
R=Path(__file__).resolve().parents[1]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 manifest=json.loads((R/'firmware/public-system-manifest.json').read_text())
 for row in manifest['files']:
  p=R/row['file']
  if p.stat().st_size!=row['size'] or digest(p)!=row['sha256']:raise ValueError('Source mismatch: '+row['file'])
 work=R/'analysis/work/system-20261025T054301+0800';work.mkdir(parents=True,exist_ok=True)
 for row in manifest['files']:
  if row['component']=='bootrom':continue
  dst=work/(row['component']+'.bin')
  if dst.exists():
   if digest(dst)!=row['sha256']:raise ValueError('Refusing to overwrite different working file: '+str(dst))
  else:shutil.copyfile(R/row['file'],dst)
 print('Verified six originals; five system working copies ready. No device access.')
if __name__=='__main__':main()
