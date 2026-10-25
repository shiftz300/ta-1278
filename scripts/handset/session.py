"""Verified session, capture and guarded actions. No reconnect/replay after a write."""
import hashlib
import io
import json
import secrets
import time
from pathlib import Path
from PIL import Image
from .protocol import Framer, Screenshot, version_request, screenshot_request, key_request


class Session:
    def __init__(self, transport, out, timeout=8):
        self.transport, self.out, self.timeout = transport, Path(out), timeout
        self.framer = Framer()
        self.seq = secrets.randbelow(0xffff0000)
        self.verified = False
        self.baseline = False
        self.out.mkdir(parents=True, exist_ok=False)
        self.logfile = (self.out/'events.jsonl').open('x')

    def log(self, event, **values):
        row = {'time_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), 'event': event, **values}
        self.logfile.write(json.dumps(row, ensure_ascii=False)+'\n')
        self.logfile.flush()
        print(json.dumps(row, ensure_ascii=False), flush=True)

    def stopped(self):
        if (self.out/'STOP').exists():
            raise InterruptedError('STOP file requested')

    def next_seq(self):
        self.seq += 1
        return self.seq

    def exchange(self, packet, kind, ignore_stop=False):
        if not ignore_stop:
            self.stopped()
        self.transport.write(packet.encode())
        self.log('request', seq=packet.seq, type=packet.kind, subtype=packet.subtype)
        deadline = time.monotonic()+self.timeout
        while time.monotonic() < deadline:
            if not ignore_stop:
                self.stopped()
            data = self.transport.read(250)
            for response in self.framer.feed(data):
                if response.seq == packet.seq and response.kind == kind:
                    yield response
        raise TimeoutError(f'No complete response for type {packet.kind}; no automatic retry')

    def verify(self):
        responses = self.exchange(version_request(self.next_seq()), 0)
        try:
            response = next(responses)
            payload = response.payload
            expected = '17.00.17.02'
            if expected.encode() not in payload and expected.encode('utf-16le') not in payload:
                raise ValueError('Firmware response did not match 17.00.17.02; raw response not saved')
            self.verified = True
            self.log('version_verified', version=expected)
        finally:
            responses.close()

    def capture(self, name):
        if not self.verified:
            raise ValueError('Version handshake required')
        if not name or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' for c in name):
            raise ValueError('Unsafe screenshot name')
        if (self.out/(name+'.jpg')).exists():
            raise ValueError('Screenshot name already exists')
        packet = screenshot_request(self.next_seq())
        image = Screenshot(packet.seq)
        responses = self.exchange(packet, 0xa0)
        try:
            for response in responses:
                try:
                    jpeg = image.add(response)
                except ValueError:
                    if image.raw is not None:
                        (self.out/(name+'.rejected-image.bin')).write_bytes(image.raw)
                        self.log('image_rejected', name=name, bytes=len(image.raw),
                                 raw_sha256=hashlib.sha256(image.raw).hexdigest())
                    raise
                if jpeg is None:
                    continue
                with Image.open(io.BytesIO(jpeg)) as img:
                    if img.format != 'JPEG' or img.size != (240, 320):
                        raise ValueError('Expected a 240x320 JPEG')
                    img.load()
                    decoded = img.convert('RGB')
                (self.out/(name+'.jpg')).write_bytes(jpeg)
                if image.header_repair:
                    (self.out/(name+'.raw-image.bin')).write_bytes(image.raw)
                decoded.save(self.out/(name+'.png'))
                self.baseline = True
                self.log('capture', name=name, sha256=hashlib.sha256(jpeg).hexdigest(),
                         raw_sha256=hashlib.sha256(image.raw).hexdigest(),
                         header_repair=image.header_repair,
                         png_sha256=hashlib.sha256((self.out/(name+'.png')).read_bytes()).hexdigest(),
                         pixel_sha256=hashlib.sha256(decoded.tobytes()).hexdigest())
                return decoded
        finally:
            responses.close()
        raise TimeoutError('Image incomplete')

    def _ack_key(self, name, event, ignore_stop=False):
        if not self.verified or not self.baseline:
            raise ValueError('Version and a valid screenshot required before any key')
        packet = key_request(self.next_seq(), name, event)
        responses = self.exchange(packet, 1, ignore_stop)
        try:
            response = next(responses)
            if response.subtype != packet.subtype or response.payload:
                raise ValueError('Unexpected key ACK')
        finally:
            responses.close()
        self.log('key_ack', key=name, action=event)

    def press(self, name, hold_ms=0):
        if hold_ms == 0:
            self._ack_key(name, 'tap')
            return
        if not 20 <= hold_ms <= 1000:
            raise ValueError('Hold must be 20..1000 ms')
        try:
            self._ack_key(name, 'down')
            self.pause(hold_ms/1000)
        finally:
            # Best effort release even if ACK, STOP or host interruption failed.
            # Never reconnect to a new device to deliver a delayed release.
            try:
                self._ack_key(name, 'up', ignore_stop=True)
            except Exception as error:
                self.log('release_failed', reason=str(error))
                raise

    def pause(self, seconds):
        deadline = time.monotonic()+seconds
        while time.monotonic() < deadline:
            self.stopped()
            time.sleep(min(0.05, max(0, deadline-time.monotonic())))

    def close(self):
        self.logfile.close()


def region_hash(image, roi):
    if (not isinstance(roi, list) or len(roi) != 4 or
            any(type(x) is not int for x in roi) or
            not 0 <= roi[0] < roi[2] <= 240 or not 0 <= roi[1] < roi[3] <= 320):
        raise ValueError('Invalid 240x320 region')
    return hashlib.sha256(image.convert('RGB').crop(roi).tobytes()).hexdigest()


def validate_plan(plan):
    from .protocol import KEYS
    if plan.get('schema') != 1 or not isinstance(plan.get('steps'), list) or not 1 <= len(plan['steps']) <= 200:
        raise ValueError('Expected schema 1 and 1..200 steps')
    names = {'baseline'}
    for step in plan['steps']:
        name = step.get('name', '')
        if not name or not name.isascii() or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' for c in name) or name in names:
            raise ValueError('Unique simple step names required')
        if name+'-before' in names:
            raise ValueError('Screenshot filenames overlap')
        names.update((name, name+'-before'))
        if step.get('key') not in KEYS:
            raise ValueError('Unknown or blocked key')
        expected = step.get('before', {})
        digest = expected.get('sha256', '')
        if len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
            raise ValueError('Every key needs an observed pre-action image hash')
        region_hash(Image.new('RGB', (240, 320)), expected.get('roi'))
        if type(step.get('hold_ms', 0)) is not int or step.get('hold_ms', 0) not in [0, *range(20, 1001)]:
            raise ValueError('Invalid hold time')
        if type(step.get('settle_ms', 400)) is not int or not 100 <= step.get('settle_ms', 400) <= 5000:
            raise ValueError('Invalid settle time')
        if 'after' in step:
            region_hash(Image.new('RGB', (240, 320)), step['after'].get('roi'))
            digest = step['after'].get('sha256', '')
            if len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
                raise ValueError('Invalid post-action hash')


def run_plan(session, plan):
    validate_plan(plan)  # Entire plan checked before first transmission.
    session.verify()
    image = session.capture('baseline')
    for step in plan['steps']:
        # Refresh immediately before acting. Unexpected dialogs/clock changes
        # in the chosen region stop the run instead of guessing the next screen.
        image = session.capture(step['name']+'-before')
        expected = step['before']
        if region_hash(image, expected['roi']) != expected['sha256']:
            raise ValueError(f"Pre-action state mismatch: {step['name']}; key not sent")
        session.press(step['key'], step.get('hold_ms', 0))
        session.pause(step.get('settle_ms', 400)/1000)
        image = session.capture(step['name'])
        if 'after' in step:
            expected = step['after']
            if region_hash(image, expected['roi']) != expected['sha256']:
                raise ValueError(f"Post-action state mismatch: {step['name']}")
            session.log('assertion_passed', step=step['name'])
    session.log('plan_complete', steps=len(plan['steps']), assertions=sum('after' in s for s in plan['steps']))
