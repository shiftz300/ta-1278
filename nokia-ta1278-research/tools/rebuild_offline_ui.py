#!/usr/bin/env python3
"""Rebuild UI analysis from verified, already acquired system work copies.
No phone access, firmware execution, package installation, or network calls.
"""
from pathlib import Path
import subprocess,sys,json,datetime
R=Path(__file__).resolve().parents[1]
scripts=['extract_ui_resources.py','decode_abm.py','analyze_ui_code.py','decode_sfon.py','export_sfon_fonts.py','decode_extra_images.py','decode_sjpg.py','extract_ui_metadata.py','extract_data_resources.py','build_interaction_map.py','recover_framework_semantics.py','interaction_reference.py','build_ui_catalog.py','validate_ui_outputs.py']
records=[]
for name in scripts:
 p=subprocess.run([sys.executable,str(R/'tools'/name)],cwd=R,capture_output=True,text=True)
 records.append({'script':name,'returncode':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
 (R/'analysis/ui/rebuild-log.json').write_text(json.dumps({'at':datetime.datetime.now().astimezone().isoformat(),'steps':records},ensure_ascii=False,indent=2)+'\n')
 print(name,'OK' if p.returncode==0 else 'FAILED',flush=True)
 if p.returncode:print(p.stderr);sys.exit(p.returncode)
