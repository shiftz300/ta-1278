#!/usr/bin/env python3
"""Export native application windows/menu bytes and associated resource candidates.
No execution of target code, no private storage inputs, no network access.
"""
import hashlib,json,struct
from pathlib import Path
R=Path(__file__).resolve().parents[1]; O=R/'analysis/ui/apps'; BASE=0x88000000
DIGEST='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
def write(n,x): (O/n).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def main():
 b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes();assert hashlib.sha256(b).hexdigest()==DIGEST
 ss=json.loads((O.parent/'strings.json').read_text());text={}
 for x in ss:text.setdefault(int(x['resource_id'],16),{})[x['locale']]=x['text']
 menus=[]
 for name,at,count in [('page',0x73e80c,8),('open',0x73e84c,5),('navigation',0x73e874,4),('save',0x73e894,4),('start',0x73e8b4,3)]:
  rows=[]
  for i in range(count):
   off=at+i*8;action,label=struct.unpack_from('<II',b,off);assert action<40 and label in text
   rows.append({'offset':hex(off),'action':action,'label_id':hex(label),'text':text[label]})
  menus.append({'menu':name,'offset':hex(at),'entry_bytes':8,'sha256':hashlib.sha256(b[at:at+count*8]).hexdigest(),'rows':rows})
 # Source IDs are only named when anchored by native menu labels/action branches.
 names={3:'settings',5:'open submenu',7:'share link',8:'bookmarks',9:'history',10:'downloads',11:'offline pages',13:'close',14:'enter URL',15:'home page',16:'stop',17:'forward',18:'back',19:'refresh',20:'set home page',21:'save bookmark',22:'save webpage',23:'page details',28:'navigation submenu',29:'save submenu',31:'save image'}
 tables=[]
 for name,at,count,bias in [('browser-page-actions',0xb636a4,30,3),('sms-insert-actions',0x39e712,12,0x37)]:
  rows=[{'action':i+bias,'target':hex(BASE+at+v*2),'table_byte':v,'meaning':names.get(i+bias,'unresolved') if name.startswith('browser') else 'unresolved'} for i,v in enumerate(b[at:at+count])]
  tables.append({'name':name,'offset':hex(at),'sha256':hashlib.sha256(b[at:at+count]).hexdigest(),'rows':rows})
 # SMS uses a different dispatch encoding: ADD PC to a u32 relative table.
 # 39c0bc compares ID with 0x89; 39c0c8..cc loads base+4*ID then adds PC.
 at=0x39c0d0;count=0x89
 sms_names={4:'insert smiley',5:'insert symbol',6:'insert template',7:'insert submenu',8:'add picture',11:'add contact card',12:'attachments submenu',23:'remove attachments',32:'save as draft',36:'clear text'}
 tables.append({'name':'sms-body-actions','offset':hex(at),'entry_bytes':4,'sha256':hashlib.sha256(b[at:at+count*4]).hexdigest(),'rows':[{'action':i,'target':hex(BASE+at+struct.unpack_from('<I',b,at+4*i)[0]),'meaning':sms_names.get(i,'unresolved')} for i in range(count)]})
 base=struct.unpack_from('<I',b,0x398cec)[0];assert base==0x400800c0
 dynamic=[]
 for action,parent,delta,call in [(7,0,0,0x3989bc),(4,7,2,0x398a04),(5,7,1,0x398a24),(6,7,3,0x398a44),(12,0,4,0x398a62),(11,12,6,0x398ab0),(23,0,7,0x398ae2),(36,0,63,0x398b0e),(32,0,66,0x398b6e)]:
  dynamic.append({'action':action,'parent_action':parent,'label_id':hex(base+delta),'text':text[base+delta],'insert_call':hex(BASE+call)})
 label=struct.unpack_from('<I',b,0x398cf0)[0]
 dynamic.append({'action':8,'parent_action':12,'label_id':hex(label),'text':text[label],'insert_call':'0x88398a94'})
 write('native-menus.json',{'source_sha256':DIGEST,'scope':'Browser arrays are 2 u32 per item, not 3. Order and IDs native; grouping corroborated by fixed W19 source. SMS nodes recovered from construction call arguments; public MMS enum numbers demonstrably differ. Dynamic visibility/enabled state still needs runtime guards.','menus':menus,'sms_body_menu_nodes':dynamic,'action_switches':tables})
 errors=[]
 error_labels={0:(0x399154,0),1:(0x399158,0),2:(0x399158,0x3d),3:(0x39915c,0),4:(0x399154,0x83),7:(0x399160,0),12:(0x39914c,-0xae)}
 for i,v in enumerate(b[0x398dfc:0x398e0a]):
  row={'native_code':i,'target':hex(0x88398dfc+2*v),'status':'unresolved'}
  if i in error_labels:
   pointer,delta=error_labels[i];label=struct.unpack_from('<I',b,pointer)[0]+delta
   row.update(label_id=hex(label),text=text[label],literal_offset=hex(pointer),literal_adjustment=delta,status='native label branch')
  elif i==13:row.update(status='delegated helper',callee='0x8858c61c')
  else:row['status']='log-only/default branch in this display helper'
  errors.append(row)
 write('sms-operation-status.json',{'source_sha256':DIGEST,'handler':'0x88398dd0','scope':'Display-helper operation codes, NOT modem/network error numbers. Code 0 is progress, not failure. No claim about subsequent transport outcome.','rows':errors})
 repeat_labels=[struct.unpack_from('<I',b,0x68a228)[0]+0x5d,struct.unpack_from('<I',b,0x68a22c)[0],struct.unpack_from('<I',b,0x68a22c)[0]-0x82]
 write('music-repeat-settings.json',{'source_sha256':DIGEST,'builder':'0x8868a1aa','selection_read':'0x886861f8','selection_write':'0x8865c9a0','rows':[{'native_value':i,'label_id':hex(v),'text':text[v]} for i,v in enumerate(repeat_labels)],'scope':'Three native list entries, getter 0/1/2 maps directly to row 0/1/2, OK handler writes selected row. Transport repeat behavior is not proven by labels alone.'})
 windows=json.loads((O.parent/'window-semantics.json').read_text())['windows'];images=json.loads((O.parent/'image-catalog.json').read_text());apps={}
 for name,groups in [('sms',(7,8)),('browser',(6,)),('music',(14,))]:
  ws=[x for x in windows if int(x['window_id'],16)>>16 in groups]
  # Keep resource classes separate: same numeric ID may be text AND image.
  ts=[x for x in ss if (int(x['resource_id'],16)>>16)&0xfff in groups]
  ims=[x for x in images if x.get('group') in groups]
  apps[name]={'window_group_ids':groups,'window_count':len(ws),'unique_window_ids':len(set(x['window_id'] for x in ws)),'windows':ws,'strings_by_resource_group':ts,'images_by_resource_group':ims,'note':'Group membership is a candidate resource set, not proof of screen placement. Common group 0 assets/fonts are shared; same ID across resource classes must not be conflated.'}
 write('application-resources.json',{'source_sha256':DIGEST,'apps':apps})
 lines=['# 应用资源入口','','本索引只使用系统镜像。各应用共有字体/图标见上一级 catalog.html。图像按资源组归类，不能据此推定它在某屏幕上的位置。','', '| 应用 | 描述片段 | 唯一窗口 ID | 文案记录（三语言合计） | 图像记录 |','|---|---:|---:|---:|---:|']
 for name,a in apps.items():lines.append(f"| {name} | {a['window_count']} | {a['unique_window_ids']} | {len(a['strings_by_resource_group'])} | {len(a['images_by_resource_group'])} |")
 lines+=['','完整偏移、处理函数、控件属性、三语言文案和已提取图片路径见 [application-resources.json](application-resources.json)。浏览器 24 个菜单项见 [native-menus.json](native-menus.json)。','', '资源 JSON 的图片 source/preview 路径沿用上一级 analysis/ui 为基准。原始图像已在 analysis/extracted/resources，预览在 analysis/extracted/png；本索引不制造第二套副本。']
 (O/'resources.md').write_text('\n'.join(lines)+'\n')
 print({k:(v['window_count'],len(v['strings_by_resource_group']),len(v['images_by_resource_group'])) for k,v in apps.items()})
if __name__=='__main__':main()
