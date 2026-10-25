#!/usr/bin/env python3
"""Verify original files against the prior manifest, then hash stable local outputs."""
from pathlib import Path
import hashlib,json
R=Path(__file__).resolve().parents[1]
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
def main():
 manifest=R/'SHA256SUMS';old={}
 for line in manifest.read_text().splitlines():
  value,name=line.split('  ',1)
  if name.startswith('firmware/original/'):old[name]=value
 assert old,'Refusing to replace a manifest without an original-file baseline'
 rows=[]
 for name,expected in sorted(old.items()):
  p=R/name;actual=digest(p)
  assert actual==expected,('Original changed',name)
  rows.append({'file':name,'size':p.stat().st_size,'sha256':actual})
 actual_originals={str(p.relative_to(R)) for p in (R/'firmware/original').rglob('*') if p.is_file() and not p.is_symlink()}
 assert actual_originals==set(old),'Original inventory changed; review instead of silently adopting it'
 report={'status':'passed','method':'All original files compared against previous SHA256SUMS before replacement','original_file_count':len(rows),'originals':rows}
 (R/'analysis/ui/original-integrity.json').write_text(json.dumps(report,indent=2)+'\n')
 records=[]
 for p in sorted(R.rglob('*')):
  rel=p.relative_to(R)
  if p.is_symlink() or not p.is_file() or p==manifest:continue
  if any(x in ('.git','__pycache__') for x in rel.parts) or p.name.endswith('.partial') or p.name=='.DS_Store':continue
  records.append(digest(p)+'  '+str(rel))
 manifest.write_text('\n'.join(records)+'\n')
 print(json.dumps({'originals_unchanged':len(rows),'stable_files_hashed':len(records)}))
if __name__=='__main__':main()
