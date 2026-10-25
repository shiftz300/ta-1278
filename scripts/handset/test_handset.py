import io
import contextlib
import json
from pathlib import Path
import struct
import tempfile
import unittest
from PIL import Image
from .protocol import Packet, Framer, Screenshot, key_request
from .session import Session, region_hash, run_plan, validate_plan


def jpeg():
    stream = io.BytesIO()
    Image.new('RGB', (240, 320), (12, 34, 56)).save(stream, format='JPEG')
    return stream.getvalue()


class FakeUSB:
    def __init__(self, version=b'17.00.17.02', fail_down=False):
        self.requests = []
        self.pending = bytearray()
        self.version, self.fail_down = version, fail_down
        self.image = jpeg()

    def write(self, wire):
        request = Framer().feed(wire)[0]
        self.requests.append(request)
        if request.kind == 0:
            reply = Packet(request.seq, 0, 0, self.version)
        elif request.kind == 0x33:
            reply = Packet(request.seq, 0xa0, 0x81, self.image)
        elif request.kind == 1:
            if request.subtype == 1 and self.fail_down:
                raise OSError('simulated interrupted down write')
            reply = Packet(request.seq, 1, request.subtype)
        else:
            raise AssertionError('Unexpected command')
        self.pending.extend(reply.encode())

    def read(self, timeout_ms):
        data = self.pending[:37]
        del self.pending[:37]
        return data


class ProtocolTests(unittest.TestCase):
    def test_fragment_escape_coalesce(self):
        p = Packet(0x7e7d, 0xa0, 0x81, b'\x7e\x7d\x00')
        wire = b'noise'+p.encode()+p.encode()
        for split in range(len(wire)+1):
            f = Framer()
            self.assertEqual(f.feed(wire[:split])+f.feed(wire[split:]), [p, p])

    def test_malformed_and_oversized_recover(self):
        valid = Packet(1, 0, 0, b'ok')
        f = Framer()
        data = b'\x7e\x7d\x7e'+b'\x7e'+b'x'*40000+b'\x7e'+valid.encode()
        self.assertEqual(f.feed(data), [valid])
        self.assertEqual(f.rejected, 2)
        with self.assertRaises(ValueError):
            Packet.decode(struct.pack('<IHBB', 1, 9, 0, 0))

    def test_chunk_order_limits_and_jpeg(self):
        s = Screenshot(7)
        self.assertIsNone(s.add(Packet(7, 160, 0x82, b'\xff\xd8'+b'x'*32766)))
        with self.assertRaises(ValueError):
            s.add(Packet(7, 160, 2, b'\xff\xd9'))
        self.assertTrue(s.add(Packet(7, 160, 0, b'\xff\xd9')).endswith(b'\xff\xd9'))
        for p in [Packet(1, 160, 0x81, jpeg()), Packet(7, 160, 0x80, b'x'), Packet(7, 160, 0x91, b'x')]:
            with self.assertRaises(ValueError):
                Screenshot(7).add(p)

    def test_native_key_wire_layout_and_blocked_keys(self):
        p = key_request(1, 'LSOFT', 'down')
        self.assertEqual((p.kind, p.subtype), (1, 1))
        self.assertEqual(p.payload, bytes.fromhex('0000000008000000'))
        for name in ['DIAL', 'POWER', 'ERASE', '1;reset']:
            with self.assertRaises(ValueError):
                key_request(1, name)

    def test_native_zero_header_preserves_pixels_and_raw(self):
        original = jpeg()
        self.assertEqual(original[20:24], b'\xff\xdb\x00C')
        raw = bytes(20) + original[20:]
        capture = Screenshot(7)
        repaired = capture.add(Packet(7, 160, 0x81, raw))
        self.assertEqual(capture.raw, raw)
        self.assertEqual(Image.open(io.BytesIO(repaired)).tobytes(),
                         Image.open(io.BytesIO(original)).tobytes())
        overwritten = bytes(range(20)) + original[20:]
        self.assertEqual(Screenshot(7).add(Packet(7, 160, 0x81, overwritten)), repaired)
        for count in [19, 21]:
            with self.assertRaises(ValueError):
                Screenshot(7).add(Packet(7, 160, 0x81, bytes(count)+original[20:]))

    def test_native_40_byte_prefix_requires_matching_full_header(self):
        stream = io.BytesIO()
        Image.new('RGB', (240, 320), (80, 90, 100)).save(
            stream, format='JPEG', quality=95, subsampling=1)
        original = stream.getvalue()
        raw = bytes(40)+original[40:]
        repaired = Screenshot(7).add(Packet(7, 160, 0x81, raw))
        self.assertEqual(Image.open(io.BytesIO(repaired)).tobytes(),
                         Image.open(io.BytesIO(original)).tobytes())
        wrong = bytearray(raw)
        wrong[100] ^= 1
        with self.assertRaises(ValueError):
            Screenshot(7).add(Packet(7, 160, 0x81, bytes(wrong)))
        overwritten = bytes(range(48))+original[48:]
        repaired = Screenshot(7).add(Packet(7, 160, 0x81, overwritten))
        self.assertEqual(Image.open(io.BytesIO(repaired)).tobytes(),
                         Image.open(io.BytesIO(original)).tobytes())
        overwritten = bytes(range(56))+original[56:]
        repaired = Screenshot(7).add(Packet(7, 160, 0x81, overwritten))
        self.assertEqual(Image.open(io.BytesIO(repaired)).tobytes(),
                         Image.open(io.BytesIO(original)).tobytes())


