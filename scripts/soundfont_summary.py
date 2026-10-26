"""Bounded, read-only SF2 structure/index inspection. Does not synthesize audio."""
import collections
import hashlib
import struct


def summarize(data):
    assert data[:4] == b'RIFF' and data[8:12] == b'sfbk'
    assert struct.unpack_from('<I', data, 4)[0] + 8 == len(data)
    chunks = {}
    groups = []

    def walk(start, end):
        while start < end:
            assert start + 8 <= end
            tag, size = struct.unpack_from('<4sI', data, start)
            assert start + 8 + size <= end
            yield tag.decode('ascii'), start, size
            start += 8 + size + (size & 1)
        assert start == end

    for tag, offset, size in walk(12, len(data)):
        assert tag == 'LIST' and size >= 4
        kind = data[offset+8:offset+12].decode('ascii')
        children = []
        for name, pos, count in walk(offset+12, offset+8+size):
            assert name not in chunks
            chunks[name] = data[pos+8:pos+8+count]
            children.append({'tag': name, 'offset': pos, 'size': count})
        groups.append({'type': kind, 'offset': offset, 'size': size, 'chunks': children})
    metadata = {k: chunks[k].rstrip(b'\0').decode('ascii') for k in ('INAM', 'isng', 'ISFT', 'ICOP', 'ICRD', 'IENG', 'ICMT')}
    assert struct.unpack('<HH', chunks['ifil']) == (2, 1)
    counts = {k: len(chunks[k]) // stride for k, stride in [('phdr',38),('pbag',4),('pmod',10),('pgen',4),('inst',22),('ibag',4),('imod',10),('igen',4),('shdr',46)]}
    for k, stride in [('phdr',38),('pbag',4),('pmod',10),('pgen',4),('inst',22),('ibag',4),('imod',10),('igen',4),('shdr',46)]:
        assert len(chunks[k]) % stride == 0
    text = lambda b: b.split(b'\0',1)[0].decode('ascii')
    presets = []
    for name, program, bank, bag, lib, genre, morph in struct.iter_unpack('<20sHHHIII', chunks['phdr'][:-38]):
        assert bag < counts['pbag'] - 1
        presets.append({'name':text(name),'bank':bank,'program':program,'bag':bag})
    assert {(r['bank'],r['program']) for r in presets} == {(0,i) for i in range(128)} | {(128,0)}
    assert chunks['phdr'][-38:-35] == b'EOP'
    samples = []
    sample_count = len(chunks['smpl'])//2
    assert len(chunks['smpl']) % 2 == 0
    for name,start,end,loop_start,loop_end,rate,key,correction,link,kind in struct.iter_unpack('<20sIIIIIBbHH',chunks['shdr'][:-46]):
        assert 0 <= start < end <= sample_count and start <= loop_start <= loop_end <= end
        assert 0 < rate <= 192000 and (key <= 127 or key == 255)
        samples.append({'name':text(name),'start':start,'end':end,'loop_start':loop_start,'loop_end':loop_end,'rate':rate,'root_key':key,'pitch_correction':correction,'link':link,'type':kind})
    assert chunks['shdr'][-46:-43] == b'EOS'
    mods = collections.Counter(struct.iter_unpack('<HHhHH',chunks['imod'][:-10]))
    assert all(transform == 0 for src,dest,amount,amount_src,transform in mods)
    return {'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'metadata':metadata,'groups':groups,'record_counts_including_terminals':counts,'presets':presets,'samples':samples,'instrument_modulators':[{'source':src,'destination':dest,'amount':amount,'amount_source':amount_src,'transform':transform,'count':count} for (src,dest,amount,amount_src,transform),count in sorted(mods.items())],'sample_rates':dict(sorted(collections.Counter(r['rate'] for r in samples).items()))}
