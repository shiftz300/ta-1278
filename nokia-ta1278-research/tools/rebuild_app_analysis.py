#!/usr/bin/env python3
"""Rebuild application contracts from existing verified system analysis artifacts."""
from pathlib import Path
import subprocess,sys,json
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps';O.mkdir(exist_ok=True)
scripts=['recover_framework_semantics.py','trace_app_code.py','recover_event_routes.py','extract_app_contracts.py','recover_app_algorithms.py','recover_interaction_core.py','test_app_contracts.py'];rows=[]
for name in scripts:
 p=subprocess.run([sys.executable,str(R/'tools'/name)],cwd=R,capture_output=True,text=True)
 rows.append({'script':name,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
 (O/'rebuild-log.json').write_text(json.dumps({'scope':'Offline static analysis and synthetic contract tests; no handset I/O or firmware execution','steps':rows},ensure_ascii=False,indent=2)+'\n')
 print(name,'OK' if p.returncode==0 else 'FAILED',flush=True)
 if p.returncode:print(p.stdout,p.stderr);raise SystemExit(p.returncode)
