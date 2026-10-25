# Replica status

2026-10-25. **Not a complete replica or a completed reverse engineering project.** Earlier completion language was wrong. Recovered data, observed behavior, implemented behavior and visual acceptance are different checks.

The handset evidence is firmware 17.00.17.02, [160 reviewed screenshots](handset-samples/20261025/index.json), [observations](handset-observations.md) and bounded firmware listings. The latest layout pass added sessions 284–312. See [browser and SMS comparisons](browser-layout.md).

## Current implementation

| Area | Implemented | Still missing / not proven |
|---|---|---|
| Home and menu | Configurable side shortcuts; Center opens the grid; 18 entries in observed order; Music initially focused; horizontal wrap; return to launch origin; original icons at their source scale. | Exact clock/status rendering, vertical edge rules and lock-state behavior. Alipay/Ximalaya are excluded. |
| Theme | Six recovered palettes, horizontal preview, clamped ends, cancel/confirm and local persistence. Preview uses counter, contacts, calendar, browser, messages, gallery, settings, camera and game assets. | Exact font rasterization and all affected controls. Initial accent is the recovered startup fallback, not the handset's saved value. |
| SMS editor | Bottom-growing 200-pixel bubble, 26-pixel text rows, recipient focus with inactive-body shading and a numeric mode marker beneath the recipient, counter, five body options, disabled focus, 4×3 smileys, 7×6 symbols, modern Chinese templates, clear, save/reopen, local queue and failed-save retention. | Native conversation list/root options, multiple recipients, attachment/MMS flow, selection details, long-press timing, send/retry/failure UI. Root currently exposes local Drafts/Outbox folders as a host convenience. No SMS backend. |
| Input | 拼 → 笔 → Abc → abc → ABC → 123; two-stage pinyin confirmation; four rows of candidates at the observed 24-pixel spacing; preedit counted as displayed; original fonts with fallback. | Prediction uses public Rime/FrequencyWords data. Original T9 internals are intentionally not restored. Candidate ordering, associations and every placement branch do not match exactly. |
| Browser | Separate start/page states, URL cursor wrap, text editors, readonly/disabled distinction, radio groups, independent checkboxes, select cancel, field labels, scrolling, Back/Forward focus restoration, separate page/browser close. Bookmarks/history/offline pages persist. Grouped settings, two font sizes, inline image toggle and picker cancel. HTML flow now preserves inline labels/fields, headings, lists and fieldsets. | Original engine, precise DOM/CSS layout, JS, POST, WML, images, downloads, complete network/error/auth behavior, APN editing, clear commit and native restore behavior. Image-download preference is stored but the text adapter does not fetch images. Offline copies reset entered fields. |
| Music | Library/songs/player/queue/repeat states, current-track entry, pause, seek/release, repeat, shuffle and corrected player geometry. | Audio decoding, real library import, playlists, metadata, interruptions, service lifecycle and full player comparison. Three synthetic tracks only. |
| Organizer | Month navigation, event forms, local contacts, recipient selection, alarms, cancel/save/delete and local JSON persistence. Local alarms display alerts while the app runs. | Native subpickers, recurrence/reminders, snooze, search/groups, image/ringtone selection and exact form layouts. Time uses UTC+8; alarm repeat recognizes 一次/每天/工作日/周末. No OS background scheduler. |
| Utilities | Calculator, stopwatch/laps, timer, conversion, flashlight state, radio frequency/options and recorder timing. | Converter formulas/maps are replacement policy; only sampled key assignments are native evidence. No LED/FM/microphone hardware. Recording stores elapsed time only; these session states are not persistent. Currency has no rates. |
| Snake | Menu, pause/resume, game type and level controls, playable local game. | Physics and maps are replacement rules, not recovered originals; no high-score persistence. |
| Empty services | Call history, file storage, counter, video/gallery have empty local pages. | Call/file/media integrations and their full workflows. An entry opening is not completion of its app. |
| System settings | Personalization → theme and keyboard → shortcuts work. | System/connectivity/sound/security/time/language settings and speed dial remain incomplete. Unsupported entries say so. |

## Acceptance boundary

Rust tests check our state model. Window smoke tests check that actual winit/wgpu frames can be presented and that these routes render. Neither proves parity with the physical phone. Handset screenshots remain separate from replica screenshots in `docs/images/`.

Next work should finish the native SMS conversation and send-dialog model, music playlists, the remaining settings routes, and targeted side-by-side measurements. Service-dependent flows need explicit test fixtures. Unknown native behavior must remain marked unknown; adding a plausible host implementation does not resolve it.
