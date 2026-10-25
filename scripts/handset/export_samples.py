"""Export an explicit allowlist of reviewed, non-private handset screens.

Raw responses and all unlisted screens stay in the external local archive.
Run from the repository root: PYTHONPATH=scripts python3 -m handset.export_samples
"""
import argparse
import hashlib
import io
import json
import math
from pathlib import Path
from PIL import Image, ImageDraw
from .protocol import Packet, Screenshot

SAMPLES = [
    (4, 'sms-new'), (6, 'baseline', 'sms-body-focus'),
    (7, 'sms-pinyin-6'), (8, 'sms-pinyin-64'), (9, 'sms-candidate-focus'),
    (10, 'sms-committed-ni'), (11, 'sms-options'), (12, 'sms-insert-menu'),
    (13, 'sms-smiley-picker'), (14, 'sms-after-smiley-cancel'),
    (15, 'sms-input-method'), (17, 'sms-input-abc'), (18, 'sms-symbol-picker'),
    (19, 'symbol-nav-11', 'sms-newline-selected'), (20, 'sms-newline'),
    (21, 'sms-delete-newline'), (22, 'sms-clear-test-character'),
    (25, 'browser-start'), (26, 'browser-url-editor'),
    (27, 'url-delete-07', 'browser-empty-url'), (28, 'browser-root-options'),
    (32, 'browser-exit'), (34, 'music-start', 'music-refresh'),
    (35, 'baseline', 'music-root-empty-selection'), (36, 'music-options'),
    (47, 'system-settings'), (49, 'personalization'), (51, 'theme'),
    (52, 'theme-right-1', 'theme-blue'),
    (53, 'theme-left-1', 'theme-grey-green'),
    (53, 'theme-left-2', 'theme-green'),
    (53, 'theme-left-3', 'theme-orange'),
    (53, 'theme-left-4', 'theme-red-orange'),
    (53, 'theme-left-5', 'theme-purple'),
    (54, 'theme-cancel'), (56, 'keyboard-settings'),
    (64, 'input-hash-3', 'sms-input-lowercase'),
    (64, 'input-hash-4', 'sms-input-uppercase'),
    (64, 'input-hash-5', 'sms-input-numeric'),
    (65, 'multitap-3', 'sms-multitap-pending'),
    (65, 'multitap-4', 'sms-multitap-committed'),
    (67, 'digits-160', 'sms-numeric-160'),
    (67, 'digits-161', 'sms-numeric-161'),
    (70, 'sms-options-down-2', 'sms-disabled-row-focus'),
    (72, 'sms-clear-result'), (73, 'sms-options', 'sms-empty-options'),
    (77, 'sms-template-inserted'),
    (78, 'draft-select-5', 'sms-save-selected'),
    (87, 'sms-new-after-save'),
    (90, 'ucs-length-070', 'sms-unicode-70'),
    (90, 'ucs-length-071', 'sms-unicode-71'),
    (91, 'cursor-up-6', 'sms-recipient-focus'),
    (91, 'cursor-body-return', 'sms-body-focus-restored'),
    (101, 'url-type-20', 'browser-public-url'),
    (103, 'baseline', 'browser-blank-page'),
    (105, 'browser-page-options'),
    (106, 'page-options-down-6', 'browser-page-settings-entry'),
    (107, 'browser-open-menu'), (109, 'navigation-menu', 'browser-navigation-menu'),
    (111, 'browser-page-details'), (113, 'browser-page-close'),
    (114, 'browser-page-closed'),
    (117, 'other-root'), (118, 'timer-root'),
    (119, 'stopwatch-root'), (120, 'stopwatch-running'),
    (121, 'stopwatch-lap'), (122, 'stopwatch-stopped'), (123, 'stopwatch-reset'),
    (124, 'unit-open-4', 'unit-converter'), (125, 'temperature'),
    (126, 'temperature-fields'), (127, 'temperature-2', 'temperature-result'),
    (127, 'temperature-3', 'temperature-reverse-focus'),
    (130, 'calculator-root'), (131, 'calculator-4', 'calculator-add'),
    (131, 'calculator-7', 'calculator-chain'), (132, 'calculator-delete'),
    (135, 'baseline', 'counter-empty'),
    (137, 'calendar-root'), (138, 'calendar-options'), (139, 'calendar-new-event'),
    (140, 'event-field-4', 'calendar-event-ringtone'),
    (140, 'event-field-5', 'calendar-event-repeat'),
    (140, 'event-field-6', 'calendar-event-note'),
    (144, 'alarm-new'), (145, 'alarm-field-4', 'alarm-snooze'),
    (148, 'snake-start'), (149, 'snake-game'), (150, 'snake-pause'),
    (151, 'snake-level-3', 'snake-level'), (152, 'snake-type-3', 'snake-types'),
    (156, 'baseline', 'radio-root'), (157, 'radio-options'),
    (158, 'radio-option-6', 'radio-close-option'),
    (161, 'files-root'), (162, 'files-options'), (163, 'files-details'),
    (165, 'recorder-root'), (168, 'contacts-options'),
    (172, 'test-contact-details'), (173, 'test-contact-options'),
    (175, 'test-contact-delete-dialog'),
    (178, 'contact-form-3', 'contact-new'),
    (179, 'contact-field-04', 'contact-phone-fields'),
    (179, 'contact-field-05', 'contact-birthday'),
    (179, 'contact-field-06', 'contact-ringtone'),
    (179, 'contact-field-07', 'contact-group'),
    (183, 'dial-zero-10', 'dial-synthetic-number'),
    (198, 'torch-off-3', 'torch-on'), (199, 'torch-disabled', 'torch-off'),
    (200, 'finish-return-1', 'other-restored'),
    (209, 'baseline', 'browser-live-text'),
    (215, 'form-load', 'browser-live-form'),
    (218, 'form-text-committed', 'browser-field-committed'),
    (221, 'telephone-editor', 'browser-tel-editor'),
    (225, 'radio-medium-focus', 'browser-radio-focus'),
    (226, 'radio-medium-selected', 'browser-radio-selected'),
    (230, 'checkbox-two-checked', 'browser-checkboxes'),
    (233, 'form-next-0', 'browser-time-focus'),
    (235, 'textarea-second-line', 'browser-textarea-newline'),
    (236, 'textarea-committed', 'browser-textarea-preview'),
    (237, 'form-close-dialog', 'browser-form-back'),
    (241, 'cern-loaded', 'browser-loading-prior-body'),
    (242, 'baseline', 'browser-links'),
    (243, 'cern-link-open', 'browser-link-target'),
    (244, 'cern-link-back', 'browser-link-back'),
    (247, 'navigation-next', 'browser-disabled-stop-focus'),
    (248, 'forward-focus', 'browser-forward-focus'),
    (249, 'forward-page', 'browser-forward-result'),
    (254, 'baseline', 'browser-https-form'),
    (257, 'select-open', 'browser-select-open'),
    (258, 'select-two-2', 'browser-select-committed'),
    (259, 'select-reopen', 'browser-select-reopened'),
    (260, 'select-one-focus', 'browser-select-uncommitted-focus'),
    (261, 'select-cancel-keeps-two', 'browser-select-cancelled'),
    (278, 'shortcut-settings', 'lock-screen-shortcuts'),
    (280, 'shortcut-choices', 'shortcut-current-choice'),
    (281, 'shortcut-choice-05', 'shortcut-options-first'),
    (281, 'shortcut-choice-14', 'shortcut-options-media'),
    (281, 'shortcut-choice-17', 'shortcut-options-browser'),
    (282, 'shortcut-cancelled', 'shortcut-cancelled'),
    (287, 'browser-open', 'browser-start-resampled'),
    (288, 'root-options', 'browser-root-overlay'),
    (290, 'browser-settings'),
    (291, 'settings-down-2', 'browser-home-type-focus'),
    (292, 'home-type-picker', 'browser-home-type-picker'),
    (293, 'home-type-cancel', 'browser-home-type-cancel'),
    (294, 'settings-down-2', 'browser-images-off'),
    (295, 'download-images-picker', 'browser-images-on'),
    (296, 'images-restored-off', 'browser-images-restored-off'),
    (297, 'font-size-focus', 'browser-font-size-focus'),
    (298, 'font-size-picker', 'browser-font-size-picker'),
    (299, 'font-size-cancel', 'browser-font-size-cancel'),
    (302, 'browser-exit-prompt', 'browser-exit-resampled'),
    (303, 'exit-left-softkey', 'browser-exit-left-unchanged'),
    (308, 'settings-down-6', 'browser-clear-focus'),
    (309, 'clear-options', 'browser-clear-picker'),
    (310, 'clear-options-cancel', 'browser-clear-cancel'),
    (311, 'settings-final-row', 'browser-restore-focus'),
]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, default=Path(
        '/Users/shiftz/dev/ta-1278-source/handset-sessions/20261025'))
    parser.add_argument('--out', type=Path, default=root/'docs/handset-samples/20261025')
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    rows, images = [], []
    for entry in SAMPLES:
        session, frame = entry[:2]
        name = entry[2] if len(entry) > 2 else frame
        directory = args.archive/f'ta1278-native-{session:03d}'
        candidates = [directory/(frame+suffix) for suffix in
                      ['.raw-image.bin', '.rejected-image.bin', '.jpg']]
        source = next(path for path in candidates if path.exists())
        raw = source.read_bytes()
        screenshot, jpeg = Screenshot(1), None
        count = math.ceil(len(raw)/32768)
        for index in range(count):
            subtype = (0x80|count) if index == 0 else (0 if index == count-1 else index+1)
            jpeg = screenshot.add(Packet(1, 160, subtype, raw[index*32768:(index+1)*32768]))
        assert jpeg is not None
        with Image.open(io.BytesIO(jpeg)) as decoded:
            assert decoded.size == (240, 320)
            decoded.load()
            image = decoded.convert('RGB')
        png = args.out/(name+'.png')
        image.save(png)
        images.append((name, image))
        rows.append({'name': name, 'source_session': directory.name, 'source_frame': frame,
                     'source_file': str(source.relative_to(args.archive)),
                     'source_sha256': digest(raw), 'header_repair': screenshot.header_repair,
                     'png': png.name, 'png_sha256': digest(png.read_bytes()),
                     'pixel_sha256': digest(image.tobytes()), 'device_firmware': '17.00.17.02'})
    # A contact sheet of actual decoded captures, never replica renders.
    sheet = Image.new('RGB', (4*248, math.ceil(len(images)/4)*352), '#dddddd')
    draw = ImageDraw.Draw(sheet)
    for index, (name, image) in enumerate(images):
        x, y = index%4*248+4, index//4*352+4
        sheet.paste(image, (x, y))
        draw.text((x, y+324), name[:31], fill='black')
    sheet.save(args.out/'contact-sheet.png')
    metadata = {'source_kind': 'actual_handset_diagnostic_JPEG', 'date': '2026-10-25',
                'test_mode_enabled': True, 'complete_sampling': False,
                'privacy': 'Explicit reviewed allowlist. No inbox, personal wallpaper, history, media names, original contacts or original alarms. Contact a0000/1230000 is a verified test-mode generated fixture. New contact/alarm/event forms are unsaved defaults; storage totals and current theme are device state.',
                'limits': 'Lossy JPEG decoded locally. Some headers reconstructed from verified native tables. These are not raw framebuffer captures.',
                'samples': rows, 'contact_sheet_sha256': digest((args.out/'contact-sheet.png').read_bytes())}
    (args.out/'index.json').write_text(json.dumps(metadata, indent=2)+'\n')
    print(f'Exported {len(rows)} reviewed native screens; raw/private data stays outside repository.')


if __name__ == '__main__':
    main()
