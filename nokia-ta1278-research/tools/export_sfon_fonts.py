#!/usr/bin/env python3
"""Export verified native SFON outlines to SVG sprites and derived OpenType CFF.
Derived metrics are nominal at 32 units; no claim of pixel-identical hinting.
"""
from pathlib import Path
import json,hashlib,html
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.qu2cuPen import Qu2CuPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'analysis/ui/fonts'
def contours(g,pen):
 start=0
 for end in g['contour_ends']:
  pts=g['points'][start:end+1];on=g['on_curve'][start:end+1];cub=g['cubic'][start:end+1];start=end+1
  if any(on):
   i=on.index(True);pts=pts[i:]+pts[:i];on=on[i:]+on[:i];cub=cub[i:]+cub[:i];origin=pts[0];seq=list(zip(pts[1:]+[origin],on[1:]+[True],cub[1:]+[False]))
  else:
   assert not any(cub);origin=[(pts[-1][a]+pts[0][a])/2 for a in (0,1)];seq=list(zip(pts+[origin],on+[True],cub+[False]))
  pen.moveTo(tuple(origin));controls=[];types=[]
  for pt,is_on,is_cubic in seq:
   if not is_on:controls.append(tuple(pt));types.append(is_cubic);continue
   if not controls:pen.lineTo(tuple(pt))
   elif any(types):
    assert all(types) and len(controls)==2
    pen.curveTo(*controls,tuple(pt))
   else:pen.qCurveTo(*controls,tuple(pt))
   controls=[];types=[]
  assert not controls;pen.closePath()
def draw(gid,gs,pen,stack=()):
 assert gid not in stack and len(stack)<16
 g=gs[gid]
 if 'components' not in g:return contours(g,pen)
 for c in g['components']:
  assert c['flags']&2,'point matching composite unsupported'
  xx,xy,yx,yy=c['matrix'];dx,dy=c['args']
  # Stored TrueType matrix order: xx, xy, yx, yy.
  draw(c['glyph_id'],gs,TransformPen(pen,(xx,xy,yx,yy,dx,dy)),stack+(gid,))
