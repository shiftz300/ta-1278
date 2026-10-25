#!/usr/bin/env python3
"""Executable specification of the recovered multi-tap core, not firmware emulation.
Events use table slots, not unverified physical-to-logical key mappings.
Timer completion is explicit: native argument 750 has an unverified time unit.
"""
import json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
class MultiTap:
 def __init__(self,rows):self.rows=rows;self.committed='';self.pending=None
 @property
 def text(self):
  return self.committed+(self.rows[self.pending[0]][self.pending[1]] if self.pending else '')
 def commit(self):
  self.committed=self.text;self.pending=None
 def press_slot(self,slot):
  if not self.rows[slot]:return
  if self.pending and self.pending[0]==slot:self.pending=(slot,(self.pending[1]+1)%len(self.rows[slot]))
  else:self.commit();self.pending=(slot,0)
def main():
 tables=json.loads((R/'analysis/ui/multitap-tables.json').read_text())['tables'];table=next(t for t in tables if t['offset']=='0x21d592');rows=[r['text'] for r in table['rows']];m=MultiTap(rows)
 m.press_slot(1);assert m.text=='a';m.press_slot(1);assert m.text=='b';m.press_slot(2);assert m.text=='bd';m.commit();m.press_slot(2);assert m.text=='bdd'
 m=MultiTap(rows)
 for _ in range(len(rows[1])+1):m.press_slot(1)
 assert m.text=='a';m.commit();m.press_slot(1);assert m.text=='aa'
 z=MultiTap(rows);z.press_slot(9);assert z.text==' ';z.press_slot(9);assert z.text=='0';z.press_slot(9);assert z.text=='\n';z.commit();z.press_slot(10);assert z.text=='\n'
 result={'status':'passed','cases':['same-key replacement','different-key commit','explicit timeout commit','full-table cycle wrap','space/zero/LF cycle','empty slot no-op'],'source_table':table['offset'],'native_cycle':'0x884816be..0x8848173c','native_commit':'0x88481588','scope':'multi-tap reference model only; not end-to-end phone acceptance'}
 (R/'analysis/ui/interaction-reference-validation.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
if __name__=='__main__':main()
