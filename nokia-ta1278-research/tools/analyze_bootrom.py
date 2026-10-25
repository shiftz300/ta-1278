#!/usr/bin/env python3
"""Analyze the acquired ROM locally. Never opens USB or executes sample code."""
import collections
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SAMPLE = ROOT / 'analysis/work/bootrom-20261025T053201+0800/bootrom.bin'
OUT = ROOT / 'analysis/bootrom'
EXPECTED = '1c633df8793137d49b50c3ebb853ba491a29186ae613af3a4ab33fe992b02e7c'
BASE = 0xffff0000

def main():
    data = SAMPLE.read_bytes()
    assert len(data) == 32768 and hashlib.sha256(data).hexdigest() == EXPECTED
    OUT.mkdir(exist_ok=True)
    strings = []
    for encoding, pattern in [('ascii', rb'[\x20-\x7e]{4,}'),
                              ('utf-16le', rb'(?:[\x20-\x7e]\x00){4,}')]:
        for m in re.finditer(pattern, data):
            strings.append({'offset': hex(m.start()), 'address': hex(BASE+m.start()),
                            'encoding': encoding, 'text': m.group().decode(encoding)})
    (OUT/'strings.json').write_text(json.dumps(strings, ensure_ascii=False, indent=2)+'\n')
    signatures = {'ELF': b'\x7fELF', 'PNG': b'\x89PNG\r\n\x1a\n',
                  'ZIP': b'PK\x03\x04', 'gzip': b'\x1f\x8b\x08',
                  'SquashFS_LE': b'hsqs', 'UBI': b'UBI#',
                  'DHTB': b'DHTB', 'SPRD3': b'SPRD3\x00'}
    hits = {name: [hex(m.start()) for m in re.finditer(re.escape(magic), data)]
            for name, magic in signatures.items()}
    descriptors = []
    for offset in range(len(data)-18):
        if data[offset:offset+2] == b'\x12\x01':
            fields = struct.unpack_from('<BBHBBBBHHHBBBB', data, offset)
            if fields[2] in (0x110, 0x200, 0x210, 0x300) and fields[6] in (8,16,32,64):
                descriptors.append({'offset': hex(offset), 'vid': hex(fields[7]),
                                    'pid': hex(fields[8]), 'bcdDevice': hex(fields[9]),
                                    'bytes': data[offset:offset+18].hex()})
    # The verified sample's USB configuration chain, bounded by wTotalLength.
    config_start = 0x6e94
    total = struct.unpack_from('<H', data, config_start+2)[0]
    offset = config_start
    config = []
    assert config_start+total <= len(data)
    while offset < config_start+total:
        length, kind = data[offset:offset+2]
        assert length >= 2 and offset+length <= config_start+total
        config.append({'offset': hex(offset), 'type': kind, 'length': length,
                       'bytes': data[offset:offset+length].hex()})
        offset += length
    (OUT/'usb-config-descriptors.json').write_text(json.dumps(config,indent=2)+'\n')
    entropy = []
    for off in range(0, len(data), 1024):
        block = data[off:off+1024]
        counts = collections.Counter(block)
        h = -sum(n/len(block)*math.log2(n/len(block)) for n in counts.values())
        entropy.append({'offset': hex(off), 'entropy_bits_per_byte': round(h,5)})
    # Synthetic ELF metadata only: the ROM is unchanged at file offset 0x100.
    # This is an analysis wrapper, not an ELF found on the device or a runnable OS.
    names = b'\0.rom\0.shstrtab\0'
    shoff = (0x100+len(data)+len(names)+3)&~3
    ident = b'\x7fELF\x01\x01\x01'+bytes(9)
    header = struct.pack('<16sHHIIIIIHHHHHH', ident, 2,40,1,BASE,0,shoff,
                         0x05000000,52,0,0,40,3,2)
    elf = header.ljust(0x100,b'\0')+data+names
    elf = elf.ljust(shoff,b'\0')+bytes(40)
    elf += struct.pack('<IIIIIIIIII',1,1,6,BASE,0x100,len(data),0,0,4,0)
    elf += struct.pack('<IIIIIIIIII',6,3,0,0,0x100+len(data),len(names),0,0,1,0)
    wrapper = OUT/'bootrom.analysis-only.elf'
    wrapper.write_bytes(elf)
    commands = [(['file',str(SAMPLE)], 'file.txt'),
                (['xxd','-l','256',str(SAMPLE)], 'header-256.txt'),
                (['xcrun','llvm-objdump','-d','--triple=armv7a-none-eabi',
                  '--start-address=0xffff0000','--stop-address=0xffff00a4',str(wrapper)],
                 'entry-arm.txt'),
                (['xcrun','llvm-objdump','-d','--triple=thumbv7a-none-eabi',
                  '--start-address=0xffff1fb6','--stop-address=0xffff1fe4',str(wrapper)],
                 'entry-call-thumb.txt')]
    runs=[]
    for argv, name in commands:
        result = subprocess.run(argv, capture_output=True, text=True)
        (OUT/name).write_text(result.stdout+result.stderr)
        runs.append({'argv':argv,'returncode':result.returncode,'output':name})
    summary={'sample':str(SAMPLE.relative_to(ROOT)), 'size':len(data),
             'sha256':EXPECTED,'base':hex(BASE), 'ascii_utf16_candidate_count':len(strings),
             'signature_hits_not_validated_containers':hits,
             'usb_device_descriptor_candidates':descriptors,'entropy_1KiB':entropy,
             'last_nonzero_offset':hex(max(i for i,b in enumerate(data) if b)),
             'zero_bytes':data.count(0),'ff_bytes':data.count(255),
             'commands':runs,'wrapper_is_synthetic':True}
    (OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps({k:v for k,v in summary.items() if k not in ['entropy_1KiB','commands']},indent=2))

if __name__ == '__main__':
    main()
