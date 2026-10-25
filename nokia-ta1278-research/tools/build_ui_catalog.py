#!/usr/bin/env python3
"""Local resource browser: no server, external assets, or network requests."""
from pathlib import Path
import hashlib, io, json, struct
from PIL import Image

R=Path(__file__).resolve().parents[1]; O=R/'analysis/ui'
def save(p,x):p.write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
def main():
    index=json.loads((R/'analysis/system/resource-index.json').read_text())
    extended=json.loads((O/'extra-image-extraction.json').read_text()) if (O/'extra-image-extraction.json').exists() else []
    extended+=json.loads((O/'sjpg-extraction.json').read_text()) if (O/'sjpg-extraction.json').exists() else []
    extended={x['source']:x for x in extended}
    symbol_rows=json.loads((O/'resource-symbols.json').read_text())['symbols'] if (O/'resource-symbols.json').exists() else []
    symbol_names={(x['group'],x['index']):x['name'] for x in symbol_rows if x['kind']==0}
    extra=[];images=[];native=O/'native-images';native.mkdir(exist_ok=True)
    for row in index:
        if row['kind'] not in ('icon','logo','animation'):continue
        source=R/'analysis/extracted/resources'/row['file'];b=source.read_bytes()
        stem=Path(row['file']).stem;png=R/'analysis/extracted/png'/(stem+'.png')
        rec=dict(row)
        if (row.get('group'),row.get('index')) in symbol_names:rec['original_name']=symbol_names[(row['group'],row['index'])]
        rec['source']='../extracted/resources/'+row['file']
        if b[:4]==b'user':
            rec['unresolved_reason']='Name-only user resource; dynamic drawing/binding path unresolved, no bitmap pixels in record.'
        rec['resource_id']=hex(0x40000000+(row.get('group',0)<<16)+row.get('index',0)+1) if 'group' in row else None
        if not png.exists() and (b[:4]==b'SGIF' or b[:2]==b'BM'):
            payload=b[12:] if b[:4]==b'SGIF' else b
            try:
                im=Image.open(io.BytesIO(payload));im.load()
                ext='.gif' if payload[:3]==b'GIF' else '.bmp'
                out=native/(stem+ext);out.write_bytes(payload)
                preview=native/(stem+'.png');im.convert('RGBA').save(preview)
                extra.append({'source':row['file'],'file':str(out.relative_to(O)),'sha256':hashlib.sha256(payload).hexdigest(),'frames':getattr(im,'n_frames',1),'width':im.width,'height':im.height})
                rec['preview']=str(preview.relative_to(O));rec['native']=str(out.relative_to(O))
                rec['decoded_width']=im.width;rec['decoded_height']=im.height
            except (OSError,ValueError) as e:rec['unresolved_reason']=str(e)
        if png.exists():rec['preview']='../extracted/png/'+png.name
        if row['file'] in extended:
            ex=extended[row['file']];rec['preview']=ex['frames'][0]['file'];rec['frames']=ex['frames'];rec['decoder_format']=ex['format']
            rec.pop('unresolved_reason',None)
            if 'channel_order' in ex:rec['channel_order']=ex['channel_order']
        rec['decoded']=bool(rec.get('preview'))
        images.append(rec)
    save(O/'image-catalog.json',images);save(O/'standard-image-extraction.json',extra)
    save(O/'unresolved-images.json',[{k:x.get(k) for k in ('file','kind','offset_hex','resource_id','unresolved_reason')} for x in images if not x['decoded']])
    menus=json.loads((O/'mainmenu-registry.json').read_text())['rows']
    ss=json.loads((O/'strings.json').read_text());grouped={}
    for row in ss:
        key=row['resource_id'];item=grouped.setdefault(key,{'id':key,'group':row['group'],'index':row['index'],'texts':{},'offsets':{},'encodings':{}})
        item['texts'][str(row['locale'])]=row['text'];item['offsets'][str(row['locale'])]=row['offset_hex'];item['encodings'][str(row['locale'])]=row['encoding']
    data=json.dumps({'menus':menus,'images':images,'texts':list(grouped.values())},ensure_ascii=False).replace('<','\\u003c')
    template='''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>TA-1278 固件资源目录</title>
<style>body{font:15px system-ui;margin:24px;background:#f6f6f4;color:#222}h1{font-size:25px}p{line-height:1.6;max-width:950px}nav{display:flex;gap:8px;flex-wrap:wrap}button,input,select{font:inherit;padding:8px;border:1px solid #aaa;border-radius:5px}button{cursor:pointer}button[aria-pressed=true]{background:#222;color:white}input{min-width:280px}#grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(165px,1fr));gap:12px;margin-top:16px}.card{background:white;border:1px solid #ddd;padding:12px;overflow-wrap:anywhere}.card img{display:block;max-width:100%;height:100px;object-fit:contain;margin:auto;image-rendering:pixelated}.card small{display:block;color:#555;margin-top:8px}table{border-collapse:collapse;width:100%;background:white;margin-top:16px}td,th{border:1px solid #ddd;text-align:left;padding:8px;white-space:pre-wrap;overflow-wrap:anywhere}code{font:12px monospace}a{color:#16548a}.hidden{display:none}footer{margin:24px 0}</style>
<h1>Nokia TA-1278 · 固件资源目录</h1><p>来自本机 17.00.17.02 系统工作副本，所有内容在本地加载。注册表含地区与可选应用，顺序不代表实机菜单。文字使用电脑系统字体显示，<b>不是手机字体预览</b>。SFON 全部字形已导出：<a href="fonts/font-preview.html">原始字形预览与字体下载</a>。SABM 原始像素的颜色通道顺序仍为暂定。</p>
<nav><button data-mode="menus" aria-pressed="true">菜单注册表</button><button data-mode="images">图像资源</button><button data-mode="texts">三语言文字</button><input id="query" placeholder="搜索名称、资源 ID、组号"><select id="lang"><option value="1">简体中文</option><option value="0">English</option><option value="2">繁體中文</option></select></nav><p id="status"></p><div id="grid"></div><table id="texttable" class="hidden"><thead><tr><th>资源 / 原始偏移</th><th>English</th><th>简体中文</th><th>繁體中文</th></tr></thead><tbody></tbody></table><button id="more">加载更多</button><footer><a href="recovery-report.md">分析报告</a> · <a href="fonts/font-preview.html">字体预览</a> · <a href="window-descriptors.json">窗口描述</a> · <a href="interaction-map.json">交互映射</a> · <a href="audio-index.json">音频索引</a> · <a href="multitap-tables.json">多击输入表</a> · <a href="../system/keymap.json">物理按键矩阵</a> · <a href="mainmenu-registry.json">菜单注册表 JSON</a></footer>
<script id="data" type="application/json">__DATA__</script><script>
const data=JSON.parse(document.getElementById('data').textContent);let mode='menus',limit=120;const $=s=>document.querySelector(s);function elem(tag,text){const e=document.createElement(tag);if(text!==undefined)e.textContent=text;return e}
function render(){const q=$('#query').value.toLowerCase(),lang=$('#lang').value;const all=data[mode];const filtered=all.filter(x=>JSON.stringify(x).toLowerCase().includes(q));$('#status').textContent=`共 ${filtered.length} 项，显示 ${Math.min(limit,filtered.length)} 项。图像目录包含未解码条目，可下载原始资源。`;$('#grid').replaceChildren();$('#texttable tbody').replaceChildren();$('#texttable').classList.toggle('hidden',mode!=='texts');$('#grid').classList.toggle('hidden',mode==='texts');for(const x of filtered.slice(0,limit)){if(mode==='texts'){let tr=elem('tr');tr.append(elem('td',x.id+' / '+Object.values(x.offsets).join(', ')));for(let i=0;i<3;i++)tr.append(elem('td',x.texts[i]));$('#texttable tbody').append(tr);continue}const c=elem('div');c.className='card';const src=mode==='menus'?(x.png?'../'+x.png:null):x.preview;if(src){let im=elem('img');im.src=src;im.loading='lazy';im.alt=mode==='menus'?x.labels[lang]:x.file;c.append(im)}else c.append(elem('p','尚未解码'));c.append(elem('strong',mode==='menus'?x.labels[lang]:`${x.original_name||x.resource_id||x.kind}`));c.append(elem('small',mode==='menus'?`app ${x.app_id} · ${x.handler_thumb}`:`组 ${x.group} / 索引 ${x.index} · ${x.width}×${x.height}`));c.append(elem('small',mode==='menus'?`文字 ${x.label_resource} · 图像 ${x.icon_resource}`:`mmi 偏移 ${x.offset_hex}`));const a=elem('a','原始资源');a.href=mode==='menus'?'../extracted/resources/'+x.icon_source:x.source;c.append(a);$('#grid').append(c)}$('#more').hidden=limit>=filtered.length}
document.querySelectorAll('[data-mode]').forEach(b=>b.onclick=()=>{mode=b.dataset.mode;limit=120;document.querySelectorAll('[data-mode]').forEach(n=>n.setAttribute('aria-pressed',n===b));render()});$('#query').oninput=()=>{limit=120;render()};$('#lang').onchange=render;$('#more').onclick=()=>{limit+=120;render()};render();</script></html>'''
    template=template.replace('<a href="recovery-report.md">分析报告</a>', '<a href="reverse-engineering-status.md">逆向交付与未恢复项</a> · <a href="interaction-spec.md">交互实现依据</a> · <a href="color-contact-sheet.png">主题颜色</a> · <a href="t9-resource-analysis.json">T9 词库索引</a> · <a href="key-event-map.json">按键事件</a>')
    (O/'catalog.html').write_text(template.replace('__DATA__',data))
    print(json.dumps({'image_entries':len(images),'previewable':sum(x['decoded'] for x in images),'additional_standard_images':len(extra),'strings_resource_ids':len(grouped)}))
if __name__=='__main__':main()
