# Actual handset screenshots

112 reviewed frames from TA-1278 firmware 17.00.17.02, captured on 2026-10-25 through the original OS diagnostic interface. These are **not desktop replica renders**.

- [All thumbnails](20261025/contact-sheet.png)
- [Names, source sessions and hashes](20261025/index.json)
- [Observed behavior and limits](../handset-observations.md)
- [Numeric SMS regression](../handset-plans/README.md)

Useful comparisons:

| Area | Frames |
|---|---|
| SMS | [Pinyin](20261025/sms-pinyin-64.png), [candidate focus](20261025/sms-candidate-focus.png), [symbols](20261025/sms-symbol-picker.png), [160 digits](20261025/sms-numeric-160.png), [161 digits](20261025/sms-numeric-161.png) |
| Browser | [Start](20261025/browser-start.png), [URL](20261025/browser-public-url.png), [page options](20261025/browser-page-options.png), [close page](20261025/browser-page-close.png) |
| Settings | [Root](20261025/system-settings.png), [theme](20261025/theme.png), [preview cancel](20261025/theme-cancel.png) |
| Other tools | [Stopwatch](20261025/stopwatch-lap.png), [temperature](20261025/temperature-result.png), [calculator](20261025/calculator-chain.png) |
| Editors | [New event](20261025/calendar-new-event.png), [new alarm](20261025/alarm-new.png), [new contact](20261025/contact-new.png) |

JPEG is lossy. Some headers were reconstructed from verified native quantization tables; raw responses and repair metadata are retained outside Git. PNG is a decoded capture, not raw framebuffer evidence. The test phone has modified test settings. Frames with original contacts/alarms, inbox previews, personal wallpaper, call history or media names are excluded. The displayed a0000/1230000 contact is a verified test-mode-generated fixture.


## 2026-10-27

[14 further native frames](20261027/index.json) cover Files and Timer. Each PNG is verified against the original session capture hash. Two originally misleading action names are corrected in the index: the supposed move-picker capture was actually delete selection (cancelled without deleting), and the timer stop attempt captured expiration before a guard stopped the key. These frames predate the owner’s request for silence; no further key or audible test is run during quiet time.
