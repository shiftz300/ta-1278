# Desktop phone

`winit` owns the window and input. `wgpu` presents a 240×320 RGBA texture with nearest sampling and letterboxing. CPU drawing uses original fonts/icons and fallback fonts. This is a new interaction program, not execution of the ARM firmware.

## Modules

| Path | Responsibility |
|---|---|
| `phone-core/input.rs` | Key sources, holds, repeat and cancellation |
| `editor.rs`, `predict.rs` | UTF-16 edits, multi-tap, pinyin/stroke replacements |
| `sms.rs`, `sms/conversation.rs`, `sms_retry.rs` | Recovered count/error rules, mailbox projection and local queue transactions |
| `browser.rs` | Pages, request generations, history/forward, focus/scroll and forms |
| `music.rs` | Playback controls, seek and repeat; external audio clock in normal runs |
| `utilities.rs`, `calendar.rs`, `snake.rs` | Local utility models, date arithmetic and replacement game rules |
| `phone-desktop/main.rs`, `host_input.rs`, `smoke.rs` | Window lifecycle, events, clocks, adapter scheduling and smoke run |
| `phone.rs`, `phone/overlays.rs` | Top-level routes and shared editor/menu overlays |
| `phone/sms.rs`, `conversation.rs`, `mail.rs`, `browser.rs`, `browser_data.rs`, `music.rs` | App routes, local mailbox reading and browser persistence |
| `phone/settings.rs`, `organizer.rs`, `contacts.rs`, `tools.rs`, `snake.rs` | Separate settings, organizer, utility and game routes |
| `ui/` | Shared title, status, softkeys, list/popup, edit/form controls, original control art and typography profiles |
| `views/` | App content and control declarations; no network or storage operations |
| `canvas.rs`, `candidate_view.rs`, `theme.rs`, `menu.rs` | Fonts, pixels, prediction panels, recovered colors and observed menu order |
| `gpu.rs`, `display.wgsl` | Surface and texture presentation |
| `web.rs`, `web/flow.rs`, `phone-core/web_flow.rs` | Bounded HTTP worker, DOM control mapping and ordered HTML layout tokens |
| `views/browser_layout.rs`, `browser_chrome.rs` | Browser document paint/focus geometry and measured native panel styles |
| `phone-vfs` | Virtual mount table, stream/volume interface and directory adapter |
| `media.rs`, `audio.rs`, `phone/files.rs`, `views/files.rs` | Shared file/library service, audio output and separate file-manager UI |
| `config.rs` | Host capability flags, separate from native settings |
| `storage.rs` | Bounded local JSON loading, sync and atomic replacement |

Core code has no USB, modem, network or filesystem access. The browser worker receives only a URL and cannot access the mailbox. No firmware binary is executed.

## Controls

| Input | Action |
|---|---|
| Arrows | D-pad; text cursor; browser focus/scroll; player track/volume |
| Enter / middle mouse | Center |
| F1 / left mouse | Left softkey |
| F2 / Escape / right mouse | Right softkey |
| F3 | Disabled call key |
| Home | End/home; asks about a nonempty draft |
| Digits | Current input mode; hold inserts a digit |
| F4 / hash in an editor | 拼 → 笔 → Abc → abc → ABC → 123 |
| Backspace / Delete | Delete text; delete selected local list entries where supported |
| Tab | SMS recipient/body |
| Shift+Enter | Newline |
| Cmd/Ctrl+A | Select all text |
| Window close / Cmd+Q | Save active draft; stay open if dirty data cannot be saved |

Mouse buttons apply anywhere in the client area, including first activation clicks. Focus loss cancels holds without a music skip. IME and direct letters work; clipboard paste is not implemented. Editor up/down uses rendered glyph widths. Candidate confirmation takes precedence over sending or closing a field. Displayed preedit participates in SMS counts but is not saved until committed. Stroke uses keys 1–5 for 一丨丿丶乛.

## App boundaries

SMS supports one recipient, peer conversations with an inline composer, drafts, received records and a local unsent queue. Received-message input is an explicit local service method; normal startup generates no incoming traffic. Contacts are created locally; no device contacts are imported. Body Save closes only after successful storage. A failed save keeps editing active, including after a later retry. Calling is disabled; SMS stays in the local unsent queue.

Browser opens its four-entry start page. `about:demo` is an offline test document. Text pages, links, fields, radios, checkboxes, single/multiple selects and GET forms work. Single-select reopens focused on row zero, separately from its selected marker. A text editor's right softkey deletes until empty, then cancels. Back/Forward restore in-session focus and scroll. Closing a page and exiting the app are separate confirmations. History URLs survive either action and app restart.

HTTP has a 12-second timeout, 2 MiB decoded-body limit and two waiting jobs. Stop invalidates results; it does not abort the worker's underlying request immediately. Back holds 32 pages; persistent history holds 100 URLs, offline storage 20 parsed documents. Offline snapshots restore initial fields and clear passwords. No JS/CSS renderer, POST, WML, images or downloads. Native settings now use one home type, two font sizes and an inline image checkbox. Clear entry/cancel is sampled; activation is unavailable because native commit behavior is unknown. Restore confirmation remains host policy. Saved pages without flow data still load. See [layout evidence](browser-layout.md).

Music decodes MP3/WAV from the VFS. Playback time comes from the audio sink and continues outside the player; held seek is canceled on navigation. Three synthetic tracks are retained only for screenshot smoke scenes. See [storage](storage.md). Utility timer/stopwatch state continues offscreen while the program runs. Radio/flashlight/recording only change local state. Recorder entries contain durations, not sound. Organizer alarms show local dialogs and snooze while running; no OS wake scheduler is implemented. Dates use UTC+8. Snake is a playable replacement; its maps/physics are not native evidence.

## Data

macOS data directory: `~/Library/Application Support/TA1278 Replica/`. Override with `--data-dir PATH`. Run one instance per directory.

- `messages.json`: drafts, received records/read state and outgoing queue; optional timestamps and explicit synthetic sent state.
- `preferences.json`: accent and side shortcuts.
- `browser.json`: bookmarks, history, offline documents and browser preferences.
- `organizer.json`: local contacts, calendar entries and alarms.
- `music.json`: volume, repeat, shuffle, virtual track path and paused resume position.
- `files/`, `card/`: default VFS volumes, separate from private app records.

Writes use a temporary file, sync and rename. Corrupt/oversized existing files are retained and startup fails rather than overwriting them. All dirty stores block normal window close on save failure. App data is separate from firmware, acquisition archives and public screenshots.

The prototype's 750 ms multi-tap, 500 ms long press, 120 ms repeat, 4096-unit SMS limit, extra local folder routes, utility formulas, game rules and host persistence are implementation choices. See [fidelity status](fidelity.md) and [asset licenses](../assets/README.md).

[Shared UI controls](shared-ui.md) describe the native framework correlation and the common host implementation. Pages supply actions and control data; they do not own softkey baselines or edit metrics.
