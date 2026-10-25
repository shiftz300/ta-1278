#!/usr/bin/env python3
"""TA-1278 runtime diagnostic capture. Run --help; monitor sends no commands."""
import argparse
import json
from pathlib import Path
import sys
import time
from handset.usb import USB


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    monitor = sub.add_parser('monitor', help='Passive descriptor watch, no device handles')
    monitor.add_argument('--seconds', type=int, default=900)
    monitor.add_argument('--out', type=Path, required=True)
    for command in ['capture', 'run', 'step']:
        p = sub.add_parser(command, help='Explicit runtime USB profile required')
        p.add_argument('--profile', type=Path, required=True)
        p.add_argument('--out', type=Path, required=True)
        p.add_argument('--wait-seconds', type=int, default=0)
        if command == 'run':
            p.add_argument('--plan', type=Path, required=True)
        if command == 'step':
            p.add_argument('--reference', type=Path, required=True)
            p.add_argument('--roi', type=int, nargs=4, default=[0, 32, 240, 280])
            p.add_argument('--key', required=True)
            p.add_argument('--name', required=True)
            p.add_argument('--settle-ms', type=int, default=600)
    region = sub.add_parser('hash-region', help='Make a pre/postcondition from an acquired screenshot')
    region.add_argument('image', type=Path)
    region.add_argument('--roi', type=int, nargs=4, default=[0, 24, 240, 296])
    args = parser.parse_args()
    if args.command == 'hash-region':
        from PIL import Image
        from handset.session import region_hash
        with Image.open(args.image) as im:
            if im.size != (240, 320):
                raise ValueError('Expected original-size handset screenshot')
            print(json.dumps({'roi': args.roi, 'sha256': region_hash(im, args.roi)}))
        return 0
    if args.command == 'monitor':
        if not 1 <= args.seconds <= 3600:
            raise ValueError('Monitor duration must be 1..3600 seconds')
        args.out.mkdir(parents=True, exist_ok=False)
        deadline, previous = time.monotonic()+args.seconds, None
        usb = USB()
        try:
            with (args.out/'usb.jsonl').open('x') as log:
                print('Passive watch ready; no device handles or commands. STOP file ends the watch.', flush=True)
                while time.monotonic() < deadline and not (args.out/'STOP').exists():
                    inventory = usb.inventory()
                    if inventory != previous:
                        record = {'time_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), 'devices': inventory}
                        log.write(json.dumps(record)+'\n'); log.flush()
                        (args.out/'latest.json').write_text(json.dumps(record, indent=2)+'\n')
                        print(json.dumps(record), flush=True)
                        previous = inventory
                    time.sleep(0.5)
                print('Passive watch ended.', flush=True)
        finally:
            usb.close()
        return 0
    from handset.session import Session, run_plan, validate_plan
    profile = json.loads(args.profile.read_text())
    if profile.get('runtime_confirmed') is not True or profile.get('expected_firmware') != '17.00.17.02':
        raise ValueError('Profile requires observed normal runtime and expected firmware')
    if not 0 <= args.wait_seconds <= 3600:
        raise ValueError('Wait must be 0..3600 seconds')
    plan = json.loads(args.plan.read_text()) if args.command == 'run' else None
    if args.command == 'step':
        from PIL import Image
        from handset.session import region_hash
        with Image.open(args.reference) as reference:
            if reference.size != (240, 320):
                raise ValueError('Expected an observed 240x320 handset screen')
            plan = {'schema': 1, 'steps': [{'name': args.name, 'key': args.key,
                    'settle_ms': args.settle_ms,
                    'before': {'roi': args.roi, 'sha256': region_hash(reference, args.roi)}}]}
    if plan is not None:
        validate_plan(plan)
    session, usb = None, USB()
    try:
        session = Session(usb, args.out)
        if plan is not None:
            (args.out/'plan.json').write_text(json.dumps(plan, indent=2)+'\n')
        deadline = time.monotonic()+args.wait_seconds
        # Wait for descriptors only. Never retry opening or transmitting after
        # a session attempt; USB errors need inspection, not blind replay.
        while not any((r['vid'], r['pid']) == (profile['vid'], profile['pid']) for r in usb.inventory()):
            session.stopped()
            if time.monotonic() >= deadline:
                raise TimeoutError('Expected runtime device did not appear')
            time.sleep(0.5)
        usb.connect(profile)
        if plan is not None:
            run_plan(session, plan)
        else:
            session.verify()
            session.capture('baseline')
            session.log('capture_complete')
        return 0
    except (Exception, KeyboardInterrupt) as error:
        if session:
            session.log('stopped', reason=str(error) or 'KeyboardInterrupt', replay=False)
        raise
    finally:
        usb.close()
        if session:
            session.close()


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (Exception, KeyboardInterrupt) as error:
        print('STOPPED: '+(str(error) or 'KeyboardInterrupt'), file=sys.stderr)
        sys.exit(1)
