#!/usr/bin/env python3
"""Verify bounded SMS error branches; extract tables without executing firmware."""
from pathlib import Path
import hashlib,json,struct,sys
from firmware_disassembly import disassemble_arm
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'nokia-ta1278-research/tools'))
from recover_event_routes import parse,run
src=ROOT/'nokia-ta1278-research/firmware/original/system-20261025T054301+0800/user.bin'
b=src.read_bytes(); sha=hashlib.sha256(b).hexdigest()
assert sha=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
prior=json.loads((ROOT/'nokia-ta1278-research/analysis/ui/apps/sms-retry-rules.json').read_text())
ranges=[]
for key in ['function','error_handler','error_handler_continuation']:
 r=prior[key];lo=int(r['start'],16);hi=int(r['end_exclusive'],16)
 assert hashlib.sha256(b[lo-0x88000000:hi-0x88000000]).hexdigest()==r['sha256']
 ranges.append((lo,hi))
listing=ROOT/'docs/evidence/sms-lifecycle-thumb.txt'
listing.write_text(disassemble_arm(b,0x88000000,ranges))
ins=parse(listing);accepted=[]
for cause in range(65536):
 pc,why,trace=run(ins,b,0x883c5e68,'r0',cause)
 assert (pc,why)==(0x883c5eca,'side-effect boundary: bx')
 if 0x883c5ec8 in trace:accepted.append(cause)
assert accepted==prior['accepted_rl_causes']
asset=ROOT/'assets/ui/sms-retry-causes.bin';asset.write_bytes(struct.pack('<'+'H'*len(accepted),*accepted))
status=json.loads((ROOT/'nokia-ta1278-research/analysis/ui/apps/sms-operation-status.json').read_text())
labels={}
for r in status['rows']:
 if 'label_id' not in r:continue
 at=int(r['literal_offset'],16)
 assert struct.unpack_from('<I',b,at)[0]+r['literal_adjustment']==int(r['label_id'],16)
 labels[str(r['native_code'])]=r['text']['1']
label_asset=ROOT/'assets/ui/sms-operation-labels.json';label_asset.write_text(json.dumps(labels,ensure_ascii=False,indent=2)+'\n')
(ROOT/'docs/evidence/sms-lifecycle.json').write_text(json.dumps({
 'source':str(src.relative_to(ROOT)),'source_sha256':sha,
 'verified_ranges':[prior[k] for k in ['function','error_handler','error_handler_continuation']],
 'comparison_inputs':65536,'accepted_rl_causes':accepted,
 'outputs':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [asset,label_asset,listing]},
 'method':'Fresh LLVM Thumb listing; bounded integer comparison evaluation stops at BX. No calls, stores, device access or mutable RAM.',
 'limits':['Error decision ordering is a reviewed bounded code translation, not a complete SMS sender.',
 'UI operation codes and modem error causes are separate namespaces.',
 'No-service UI, local folders/read/retry/delete and disk transactions are replica policy; no native conversation layout claim.']},ensure_ascii=False,indent=2)+'\n')
print(f'Verified 65536 inputs, {len(accepted)} retry causes and {len(labels)} operation labels.')