class SessionTests(unittest.TestCase):
    def setUp(self):
        self.stdout = contextlib.redirect_stdout(io.StringIO())
        self.stdout.__enter__()
        self.temp = tempfile.TemporaryDirectory(prefix='ta1278-handset-test-')
        self.usb = FakeUSB()
        self.session = Session(self.usb, Path(self.temp.name)/'run', timeout=.05)

    def tearDown(self):
        self.session.close()
        self.temp.cleanup()
        self.stdout.__exit__(None, None, None)

    def test_wrong_version_never_captures_or_presses(self):
        self.usb.version = b'wrong version'
        with self.assertRaises(ValueError):
            self.session.verify()
        with self.assertRaises(ValueError):
            self.session.press('UP')
        self.assertEqual([p.kind for p in self.usb.requests], [0])

    def test_capture_then_guarded_plan(self):
        with Image.open(io.BytesIO(self.usb.image)) as img:
            expected = {'roi': [0, 24, 240, 296], 'sha256': region_hash(img, [0, 24, 240, 296])}
        plan = {'schema': 1, 'steps': [{'name': 'up', 'key': 'UP', 'before': expected, 'after': expected, 'settle_ms': 100}]}
        run_plan(self.session, plan)
        self.assertEqual([p.kind for p in self.usb.requests], [0, 51, 51, 1, 51])
        self.assertTrue((self.session.out/'up.png').exists())
        events = [json.loads(s) for s in (self.session.out/'events.jsonl').read_text().splitlines()]
        self.assertEqual(events[-1]['event'], 'plan_complete')

    def test_state_mismatch_sends_no_key(self):
        plan = {'schema': 1, 'steps': [{'name': 'up', 'key': 'UP', 'before': {'roi': [0, 0, 240, 320], 'sha256': '0'*64}}]}
        with self.assertRaisesRegex(ValueError, 'key not sent'):
            run_plan(self.session, plan)
        self.assertNotIn(1, [p.kind for p in self.usb.requests])

    def test_release_after_failed_down(self):
        self.session.verify(); self.session.capture('baseline')
        self.usb.fail_down = True
        with self.assertRaises(OSError):
            self.session.press('UP', 100)
        self.assertEqual([p.subtype for p in self.usb.requests if p.kind == 1], [1, 2])

    def test_stop_and_bad_jpeg(self):
        self.session.verify()
        self.usb.image = b'\xff\xd8bad\xff\xd9'
        with self.assertRaises(Exception):
            self.session.capture('bad')
        self.assertFalse(self.session.baseline)
        (self.session.out/'STOP').touch()
        count = len(self.usb.requests)
        with self.assertRaises(InterruptedError):
            self.session.capture('stopped')
        self.assertEqual(len(self.usb.requests), count)

    def test_timeout_does_not_retry(self):
        self.usb.write = lambda wire: self.usb.requests.append(wire)
        with self.assertRaises(TimeoutError):
            self.session.verify()
        self.assertEqual(len(self.usb.requests), 1)

    def test_stale_response_ignored(self):
        original = self.usb.write
        def write(data):
            request = Framer().feed(data)[0]
            self.usb.pending.extend(Packet(request.seq-1, 0, 0, b'wrong').encode())
            original(data)
        self.usb.write = write
        self.session.verify()
        self.assertTrue(self.session.verified)

    def test_invalid_plan_preflight(self):
        for step in [{'name': '../unsafe', 'key': 'UP'}, {'name': 'call', 'key': 'DIAL'}, {'name': 'up', 'key': 'UP'}]:
            with self.assertRaises(ValueError):
                run_plan(self.session, {'schema': 1, 'steps': [step]})
        self.assertEqual(self.usb.requests, [])
        before = {'roi': [0, 0, 240, 320], 'sha256': '0'*64}
        overlap = {'schema': 1, 'steps': [
            {'name': 'up-before', 'key': 'UP', 'before': before},
            {'name': 'up', 'key': 'UP', 'before': before}]}
        with self.assertRaises(ValueError):
            validate_plan(overlap)


if __name__ == '__main__':
    unittest.main()