def main():
 summaries=[]
 for src in sorted(OUT.glob('*-outlines.json')):
  d=json.loads(src.read_text());assert not d['errors'] and not d['missing_codepoints'];gs={g['glyph_id']:g for g in d['glyphs']};stem=src.name.replace('-outlines.json','');cmap=json.loads((OUT/(stem+'-cmap.json')).read_text());em=d['units_per_em'];order=['.notdef']+[f'g{gid}' for gid in gs];chars={};metrics={};sprite=[];paths={}
  pen=T2CharStringPen(em/2,None);chars['.notdef']=pen.getCharString();metrics['.notdef']=(em//2,0)
  for gid,g in gs.items():
   name=f'g{gid}';adv=round(g['advance_32']*em/32);rec=RecordingPen();draw(gid,gs,rec)
   pen=T2CharStringPen(adv,None);rec.replay(Qu2CuPen(pen,max_err=0.01,all_cubic=True));chars[name]=pen.getCharString();metrics[name]=(adv,0)
   svg=SVGPathPen(None);rec.replay(svg);path=svg.getCommands();paths[gid]=path
   bp=BoundsPen(None);rec.replay(bp);xmin,ymin,xmax,ymax=bp.bounds or (0,0,em,em)
   sprite.append(f'<symbol id="{name}" viewBox="{xmin-1} {-ymax-1} {max(1,xmax-xmin)+2} {max(1,ymax-ymin)+2}"><path transform="scale(1,-1)" d="{path}"/></symbol>')
  fb=FontBuilder(em,isTTF=False);fb.setupGlyphOrder(order);fb.setupCharacterMap({x['codepoint']:f"g{x['glyph_id']}" for x in cmap});fb.setupHorizontalMetrics(metrics);fb.setupHorizontalHeader(ascent=em,descent=-em//4)
  family='TA1278 Extracted '+stem;ps='TA1278-'+stem
  fb.setupNameTable({'familyName':family,'styleName':'Regular','uniqueFontIdentifier':ps,'fullName':family,'psName':ps,'version':'Version 0.1; derived offline from SFON'})
  fb.setupOS2(sTypoAscender=em,sTypoDescender=-em//4,usWinAscent=em,usWinDescent=em//4);fb.setupPost();fb.setupCFF(ps,{'FullName':family,'FamilyName':family,'Weight':'Regular'},chars,{});fb.setupMaxp();fb.font['head'].created=fb.font['head'].modified=2082844800;otf=OUT/(stem+'.otf');fb.save(otf)
  (OUT/(stem+'-glyphs.svg')).write_text('<svg xmlns="http://www.w3.org/2000/svg"><defs>'+''.join(sprite)+'</defs></svg>')
  # Independent FreeType load/rasterization of derived font. Only generated font data, no firmware code.
  im=Image.new('RGB',(1000,310),'white');dr=ImageDraw.Draw(im);font=ImageFont.truetype(str(otf),30);big=ImageFont.truetype(str(otf),48)
  lines=['ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz 0123456789','Messaging Options Back Delete','信息 编辑 短信 返回 删除 输入法','訊息 編輯 簡訊 返回 刪除 輸入法']
  known={x['character'] for x in cmap}
  for y,line in enumerate(lines):dr.text((14,10+56*y),''.join(c if c in known or c==' ' else ' ' for c in line),font=big if y>=3 else font,fill='black')
  im.save(OUT/(stem+'-preview.png'))
  from fontTools.ttLib import TTFont
  check=TTFont(otf);assert len(check.getBestCmap())==len(cmap);assert len(check.getGlyphOrder())==len(gs)+1
  summaries.append({'font':stem,'glyphs':len(gs),'mapped_codepoints':len(cmap),'otf_sha256':hashlib.sha256(otf.read_bytes()).hexdigest(),'all_outlines_consumed':True,'freetype_rendered':True,'nominal_metrics':True,'native_hinting_recovered':False})
  print(stem,len(gs),'exported')
 (OUT/'font-preview.html').write_text('<!doctype html><meta charset="utf-8"><title>TA-1278 原始字形</title><style>@font-face{font-family:f0;src:url(font-0-latn.otf)}@font-face{font-family:f1;src:url(font-1-hani.otf)}@font-face{font-family:f2;src:url(font-2-hani.otf)}body{font:16px system-ui;margin:32px;background:#eee}article{background:white;padding:20px;margin:16px 0}textarea{width:90%;min-height:80px;font:20px f0,f1,f2}.sample{font-family:f0,f1,f2;line-height:1.5;white-space:pre-wrap}a{color:#164d8c}</style><h1>TA-1278 SFON 字形提取</h1><p>来自本机 17.00.17.02。全部 22,349 个字形记录已解析；OTF 为转换产物，字距使用标称 32 单位转换。浏览器抗锯齿、字距及回退规则不代表手机原始渲染。只在本地加载。</p><textarea id="text">信息 编辑 短信 返回 删除 输入法\n訊息 編輯 簡訊 返回 刪除 輸入法\nMessaging Options Back Delete 0123456789</textarea><article><p>16px 派生字体预览</p><div class="sample" style="font-size:16px"></div></article><article><p>20px 派生字体预览</p><div class="sample" style="font-size:20px"></div></article><article><p>24px 派生字体预览</p><div class="sample" style="font-size:24px"></div></article><article><p>32px 派生字体预览</p><div class="sample" style="font-size:32px"></div></article><p><a href="font-0-latn.otf">font-0-latn.otf</a> · <a href="font-0-latn-glyphs.svg">全部 SVG 字形</a> · <a href="font-0-latn-outlines.json">原始曲线坐标与偏移</a></p><p><a href="font-1-hani.otf">font-1-hani.otf</a> · <a href="font-1-hani-glyphs.svg">全部 SVG 字形</a> · <a href="font-1-hani-outlines.json">原始曲线坐标与偏移</a></p><p><a href="font-2-hani.otf">font-2-hani.otf</a> · <a href="font-2-hani-glyphs.svg">全部 SVG 字形</a> · <a href="font-2-hani-outlines.json">原始曲线坐标与偏移</a></p><script>function draw(){document.querySelectorAll(\'.sample\').forEach(x=>x.textContent=document.querySelector(\'#text\').value)}document.querySelector(\'#text\').oninput=draw;draw()</script>')
 (OUT/'export-validation.json').write_text(json.dumps(summaries,indent=2)+'\n')
if __name__=='__main__':main()
