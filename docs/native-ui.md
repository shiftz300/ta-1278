# Native UI recovery

2026-10-25. Firmware **17.00.17.02**, `user.bin` SHA-256 `8f0db2dc82f9e763cb700c2e5781b7ca7125a4d250f4c075dbec793eb549c1ca`.

Run `python3 scripts/recover-ui-style.py` from the repository root. It reads the published original, verifies its hash, extracts the palette and generates bounded LLVM Thumb listings. It does not run firmware, access the handset or read saved preferences. Output: `assets/ui/native-theme.json` and `docs/evidence/`.

## New native evidence

| Finding | Native evidence | Meaning and limits |
|---|---|---|
| Six accent palettes, seven shades each | `0x883a463a` accepts accent IDs below 6 and offsets below 7. `0x883a4664` uses the current accent. Literal `user+0x3a49c0` points to `0x886f73b4`; indexing is `(accent*7+shade)*2`. | The 55 named resource colors are not the whole effective theme. The prototype now uses this table directly instead of an invented blue palette. |
| Startup preference fallback is 1 | `0x883a45d8..0x883a460e` initializes the preference to 1 before attempting the saved-setting path. | The user's saved theme was not read. The host starts with this fallback and exposes all six choices in Settings. Horizontal preview/cancel/confirm now follow live observations; preferences persist locally. |
| Chinese input calls the popup drawing routines | `0x881cb856..0x881cb8c0` calls geometry `0x881c9d8a`, the `+0x11bc` list draw `0x88a79d26` for mode byte 1, and the `+0xe04` list draw `0x88a79f5c` when its dirty flag is set. | This is a Chinese input drawing path, distinct from the SMS contact-recipient candidate window. The two lists are interpreted as spelling and text candidates from the Chinese dispatch and adjacent conversion code; not all input modes are reconstructed. |
| Four entries per page | `0x88a79e86` / `0x88a79fa6` mask the selection with `~3`; bounds end at page start +3. `0x88a79fa4` also clamps to count-1. | The previous three-row host panel was unsupported. The replacement presents four entries per column. |
| Native popup shades | `0x88a79e66..0x88a79ec0` and `0x88a7a0ea..0x88a7a146` request accent shade 5 for the fill and shade 2 for the selection. Text uses the halfword at `0x8874cd5c+0x18`, which is `0xffff`. | These values are wired into `candidate_view.rs`. The inactive-column emphasis is a host choice. |
| Row size and caret placement | `0x88a79d34` / `0x88a79f66` use 30; the inset path uses 24. `0x881c9ccc` derives width from maximum item width +31, limits rows to four, and changes above/below positioning when the supplied caret rectangle bottom reaches 184. `0x881c9d8a` places the pair side by side within a 240-wide display. | The host now uses four 24-pixel rows, matching the observed inset path, measured width +31, and the 184 threshold. It clamps to host editor bounds; exact native metrics, popup flags, paired-rectangle overflow adjustment and every focus branch are not emulated. |

The small listings in [evidence](evidence/native-ui-ranges.json) include source-range SHA-256 values. Static call/data evidence establishes these local rules, not complete runtime equivalence.

## Public visual cross-check

- [Cool3c, 2020-11-16 Nokia 215 4G review](https://www.cool3c.com/article/158073): the menu photo shows three columns, wallpaper, a circular focus marker and the selected app name at the top. The device is a Taiwan review unit; it does not establish this handset's enabled apps, order or saved wallpaper.
- [Ultra Value Tech, Nokia 215 4G SMS input demonstration](https://www.youtube.com/watch?v=HxlTSA_LpJ0): inspected at approximately 0:15 and 0:46. The recipient field is at the top, the message input is a bottom bubble, the counter sits below it, softkeys use icons, and the English candidate popup floats near the composition. This is an external English-language variant, not proof of the user's Chinese layout or encoding settings. Its displayed count must not replace the recovered counter configuration.

Only the page/video was viewed. Third-party photos/video were not bundled into the application. The owner reports a pure-black default background and color selection in Settings (2026-10-25). The replica now uses black for page backgrounds. This is owner-provided handset evidence, not an independently captured screenshot or a recovered firmware default. Available assets alone do not establish where or when they are drawn.

On 2026-10-26 the owner explicitly selected the green character wallpaper shown on their own handset as the replica default. The intact `wallpaper.dat` was recovered from the existing private FAT12 backup and decoded as 232×316 RGB565; this is not a screenshot with UI painted out and is not a factory-default claim. Home and Menu share one clipped background renderer. App pages stay black. Position (4,2), clip (4,24,232,258) and the Menu's 3/5 brightness follow capture comparison; the brightness is an approximation. [Asset provenance](../assets/ui/owner-wallpaper.json).

Wallpaper settings now offer Default, None and Choose from files. Older profiles with no selected file adopt the new default; saved custom files retain priority. Explicit None persists through `wallpaper_enabled=false`. `--disable pictures` suppresses both built-in and custom wallpapers. A missing selected custom file stays black instead of silently substituting the default. The raw cache, its trailer and complete private captures remain outside the repository.

- `IMAGE_THEME_BG`, `native-images/00291-icon-00049864.png`: decoded 240×320 gradient. Preserved in the research archive; not used as the replica default.
- `IMAGE_WALLPAPER_1`, `native-images/02851-icon-00b02630.png`: decoded moon wallpaper, matching the type shown in the public menu photo. Preserved in the research archive; not used as the replica default.
- The grid now uses all 18 observed entries and their original icons. The two owner-excluded services are inactive. Original 76×76 icons contain transparent margins, so they are drawn without the earlier extra downscale. Theme preview additionally uses the original camera/game icons. Opening an entry is not full implementation of its service.

## SMS behavior corrected

The [native SMS contract](../nokia-ta1278-research/analysis/ui/apps/sms-contract.md) already established these rules; the first desktop implementation failed to apply them:

- Recipient softkey menu: Contacts / Groups. Contacts now select from a separate local store; groups remain empty.
- Body softkey menu: Insert options, Attachments, Remove attachments, Clear text, Save as draft. Disabled rows receive focus; attachments have no backend.
- Insert options opens smiley / symbol / template pickers. Seven modern Chinese templates use `0x400800c8..ce`; the earlier `0x40080063..67` duplicate block was not the observed modern template set. Smiley/symbol grids now follow the 4×3 and 7×6 observed ordering.
- Clear text is immediate on this path.
- Save as draft exits after storage confirmation. A failed save keeps the editor and body. Subsequent background retry cannot close an editor resumed by the user.
- Center in the body validates recipient/body and queues a local test message. It cannot send an SMS. Confirming an active candidate consumes the key first and cannot accidentally send.

Chinese labels for Insert options, smiley, symbol, template, Clear text and Save as draft are from `0x400800c0..c3`, `0x400800ff`, `0x40080102`. The current composer uses the matching modern Chinese labels `0x40080106..107`, rather than the older untranslated duplicate resources.

## Still host implementations

Replacement Rime/FrequencyWords prediction and ordering; left/right to switch spelling/text columns; persistent selection/context learning; font rasterization and fallback; the exact speech-bubble dimensions and tail; status icons; unsampled list edge cases; local persistence; HTTP text layout and synthetic music. These are explicitly not claimed as recovered native behavior. The original dictionary engine, complete UI framework and service state machines remain unfinished.
