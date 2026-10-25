#!/usr/bin/env python3
"""Reproducible offline UI tables + selected Thumb disassembly.
Creates analysis metadata, never runs instructions from the firmware.
"""
from pathlib import Path
import hashlib, json, re, struct, subprocess
from PIL import Image, ImageDraw

R=Path(__file__).resolve().parents[1]
O=R/'analysis/ui'
BASE=0x88000000
def save(name,obj): (O/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')

def main():
    b=(R/'analysis/work/system-20261025T054301+0800/user.bin').read_bytes()
    assert hashlib.sha256(b).hexdigest()=='8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca'
    strings=json.loads((O/'strings.json').read_text())
    resources=json.loads((R/'analysis/system/resource-index.json').read_text())
    ss={(x['group'],x['index'],x['locale']):x for x in strings}
    icons={(x['group'],x['index']):x for x in resources if x['kind']=='icon'}
    # Literal at 0x16d20c is loaded by 0x8816d0ba; loop uses 24-byte
    # stride and the constant count 30 loaded at 0x8816cfb0.
    start=struct.unpack_from('<I',b,0x16d20c)[0]-BASE
    assert start==0x21cf2c
    rows=[];canvas=Image.new('RGB',(720,600),'#e8e8e8');draw=ImageDraw.Draw(canvas)
    for i in range(30):
        at=start+i*24;v=struct.unpack_from('<6I',b,at)
        g=(v[1]>>16)&0xff;ix=(v[1]&65535)-1
        ig=(v[2]>>16)&0xff;ii=(v[2]&65535)-1
        icon=icons.get((ig,ii))
        row={'slot':i,'file_offset':hex(at),'app_id':hex(v[0]),'label_resource':hex(v[1]),'icon_resource':hex(v[2]),'secondary_resource':hex(v[3]),'handler_thumb':hex(v[4]),'handler_file_offset':hex((v[4]&~1)-BASE),'parameter':v[5],'labels':{str(l):ss.get((g,ix,l),{}).get('text') for l in range(3)},'label_group':g,'label_index':ix,'icon_group':ig,'icon_index':ii,'icon_source':icon['file'] if icon else None}
        assert all(row['labels'].values()) and BASE < (v[4]&~1)<BASE+len(b)
        if icon:
            png=R/'analysis/extracted/png'/(Path(icon['file']).stem+'.png')
            if png.exists():
                row['png']=str(png.relative_to(R/'analysis'))
                im=Image.open(png).convert('RGBA');x=(i%6)*120;y=(i//6)*120
                canvas.paste(im,(x+(120-im.width)//2,y+4),im)
                draw.text((x+4,y+86),row['labels']['0'][:18],fill='black')
                draw.text((x+4,y+101),f'app {v[0]:x} / {v[2]:x}',fill='black')
        rows.append(row)
    save('mainmenu-registry.json',{'note':'Compiled registry, not runtime visibility or menu order. Low16 resource index is one-based. Secondary resource semantics unresolved.','table_offset':hex(start),'stride':24,'count':30,'reference':'0x8816d0ba..0x8816d128','rows':rows})
    canvas.save(O/'mainmenu-contact-sheet.png')

    # Two registries selected by the real lookup functions at 0x8802f1f8
    # and 0x8802f220. The first loop is inclusive <=390, the second <390.
    registries=[];references={}
    for literal,count in [(0x2f53c,391),(0x2f540,390)]:
        table=struct.unpack_from('<I',b,literal)[0]-BASE
        entries=[]
        for i in range(count):
            at=table+i*8;code,ptr=struct.unpack_from('<II',b,at)
            st=ptr-BASE;assert 0<=st<len(b)-960
            item={'offset':hex(at),'mode_code':hex(code&65535),'reserved_u16':code>>16,'table_offset':hex(st)}
            entries.append(item);references.setdefault(st,[]).append(item)
        registries.append({'offset':hex(table),'entries':entries,'count':count})
    save('input-mode-registries.json',registries)
    tables=[]
    for st in sorted(references):
            keyrows=[]
            for j in range(12):
                at=st+j*80;n=struct.unpack_from('<H',b,at)[0]
                assert n<=39
                chars=b[at+2:at+2+n*2]
                text=chars.decode('utf-16le',errors='strict')
                keyrows.append({'row':j,'key':str(j+1) if j<9 else ('0' if j==9 else 'unresolved'),'count':n,'text':text,'code_units':list(struct.unpack('<'+str(n)+'H',chars)),'file_offset':hex(at),'nonzero_unused_padding':bool(any(b[at+2+n*2:at+80]))})
            tables.append({'offset':hex(st),'size':960,'sha256':hashlib.sha256(b[st:st+960]).hexdigest(),'mode_entries':references[st],'rows':keyrows,'locale_assignment':'unresolved'})
    save('multitap-tables.json',{'layout':'12 records, each u16 count + 39 UTF-16LE code units; code verified at 0x8802f33c and 0x8802f354','count':len(tables),'tables':tables})

    # Synthetic ELF is a container for LLVM only, not a recovered ELF.
    names=b'\0.text\0.shstrtab\0';so=(256+len(b)+len(names)+3)&~3
    h=struct.pack('<16sHHIIIIIHHHHHH',b'\x7fELF\x01\x01\x01'+bytes(9),2,40,1,BASE,0,so,0x5000000,52,0,0,40,3,2)
    e=(h.ljust(256,b'\0')+b+names).ljust(so,b'\0')+bytes(40)
    e+=struct.pack('<10I',1,1,6,BASE,256,len(b),0,0,4,0)+struct.pack('<10I',7,3,0,0,256+len(b),len(names),0,0,1,0)
    elf=O/'user.analysis-only.elf';elf.write_bytes(e)
    ranges=[('font-lookup',0x3216c,0x34800),('font-cmap',0x563970,0x563bbc),('multitap',0x1c8e3c,0x1c9450),('multitap-engine',0x481318,0x481a80),('keytable-core',0x2f1f8,0x2f580),('keytable-access',0xa77e20,0xa77f40),('input-timer',0x482ce4,0x482d2c),('cursor',0x2ff7e2,0x300070),('editor-delete',0x951fa,0x95264),('editor-character-pair',0x95f68,0x96060),('mainmenu',0x16cf9c,0x16d210),('mainmenu-launchers',0xbce00,0xbd040),('sms-content',0x3d92ec,0x3d9614),('sms-length',0xb50d34,0xb51080)]
    ranges += [('font-simple-outline',0x55f678,0x55fe64),('font-outline',0x55f800,0x560000),('font-render',0x560000,0x560c00),('window-name-lookup',0x5f22d0,0x5f2410),('message-compose-window',0x39962a,0x39af96)]
    commands=[]
    ranges += [('key-dispatch',0x2e2900,0x2e3700),('key-converter',0x13502e,0x135348),('t9-ldb-loader',0x54ca00,0x54cf94),('t9-ldb-init',0x54c600,0x54ca40),('browser-launchers',0xbcdf6,0xbce6c)]
    pattern=re.compile(rb'Opera Mini|NativeOperaMini|Dorado|DORADO|browser|brw_|T9(?:CC|AW)? |T9CCLoadLdb|dictionary|predict',re.I)
    evidence=[{'offset':hex(m.start()),'text':m[0].decode('ascii')} for m in re.finditer(rb'[\x20-\x7e]{7,}',b) if pattern.search(m[0])]
    save('browser-input-string-evidence.json',{'scope':'Static system image strings; existence does not prove active or reachable feature. No personal browsing or dictionary data.','strings':evidence})
    for name,lo,hi in ranges:
        argv=['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi','--mcpu=cortex-a5',f'--start-address={BASE+lo}',f'--stop-address={BASE+hi}',str(elf)]
        p=subprocess.run(argv,capture_output=True,text=True,check=True)
        (O/f'{name}-thumb.txt').write_text(p.stdout+p.stderr)
        commands.append({'name':name,'file_start':hex(lo),'file_end_exclusive':hex(hi),'argv':argv,'returncode':p.returncode})
    save('code-analysis.json',{'source_sha256':hashlib.sha256(b).hexdigest(),'base':hex(BASE),'base_includes_DHTB_header':True,'wrapper_synthetic':True,'warning':'Linear disassembly includes inline literals and strings. Only manually verified function ranges constitute code evidence.','commands':commands})
    print(json.dumps({'mainmenu_entries':len(rows),'multitap_tables':len(tables),'disassembly_excerpts':len(commands)}))

if __name__=='__main__': main()
