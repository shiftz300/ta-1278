#!/usr/bin/env python3
"""Assemble source-addressed interaction evidence; no guessed event names."""
from pathlib import Path
import json
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui'
def main():
 windows=json.loads((O/'window-descriptors.json').read_text())['windows'];chosen=[x for x in windows if x['window_id'] in ('0x7000c','0x70010','0x70011','0x70026')];compose=next(x for x in chosen if x['window_id']=='0x7000c');nodes=[]
 for a in compose['attributes']:
  op=int(a['opcode'],16);args=a['arguments']
  if 0xf100<=op<=0xf1ff:
   assert len(args)>=4
   nodes.append({'opcode':a['opcode'],'node':args[-2],'parent':args[-1],'parameters':args[:-2]})
 known={'0x70071'}|{x['node'] for x in nodes};assert all(x['parent'] in known for x in nodes)
 events={0x1:0x399cdc,0x500:0x39acc0,0x515:0x399ed2,0x800:0x399ed8,0xe001:0x39a0a0,0xe002:0x399ed4,0xe004:0x3999da,0xe017:0x399996,0xe019:0x399998,0xe01a:0x399cce,0xe030:0x399994,0xe060:0x39999c,0xf021:0x399908,0xf022:0x399ed6,0xf024:0x3999d4,0xf025:0x3999d6,0xf026:0x399f54,0xf02a:0x39999e,0xf02b:0x3999d8,0xfa03:0x39a0e8,0xfa04:0x39adc0,0xfa16:0x39a0a0,0xfa17:0x399ed4,0xfa1b:0x3999da,0xfa68:0x39a0a0}
 result={'scope':'Partial reconstructed logic, not complete phone behavior','source':'user.bin SHA256 8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca','message_windows':chosen,'compose_layout':{'source_offset':'0x227790','root':'0x70071','nodes':nodes,'interpretation':'Last two control operands form node/parent tree; geometry/type parameter semantics unresolved'},'compose_dispatch':{'handler':'0x8839962a','branches':[{'event':hex(k),'block':hex(0x88000000+v)} for k,v in events.items()],'event_names':'unresolved; do not label raw IDs as physical keys','evidence':'message-compose-window-thumb.txt, 0x8839968e..0x88399906'},'multitap':{'idle':'last_key=12, cycle=0','same_key':'increment 1-based cycle; wrap to 1 after table count','explicit_commit':'reset last_key/cycle','timer_argument':750,'timer_unit':'unresolved','table_count':147,'reference_model':'../../tools/interaction_reference.py'},'editor_fields':{'text_pointer':'+0x90','length_u16':'+0x94','cursor_u16':'+0xae','selection_u16':['+0xc2','+0xc4'],'range_list':'+0x240'},'known_gaps':['Runtime menu visibility/order','Physical confirm/power/long-press mapping','Event ID names and complete handler side effects','Predictive dictionary and Chinese candidate algorithm','SMS encoding selection and visible character/segment counter','Window stack/focus/scroll/return state','Font native scaling/rasterizer equivalence','Browser history and network transitions']}
 (O/'interaction-map.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(len(nodes),'layout nodes',len(events),'raw event branches')
if __name__=='__main__':main()
