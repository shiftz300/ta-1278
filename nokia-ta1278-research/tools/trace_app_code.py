#!/usr/bin/env python3
"""Selected application disassembly and literal references, never firmware execution."""
from pathlib import Path
import hashlib,json,re,struct,subprocess
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui/apps';BASE=0x88000000
RANGES=[
 ('sms-character-classification',0x3a481e,0x3a4b48),('sms-edit-count-update',0xb0f0e,0xb119a),
 ('sms-extension-alphabet',0xaca804,0xaca862),('sms-alphabet-membership',0xacb324,0xacb364),
 ('sms-transliteration',0x13bb42,0x13bb96),('sms-alphabet-management',0x3c7c1e,0x3c7c36),
 ('browser-input-click',0xbed86,0xbf088),('browser-form-reset',0xbea84,0xbebc6),
 ('browser-select',0xb482c2,0xb488b4),('browser-select-model',0x6296ea,0x629754),
 ('browser-select-change',0x629878,0x629898),
 ('music-play-mode',0x68629c,0x68630a),('music-play-end',0x600036,0x6002fa),
 ('music-manual-skip',0x6002fa,0x600424),('music-index-step',0x619b68,0x619c10),
 ('music-queue-step',0x61b3c0,0x61b55c),('music-unsupported-count',0x6010b6,0x6010d2),
 ('sms-ui-count',0x351b12,0x351c0c),('sms-ui-format',0x352232,0x352330),
 ('sms-ui-config',0x6240c8,0x624216),('sms-ui-convert',0xb0754,0xb0784),
 ('browser-field-input',0xb488b4,0xb48fb8),('browser-field-model',0x629754,0x629be0),
 ('browser-page-received',0xb65986,0xb659d4),
 ('music-back-state',0x600ea6,0x600ec6),('music-review-state',0x3f12a0,0x3f12b8),
 ('music-review-actions',0x3f25fc,0x3f2696),
 ('sms-retry-causes',0x3c5e68,0x3c5ecc),('browser-url-check',0x3b4724,0x3b4768),
 ('browser-auth-state',0x35c250,0x35c35e),
 ('browser-url-narrow',0x3a51dc,0x3a522c),
 ('music-view-install',0xace712,0xace7bc),('music-view-callback',0x68ac44,0x68ad94),
 ('music-center-key',0xacec7a,0xacecaa),('music-toggle',0x3f11ea,0x3f126a),
 ('music-play-pause-resume',0x5ff838,0x5ffd30),
 ('sms-send-results',0x3c3900,0x3c47a0),('sms-send-wait',0xc8400,0xc8c70),
 ('sms-encoding-content',0x3d92f4,0x3d9700),('sms-segment-capacity',0xb50d34,0xb51180),
 ('browser-url-submit',0xb65c54,0xb65f30),('browser-auth',0xb62ce0,0xb6361c),
 ('browser-credentials',0xb639b4,0xb63b90),
 ('sms-storage-flags',0x80100,0x802c0),('sms-draft-helpers',0x397d60,0x398020),
 ('browser-navigation-core',0xbda20,0xbda78),('browser-key-timers',0xb619fc,0xb61a78),
 ('music-transport-helpers',0xacecce,0xacf2de),('music-view-dispatch',0xad0a00,0xad1100),
 ('sms-compose-helpers',0x398020,0x39962a),('sms-compose',0x39962a,0x39af96),
 ('sms-options',0x39bf5a,0x39cb70),('sms-recipient-editor',0x39cb70,0x39d6d8),
 ('sms-text-editor',0x39d6d8,0x39d96e),('sms-insert-options',0x39e5b4,0x39f13c),
 ('browser-page',0xb61ff2,0xb62a68),('browser-address',0xb62a68,0xb62ce0),
 ('browser-options',0xb6361c,0xb639b4),('browser-start',0xb6483c,0xb649c4),
 ('browser-start-options',0xb649c4,0xb64e56),
 ('music-player',0xacf2de,0xad0a00),('music-settings',0x68825a,0x688f40),
 ('music-repeat-shuffle',0x689a9c,0x689bf6),('music-repeat-list',0x68a1aa,0x68a310),('music-track-list',0xad3f96,0xad46f0)]
def main():
 O.mkdir(exist_ok=True);b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
 assert hashlib.sha256(b).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
 strings=json.loads((R/'analysis/ui/strings.json').read_text());ss={}
 for x in strings:ss.setdefault(int(x['resource_id'],16),{})[x['locale']]=x['text']
 index=[]
 for name,lo,hi in RANGES:
  # Enable Thumb division decoding only for the two counter ranges. This is an
  # instruction decoder option, not evidence for a different physical CPU.
  extra=['--mattr=+hwdiv'] if name in ('sms-ui-count','sms-ui-format','music-index-step') else []
  out=subprocess.check_output(['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi','--mcpu=cortex-a5',*extra,f'--start-address={BASE+lo}',f'--stop-address={BASE+hi}',str(R/'analysis/ui/user.analysis-only.elf')],text=True)
  out=out.replace(str(R),'PROJECT_ROOT');(O/(name+'-thumb.txt')).write_text(out)
  refs=[];calls=[]
  for line in out.splitlines():
   m=re.match(r'([0-9a-f]{8}):.*?\t([a-z][a-z0-9.]*)\s+(.*)',line)
   if not m:continue
   pc=int(m[1],16);op=m[2];args=m[3];ref=None
   if op in ('bl','blx'):
    t=re.search(r'0x([0-9a-f]+)',args)
    if t:calls.append({'at':hex(pc),'target':hex(int(t[1],16))})
   if op.startswith('ldr') and '[pc,' in args:
    t=re.search(r'@ 0x([0-9a-f]+)',args)
    if t:
     at=int(t[1],16)-BASE
     if 0<=at<=len(b)-4:ref=struct.unpack_from('<I',b,at)[0]
   elif op.startswith('adr'):
    t=re.search(r'#(-?[0-9]+)',args)
    if t:ref=((pc+4)&~3)+int(t[1])
   if ref is None:continue
   row={'at':hex(pc),'value':hex(ref)}
   if ref in ss:row['resource_text']=ss[ref]
   if BASE<=ref<BASE+len(b):
    raw=b[ref-BASE:ref-BASE+300].split(b'\0')[0]
    if len(raw)>=5 and all(32<=c<127 for c in raw):row['ascii']=raw.decode()
   refs.append(row)
  index.append({'name':name,'file':name+'-thumb.txt','start':hex(BASE+lo),'end_exclusive':hex(BASE+hi),'source_range_sha256':hashlib.sha256(b[lo:hi]).hexdigest(),'literal_references':refs,'direct_call_candidates':calls})
 (O/'code-index.json').write_text(json.dumps({'scope':'Linear disassembly includes literal pools. Reference/call candidates are not a complete CFG or proven functions. Semantics require branch tracing.','ranges':index},ensure_ascii=False,indent=2)+'\n')
 print(len(index),'application disassembly ranges')
if __name__=='__main__':main()
