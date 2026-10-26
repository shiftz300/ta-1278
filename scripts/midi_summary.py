"""Bounded SMF inspection; never executes firmware or sends MIDI events."""
import struct

def summarize(data):
    assert data[:4] == b'MThd' and len(data) <= 131072
    size, fmt, count, division = struct.unpack_from('>IHHH', data, 4)
    assert size >= 6 and 0 < division < 32768 and count <= 64
    cursor = 8 + size
    tracks = []; tempos = [(0, 500000)]; max_tick = 0
    for _ in range(count):
        assert data[cursor:cursor+4] == b'MTrk'
        size = int.from_bytes(data[cursor+4:cursor+8], 'big'); cursor += 8
        end = cursor + size; assert end <= len(data)
        tick = 0; running = None; names = []; programs = []; notes = 0
        def vlq():
            nonlocal cursor
            n = 0
            for _ in range(4):
                assert cursor < end
                b = data[cursor]; cursor += 1; n = (n << 7) | (b & 127)
                if not b & 128: return n
            raise ValueError('Overlong MIDI VLQ')
        while cursor < end:
            tick += vlq(); status = data[cursor]
            if status & 128: cursor += 1
            else: assert running is not None; status = running
            if status == 255:
                kind = data[cursor]; cursor += 1; length = vlq(); raw = data[cursor:cursor+length]; cursor += length
                if kind in (3,4): names.append(raw.decode('latin1'))
                if kind == 81: assert length == 3; tempos.append((tick, int.from_bytes(raw,'big')))
                if kind == 47: break
            elif status in (240,247):
                length = vlq(); cursor += length; running = None
            else:
                assert 128 <= status < 240; running = status
                length = 1 if status & 240 in (192,208) else 2
                raw = data[cursor:cursor+length]; cursor += length; assert len(raw) == length and max(raw) < 128
                if status & 240 == 192: programs.append([status & 15,raw[0]])
                if status & 240 == 144 and raw[1]: notes += 1
            assert cursor <= end
        max_tick = max(max_tick,tick); cursor = end
        tracks.append({'names':names,'programs':programs,'note_on':notes,'last_tick':tick})
    assert cursor == len(data), (cursor,len(data))
    last = 0; tempo = 500000; seconds = 0
    for tick,value in sorted(tempos):
        if tick > max_tick: break
        seconds += (tick-last)*tempo/(division*1000000); last=tick;tempo=value
    seconds += (max_tick-last)*tempo/(division*1000000)
    return {'format':fmt,'division':division,'tracks':tracks,'seconds_to_last_event':seconds}
