"""Bounded diagnostic framing and commands; no generic command sender."""
import struct
import hashlib
from dataclasses import dataclass

HEADER = struct.Struct('<IHBB')
MAX_PACKET = 32776
# Diagnostic key aliases. Home softkeys follow configured lock-screen shortcuts;
# session 278 verified left=Internet and right=Messages on this test handset.
KEYS = {'UP': 4, 'DOWN': 5, 'LEFT': 6, 'RIGHT': 7, 'LSOFT': 8,
        'RSOFT': 9, 'CENTER': 13, 'STAR': 42, 'HASH': 35,
        **{str(i): 48+i for i in range(10)}}


@dataclass(frozen=True)
class Packet:
    seq: int
    kind: int
    subtype: int
    payload: bytes = b''

    def encode(self):
        raw = HEADER.pack(self.seq, 8+len(self.payload), self.kind, self.subtype)+self.payload
        if len(raw) > MAX_PACKET:
            raise ValueError('Packet too large')
        return b'\x7e'+raw.replace(b'\x7d', b'\x7d\x5d').replace(b'\x7e', b'\x7d\x5e')+b'\x7e'

    @classmethod
    def decode(cls, raw):
        if not 8 <= len(raw) <= MAX_PACKET:
            raise ValueError('Invalid packet size')
        seq, length, kind, subtype = HEADER.unpack_from(raw)
        if length != len(raw):
            raise ValueError('Header length mismatch')
        return cls(seq, kind, subtype, raw[8:])


class Framer:
    """Accept arbitrary USB boundaries; discard oversized frames until delimiter."""
    def __init__(self):
        self.buf = bytearray()
        self.active = False
        self.escape = False
        self.bad = False
        self.rejected = 0

    def feed(self, data):
        result = []
        for value in data:
            if value == 0x7e:
                if self.active and (self.buf or self.bad or self.escape):
                    try:
                        if self.bad or self.escape:
                            raise ValueError('Broken frame')
                        result.append(Packet.decode(bytes(self.buf)))
                    except ValueError:
                        self.rejected += 1
                self.buf.clear()
                self.active, self.escape, self.bad = True, False, False
            elif self.active and not self.bad:
                if self.escape:
                    if value not in (0x5d, 0x5e):
                        self.bad = True
                    else:
                        self.buf.append(value ^ 0x20)
                    self.escape = False
                elif value == 0x7d:
                    self.escape = True
                else:
                    self.buf.append(value)
                if len(self.buf) > MAX_PACKET:
                    self.buf.clear()
                    self.bad = True
        return result


def version_request(seq):
    return Packet(seq, 0, 0)


def screenshot_request(seq):
    return Packet(seq, 0x33, 0)


def key_request(seq, name, event='tap'):
    if name not in KEYS or event not in ('tap', 'down', 'up'):
        raise ValueError('Unsupported key/event (no call or power key)')
    # Native handler reads uint32 key at request+12. The preceding four payload
    # bytes are the legacy signal header; this handler does not consume them.
    return Packet(seq, 1, {'tap': 0, 'down': 1, 'up': 2}[event],
                  struct.pack('<HHI', 0, 0, KEYS[name]))


class Screenshot:
    def __init__(self, seq):
        self.seq = seq
        self.total = None
        self.parts = []
        self.raw = None
        self.header_repair = None

    def add(self, p):
        if p.seq != self.seq or p.kind != 0xa0:
            raise ValueError('Wrong screenshot transaction')
        if not p.payload or len(p.payload) > 32768:
            raise ValueError('Invalid image chunk')
        index = len(self.parts)+1
        if index == 1:
            if not p.subtype & 0x80:
                raise ValueError('Missing image start')
            self.total = p.subtype & 0x7f
            if not 1 <= self.total <= 16:
                raise ValueError('Invalid image chunk count')
        elif index > self.total or p.subtype != (0 if index == self.total else index):
            raise ValueError('Missing, duplicate or reordered image chunk')
        if index < self.total and len(p.payload) != 32768:
            raise ValueError('Short non-final chunk')
        self.parts.append(p.payload)
        if index == self.total:
            data = b''.join(self.parts)
            self.raw = data
            if (data[:40] == bytes(40) and
                    hashlib.sha256(data[40:623]).hexdigest() ==
                    'd8ec273f397eb3e957120afd942c0ec12e7b0786534c84460432fbf8210ca3aa'):
                # A second observed variant clears 40 leading bytes. Recover
                # only this exact prefix from the previously acquired intact
                # quality-95 DQT, also matched to kernel tables at bc6820 and
                # bc6960. Require the ENTIRE remaining header (through SOS)
                # to match. Entropy data is never synthesized or altered.
                data = b'\xff\xd8' + bytes.fromhex(
                    'ffdb004300020101010101020101010202020202') + data[40:]
                self.header_repair = 'recover_40_zero_prefix_with_verified_native_DQT'
            # Observed on 17.00.17.02: SOI/APP0's 20-byte prefix is zeroed
            # or partly overwritten. Require the observed DQT/SOF layout and
            # exact 240x320 frame before replacing this prefix. All tables,
            # entropy and EOI remain untouched; preserve raw bytes separately.
            elif (not data.startswith(b'\xff\xd8') and
                    data[20:25] == b'\xff\xdb\x00C\x00' and
                    data[89:94] == b'\xff\xdb\x00C\x01' and
                    data[158:167] == b'\xff\xc0\x00\x11\x08\x01\x40\x00\xf0'):
                data = b'\xff\xd8' + data[20:]
                self.header_repair = 'replace_damaged_20_byte_prefix_with_SOI'
            elif (not data.startswith(b'\xff\xd8') and
                    hashlib.sha256(data[48:623]).hexdigest() ==
                    '9848b8ae202194b98568b4c88318b5dfbcad3adddf932b436128e6cb0553ebe7'):
                # Observed allocator-like overwrite extends to byte 47 on
                # some screens. Same native table evidence as above; reject
                # any difference after this bounded metadata prefix.
                data = b'\xff\xd8' + bytes.fromhex(
                    'ffdb0043000201010101010201010102020202020403020202020504') + data[48:]
                self.header_repair = 'recover_48_byte_prefix_with_verified_native_DQT'
            elif (not data.startswith(b'\xff\xd8') and
                    hashlib.sha256(data[89:623]).hexdigest() ==
                    'bb3d843f8d03b58115ee3e5859eb8c4f2092dd8eb6ed1e7912bb91150c45fb6d'):
                # The first DQT can also be partially overwritten (observed
                # through byte 55). Restore that one fixed quality-95 table
                # only. Require the second DQT, SOF, all DHT and SOS to match
                # the prior intact native response exactly. No entropy edit.
                data = b'\xff\xd8' + bytes.fromhex(
                    'ffdb004300020101010101020101010202020202040302020202050404030406'
                    '0506060605060606070908060709070606080b08090a0a0a0a0a06080b0c0b'
                    '0a0c090a0a0a') + data[89:]
                self.header_repair = 'recover_first_DQT_with_verified_native_quality95'
            if not data.startswith(b'\xff\xd8') or not data.endswith(b'\xff\xd9'):
                raise ValueError('Incomplete JPEG')
            return data
        return None
