# UI review

The whole-application review was completed on 2026-10-27 against the 194 approved handset captures and 282 current CPU frames. Every reference has a reviewed comparison. This records a completed review with stated differences; it does not claim exact handset equivalence.

## Repeat

```sh
sh scripts/frames.sh /tmp/ta1278-frames
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/check-assets.py
sh scripts/package-macos.sh
target/TA1278.app/Contents/MacOS/phone-desktop --data-dir /tmp/ta1278-review-profile --disable sounds,network --smoke-test /tmp/ta1278-window-frames
```

Use a new profile folder for the window run. CPU rendering uses synthetic data and performs no audio, USB, network or personal-file access. The window run uses the same 126 presentation scenarios and a separate temporary profile. Frame PNGs come from the CPU buffer; successful GPU presentation is checked separately. This is not a GPU readback comparison.

`frames.rs` holds presentation scenarios shared by the desktop and CPU checks. `views/samples.rs` adds matching focus, text, value and return states. Local browser HTML has the visible text/values needed for comparison; it is not a captured handset HTTP payload.

## Corrections

- Shared Contacts, Backspace, Close, battery, pencil and radio artwork use original PNGs. PNG alpha is applied once. Battery shows a fixed full-battery placeholder. The radio center is round and takes the theme color.
- Composer and both conversation directions share one bubble-tail painter. The triangle opens into the body. Recipient focus uses Contacts; body focus uses Menu. Empty pinyin composition has the observed Back icon.
- Candidates are centered within their columns. Position and unfocused-row color were corrected. Multi-tap exposes the pending character to the shared editor. Symbols, smileys, counts, wrap and popup layers were reviewed.
- Shortcut lists open at the current assignment and retain their viewport while keys move. Cancel leaves the assignment unchanged.
- Contact birthday labels, values, digit highlight and focused underline were adjusted. January 1 shows the observed 元旦 caption. Other holiday tables remain unavailable.
- Timer expiry keeps the underlying page. Stopping the alert preserves the previous menu/focus and shows 计时已停止. A fixture formerly compared alert-stop with manual menu-stop; those are now separate states.
- Browser text-only lines have more leading while form rows keep their measured spacing. Empty HTML stays blank. Radio focus fills the circle rather than a square. Editors, menus, page details, selectors, history and return paths were reviewed.
- Media empty lists draw one caption. The old second storage-volume fixture was removed. The application maps one selected host folder.
- Earlier fixes retained: dialer layout, SMS/Insert popups, calendar/file wording, shortcut row heights, checkbox backgrounds, stopwatch/counter placement and Snake end/level persistence.

## Review results

| Area | Result | Remaining differences or evidence limits |
|---|---|---|
| Messages/input | Reviewed editor, counters, wrap/scroll, modes, candidates, menus, grids and local conversation flows | Outline conversion and font rasterization differ. Inbox/thread states have synthetic coverage but no matched approved handset dataset. |
| Browser | Reviewed start, settings, menus, editors, forms, radio/check/select controls, clear/restore and navigation | Some wrapping, field sizes, URL/title presentation and transient footer timing differ. Test HTML is local; CSS, JavaScript and full web compatibility are outside the supported browser. |
| Contacts/telephone | Reviewed search, forms, birthday, choices, details, menus, delete, local dialer and failed-call history | Some font/baseline differences. Search, groups, picture selection and calls include local policies. No approved connected-call/history reference; no call backend. |
| Home/settings/menu | Reviewed all current pages and all six theme previews | Standalone handset home/menu images are absent from the approved set. Settings-root gear behavior is unknown and omitted. Hardware-only settings stay disabled. |
| Files/media/music | Reviewed storage, folders, operations, dialogs, pickers, gallery, video, music and playlist pages | Host capacity/content vary. Playback, gallery and video lack matching handset captures. No audio output was used in this UI review. |
| Organizer/tools | Reviewed calendar, event/alarm forms, repeats, snooze, timer, stopwatch, converter, counter, torch, radio and recorder | Full holiday tables are unavailable. Some text and rounded edges differ. Radio, recorder and torch have local UI behavior. |
| Snake | Reviewed menu, pause/continue, type, sound, level/cancel/save, scores, play and game over | Positions and scores depend on local state. Sprite/font edges can differ from the lossy handset captures. |

Offline status, host capacity, synthetic messages/contacts, local scores and disabled hardware are explicit host behavior. Removed optional services stay absent. These differences are not hidden by copying handset data into the application.

The full 194-row ledger, reference hashes, comparison sheets and frame hashes are in the external research archive: `ta-1278-source/validation/system-20261027-final/`. Research screenshots and firmware are not application build inputs.

## Validation

- Original UI review: 216 passed, 4 opt-in tests ignored. The later acceptance run has 217 passed and 5 opt-in tests ignored; its full-use test also passed separately with real audio and localhost HTTP. See [acceptance.md](acceptance.md).
- CPU views: 21 tests; 282 current state frames.
- Native macOS window: Metal, 127 frames presented; 126 named scenes captured. Sounds and network disabled; temporary profile only.
- Workspace Clippy with warnings denied, resource SHA-256, formatting and diff checks passed.
- Release application rebuilt; ad-hoc signature verified.

This completes the available-evidence UI review. The table above is the remaining fidelity boundary, not an assertion that unobserved handset states have been verified.
