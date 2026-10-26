#!/usr/bin/env python3
"""Rebuild app evidence in an isolated Git-visible export with only public system originals.
Creates a temporary copy; no network, handset I/O or firmware execution.
"""
from pathlib import Path
import tempfile,subprocess,shutil,json,hashlib,argparse
project=Path(__file__).resolve().parents[1];repo=project.parent
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--reference-root',type=Path,help='Archived mmi-framework directory; each file is hash-checked')
parser.add_argument('--report',type=Path,default=project/'analysis/ui/apps/reproduction-validation.json')
args=parser.parse_args()
ref=Path('tools/upstream-reference/mmi-framework');manifest=json.loads((project/ref/'provenance.json').read_text())
reference_root=args.reference_root or project/ref
for item in manifest['files']:
 assert hashlib.sha256((reference_root/item['file']).read_bytes()).hexdigest()==item['local_sha256'],item['file']
root=Path(tempfile.mkdtemp(prefix='ta1278-app-repro-',dir='/private/tmp'))
files=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],cwd=repo).decode().split('\0')
for name in set(filter(None,files)):
 src=repo/name
 if not src.is_file():continue
 dst=root/name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst)
r=root/'nokia-ta1278-research'
for item in manifest['files']:
 src=reference_root/item['file'];assert hashlib.sha256(src.read_bytes()).hexdigest()==item['local_sha256'];shutil.copyfile(src,r/ref/item['file'])
# Remove copied application outputs; regenerate them in an isolated public export.
shutil.rmtree(r/'analysis/ui/apps')
records=[]
for name in ['prepare_offline_workspace.py','analyze_ui_code.py','rebuild_app_analysis.py']:
 p=subprocess.run(['python3',str(r/'tools'/name)],cwd=r,capture_output=True,text=True)
 records.append({'script':name,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
 if p.returncode:raise RuntimeError(p.stderr+p.stdout)
compared=[]
for p in sorted((r/'analysis/ui/apps').iterdir()):
 if p.name in ('rebuild-log.json','contract-validation.json'):continue
 orig=project/'analysis/ui/apps'/p.name
 assert p.read_bytes()==orig.read_bytes(),p.name
 compared.append({'file':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
report={'status':'passed','method':'Isolated export of Git-visible files, original system manifest preparation, synthetic ELF regenerated, application outputs removed and rebuilt. Public reference texts copied only after hash verification (count in reference_files); no fresh network fetch claimed. No private NAND/NV/work copies imported.','reference_files':len(manifest['files']),'native_outputs_identical':compared,'commands':records,'contract_tests':json.loads((r/'analysis/ui/apps/contract-validation.json').read_text())['tests']}
args.report.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'status':'passed','identical_generated_outputs':len(compared),'contract_tests':report['contract_tests'],'export':str(root)}))
