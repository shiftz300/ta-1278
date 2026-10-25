#!/usr/bin/env python3
"""Offline, bounds-checked TA-1278 SRES strings and SFON/cmap extraction.
Input is the verified logical system work copy, never user/NV partitions.
SFON bodies are preserved verbatim, not misrepresented as TrueType fonts.
"""
from pathlib import Path
import collections, hashlib, json, re, struct

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'analysis/ui'
SRC = ROOT / 'analysis/work/system-20261025T054301+0800/mmi.bin'

def sha(b): return hashlib.sha256(b).hexdigest()
def savej(p, obj): p.write_text(json.dumps(obj, ensure_ascii=False, indent=2) + '\n')

def cmap4(b, off):
    version, num = struct.unpack_from('>HH', b, off)
    assert version == 0 and num < 32
    result = {}
    for i in range(num):
        platform, encoding, rel = struct.unpack_from('>HHI', b, off+4+i*8)
        p = off+rel
        fmt, length, language, twice = struct.unpack_from('>4H', b, p)
        assert fmt == 4 and length >= 16 and p+length <= len(b)
        n = twice//2
        ends = struct.unpack_from('>'+str(n)+'H', b, p+14)
        starts_at = p+16+2*n
        starts = struct.unpack_from('>'+str(n)+'H', b, starts_at)
        deltas = struct.unpack_from('>'+str(n)+'H', b, starts_at+2*n)
        ranges_at = starts_at+4*n
        ranges = struct.unpack_from('>'+str(n)+'H', b, ranges_at)
        for j, (start, end, delta, ran) in enumerate(zip(starts, ends, deltas, ranges)):
            assert start <= end
            for cp in range(start, end+1):
                if cp == 0xffff: continue
                if ran:
                    at = ranges_at+2*j+ran+2*(cp-start)
                    # All three SFONs store a format-4 length two bytes short
                    # of the final glyph word. Bound by both that observed
                    # exception and the independently stored body boundary.
                    body = struct.unpack_from('>I', b, 40)[0]
                    assert p <= at and at+2 <= min(p+length+2, body)
                    gid = struct.unpack_from('>H', b, at)[0]
                    # Firmware 0x88563b72..0x88563b86 returns the range
                    # array value directly, without applying idDelta.
                else: gid = (cp+delta)&65535
                if gid: result[cp] = gid
    return [{'codepoint':cp, 'unicode':f'U+{cp:04X}', 'character':chr(cp), 'glyph_id':gid}
            for cp,gid in sorted(result.items())]

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT/'fonts').mkdir(exist_ok=True)
    b = SRC.read_bytes()
    assert b[:8] == b'sres0001'
    strings, fonts, errors, groups = [], [], [], []
    for ti in range(33):
        tb, size = struct.unpack_from('<II', b, 16+ti*8)
        if not size: continue
        assert b[tb:tb+8] == b'tres0001'
        end = struct.unpack_from('<I', b, tb+16)[0]
        for group in range(64):
            rel = struct.unpack_from('<I', b, tb+20+group*4)[0]
            if not 0 < rel < end: break
            gb = tb+rel
            off,count,length = struct.unpack_from('<III', b, gb+3*12)
            if count and length:
                section = gb+off
                first = struct.unpack_from('<I', b, section)[0]
                locale_count = (gb+first-section)//8
                assert locale_count == 3
                groups.append({'tres':ti,'group':group,'count':count,'locales':locale_count,'group_offset':gb,'section_offset':section})
                for locale in range(locale_count):
                    table_rel, aux = struct.unpack_from('<II', b, section+locale*8)
                    table = gb+table_rel
                    assert section <= table and table+count*4 <= section+length
                    for index in range(count):
                        p = gb+struct.unpack_from('<I', b, table+index*4)[0]
                        assert section <= p < section+length
                        if b[p:p+2] == b'\x80\0':
                            n = struct.unpack_from('<H', b, p+2)[0]
                            assert n%2 == 0 and p+4+n <= section+length
                            raw=b[p+4:p+4+n]; enc='utf-16-le'; total=n+4
                        else:
                            e=b.find(b'\0', p, section+length)
                            assert e >= p
                            raw=b[p:e]; enc='ascii'; total=e-p+1
                        try: text=raw.decode(enc, errors='strict')
                        except UnicodeDecodeError as ex:
                            errors.append({'group':group,'index':index,'locale':locale,'offset':p,'error':str(ex)})
                            # Lossless byte preservation; Latin-1 display is
                            # provisional until the 8-bit code page is known.
                            text=raw.decode('latin-1'); enc='8-bit-latin1-provisional'
                        row={'tres':ti,'group':group,'index':index,'resource_id':hex(0x40000000+(group<<16)+index+1),'locale':locale,'offset':p,'offset_hex':hex(p),'encoded_bytes':total,'encoding':enc,'text':text}
                        if enc=='8-bit-latin1-provisional':row['raw_hex']=raw.hex()
                        strings.append(row)
            off,count,length = struct.unpack_from('<III', b, gb+4*12)
            if not count: continue
            for i in range(count):
                font_id, ptr = struct.unpack_from('<II', b, gb+off+i*8)
                variants=[struct.unpack_from('<II', b, gb+ptr+slot*8) for slot in range(3)]
                assert len(set(variants)) == 1
                n, relfont=variants[0]; start=gb+relfont; data=b[start:start+n]
                assert len(data)==n and data[:4]==b'sfon'
                header=struct.unpack_from('>11I', data)
                mapping=cmap4(data, header[5])
                stem=f'font-{font_id}-{data[8:12].decode("ascii")}'
                (OUT/'fonts'/f'{stem}.sfon').write_bytes(data)
                savej(OUT/'fonts'/f'{stem}-cmap.json', mapping)
                fonts.append({'id':font_id,'group':group,'locales':[0,1,2],'file':f'fonts/{stem}.sfon','offset':start,'offset_hex':hex(start),'size':n,'sha256':sha(data),'script':data[8:12].decode('ascii'),'header_u32_be':[hex(v) for v in header],'cmap_offset':header[5],'body_offset':header[10],'mapped_codepoints':len(mapping),'max_glyph_id':max(x['glyph_id'] for x in mapping),'body_decoded':False})
    savej(OUT/'strings.json',strings)
    savej(OUT/'string-groups.json',groups)
    savej(OUT/'fonts.json',fonts)
    savej(OUT/'decode-errors.json',errors)
    keyed=collections.defaultdict(dict)
    for row in strings: keyed[(row['tres'],row['group'],row['index'])][row['locale']]=row
    focus=re.compile(r'sms|message|inbox|outbox|draft|sent|write|input|predict|dictionary|cursor|insert|delete|character|text|menu|shortcut|keypad|keyboard|短信|信息|消息|收件|发件|草稿|文字|輸入|输入|字符|字元|光标|游標|删除|刪除|插入|选项|選項|菜单|功能表',re.I)
    focused=[]
    for key,rows in keyed.items():
        if any(focus.search(r['text']) for r in rows.values()):
            focused.append({'tres':key[0],'group':key[1],'index':key[2],'translations':{str(k):v['text'] for k,v in rows.items()},'offsets':{str(k):v['offset_hex'] for k,v in rows.items()}})
    savej(OUT/'menu-sms-texts.json',focused)
    for slot,label in enumerate(['en-observed','zh-Hans-observed','zh-Hant-observed']):
        rows=[r for r in strings if r['locale']==slot]
        savej(OUT/f'strings-{label}.json',rows)
    summary={'source':str(SRC.relative_to(ROOT)),'source_sha256':sha(b),'strings':len(strings),'string_groups':len(groups),'focused_resource_ids':len(focused),'fonts':fonts,'decode_errors':len(errors),'locale_labels':'0 English, 1 Simplified Chinese, 2 Traditional Chinese; inferred from decoded text, not locale identifier metadata'}
    savej(OUT/'extraction-summary.json',summary)
    print(json.dumps({k:v for k,v in summary.items() if k!='fonts'},ensure_ascii=False,indent=2))

if __name__=='__main__': main()
