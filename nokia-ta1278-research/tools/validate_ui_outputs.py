#!/usr/bin/env python3
"""Validate extracted output against independent source byte ranges.
Does not execute firmware or access a connected phone.
"""
from pathlib import Path
import hashlib,json,re,struct,subprocess,tempfile
from PIL import Image
R=Path(__file__).resolve().parents[1];O=R/'analysis/ui'
def readj(p):return json.loads(p.read_text())
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work=R/'analysis/work/system-20261025T054301+0800'
    for component in readj(R/'analysis/system/components.json'):
        assert digest(work/(component['name']+'.bin'))==component['sha256']
    mmi=(work/'mmi.bin').read_bytes();user=(work/'user.bin').read_bytes()
    strings=readj(O/'strings.json')
    for row in strings:
        p=row['offset'];enc=row['encoding'];s=row['text']
        if enc=='utf-16-le':
            raw=s.encode('utf-16-le');assert mmi[p:p+2]==b'\x80\0'
            assert struct.unpack_from('<H',mmi,p+2)[0]==len(raw)
            assert mmi[p+4:p+4+len(raw)]==raw
        else:
            raw=s.encode('ascii' if enc=='ascii' else 'latin-1')
            assert mmi[p:p+len(raw)+1]==raw+b'\0'
            if enc!='ascii':assert row['raw_hex']==raw.hex()
    fonts=readj(O/'fonts.json')
    for row in fonts:
        raw=(O/row['file']).read_bytes()
        assert len(raw)==row['size'] and hashlib.sha256(raw).hexdigest()==row['sha256']
        assert raw==mmi[row['offset']:row['offset']+row['size']]
    tables=readj(O/'multitap-tables.json')['tables']
    for table in tables:
        p=int(table['offset'],16)
        assert hashlib.sha256(user[p:p+960]).hexdigest()==table['sha256']
        for row in table['rows']:
            at=int(row['file_offset'],16);n=row['count']
            assert struct.unpack_from('<H',user,at)[0]==n
            assert user[at+2:at+2+n*2]==row['text'].encode('utf-16-le')
    images=readj(O/'image-catalog.json');preview_count=0
    for row in images:
        assert (O/row['source']).is_file()
        if 'preview' in row:
            p=O/row['preview']
            with Image.open(p) as im:im.load();assert im.width>0 and im.height>0
            preview_count+=1
    abm=readj(R/'analysis/system/abm-decode.json')
    for row in abm:
        assert row['status']=='decoded'
        assert digest(R/'analysis/extracted/png'/row['file'])==row['sha256']
    menu=readj(O/'mainmenu-registry.json')['rows']
    for row in menu:
        values=struct.unpack_from('<6I',user,int(row['file_offset'],16))
        assert values[0]==int(row['app_id'],16) and values[4]==int(row['handler_thumb'],16)
        assert (R/'analysis'/row['png']).is_file()
    html=(O/'catalog.html').read_text()
    js=re.search(r'</script><script>(.*?)</script>',html,re.S).group(1)
    embedded=json.loads(re.search(r'<script id="data" type="application/json">(.*?)</script>',html,re.S).group(1))
    assert len(embedded['texts'])==7500
    assert 'http://' not in js and 'https://' not in js and 'fetch(' not in js
    with tempfile.TemporaryDirectory(prefix='ta1278-js-') as tmp:
        p=Path(tmp)/'catalog.js';p.write_text(js)
        subprocess.run(['node','--check',str(p)],capture_output=True,text=True,check=True)
    extended=readj(O/'extra-image-extraction.json')+readj(O/'sjpg-extraction.json');frame_count=0
    for item in extended:
        for frame in item['frames']:
            path=O/frame['file'];assert digest(path)==frame['sha256']
            with Image.open(path) as im:im.load();assert im.size==(frame['width'],frame['height'])
            frame_count+=1
    outlines=[readj(p) for p in sorted((O/'fonts').glob('*-outlines.json'))]
    for item in outlines:
        assert not item['errors'] and not item['missing_codepoints']
        assert len(item['glyphs'])==item['declared_glyph_count']
    exports=readj(O/'fonts/export-validation.json')
    for item in exports:assert digest(O/'fonts'/(item['font']+'.otf'))==item['otf_sha256']
    windows=readj(O/'window-descriptors.json')['windows']
    for item in windows:
        at=int(item['offset'],16);raw=user[at:at+item['size']]
        assert hashlib.sha256(raw).hexdigest()==item['sha256']
        assert list(struct.unpack('<'+str(len(raw)//4)+'I',raw))==[int(v,16) for v in item['words']]
    names=readj(O/'window-names.json')['names']
    for item in names:
        at=int(item['offset'],16);assert user[at:at+50].split(b'\0')[0].decode()==item['name']
    audio=readj(O/'audio-index.json')
    for item in audio:
        if 'file' in item:assert digest(O/item['file'])==item['sha256']
    data=readj(O/'data-resource-index.json')['entries']
    for item in data:
        raw=(O/item['file']).read_bytes();at=int(item['offset'],16)
        assert len(raw)==item['size'] and digest(O/item['file'])==item['sha256']
        assert raw==mmi[at:at+len(raw)]
    colors=readj(O/'color-resources.json');rectangles=readj(O/'rectangle-resources.json')
    for item in colors+rectangles:
        raw=bytes.fromhex(item['raw_hex']);at=int(item['offset'],16)
        assert raw==mmi[at:at+len(raw)]
    t9=readj(O/'t9-resource-analysis.json');payloads={x['index']:(O/x['file']).read_bytes() for x in data}
    for item in t9['alphabetic']:
        assert (O/item['file']).read_bytes()==b''.join(payloads[i] for i in item['data_indices'])
        assert digest(O/item['file'])==item['sha256']
    init=(R/'analysis/extracted/kernel/init_81c20000.bin').read_bytes()
    assert hashlib.sha256(init).hexdigest()==t9['init_sha256']
    for item in t9['chinese']:
        header=(O/item['header_file']).read_bytes();at=int(item['header_init_offset'],16)
        assert init[at:at+192]==header and hashlib.sha256(header).hexdigest()==item['header_sha256']
        if 'file' not in item:continue
        raw=(O/item['file']).read_bytes();assert raw[:192]==header and digest(O/item['file'])==item['sha256']
        for block in item['blocks']:
            at=struct.unpack_from('<I',raw,192+block['slot']*4)[0]
            assert raw[at:at+block['header_size']]==payloads[block['data_index']]
    keys=readj(O/'key-event-map.json');at=int(keys['table_offset'],16)
    assert user[at:at+105]==bytes.fromhex(keys['raw_hex'])
    result={'status':'passed','system_component_hashes':5,'string_roundtrips':len(strings),'native_font_byte_comparisons':len(fonts),'input_tables_verified':len(tables),'abm_png_hashes':len(abm),'image_previews_loaded':preview_count,'menu_rows':len(menu),'html_embedded_data_and_js_syntax':'passed','phone_access':False,'firmware_executed':False,'browser_interaction_tested':False,'extra_frame_hashes':frame_count,'font_outlines':sum(x['glyph_count'] for x in outlines),'derived_otf_hashes':len(exports),'window_fragments_verified':len(windows),'window_names_verified':len(names),'audio_hashes':sum('file' in x for x in audio)}
    result.update(data_source_byte_comparisons=len(data),color_records=len(colors),rectangle_records=len(rectangles),t9_rebuilt_containers=3,t9_chinese_headers=3,key_table_bytes=105)
    (O/'validation.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
