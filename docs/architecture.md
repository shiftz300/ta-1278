# Desktop phone

The public `phone_runtime` library owns phone interaction and CPU drawing. The optional desktop host uses `winit` for the window and input. `wgpu` presents a 240×320 RGBA texture with nearest sampling and letterboxing. CPU drawing uses original fonts/icons and fallback fonts. This is a new interaction program, not execution of the ARM firmware.

## Modules

Rust source files use one word. Parent directories name the owner: `audio/midi.rs`,
`storage/files.rs`, `phone/browser/data.rs`, `views/browser/layout.rs`. Controllers
stay under `phone/`, presentation under `views/`, and reusable controls under `ui/`.
Generated data belongs to its feature (`converter/data.rs`). Original firmware,
resource and capture filenames keep their evidence identifiers.

| Path | Responsibility |
|---|---|
| `phone-core/input.rs` | Key sources, holds, repeat and cancellation |
| `editor.rs`, `predict/` | UTF-16 edits, multi-tap, replaceable prediction and persistent learning |
| `sms.rs`, `sms/conversation.rs`, `sms/retry.rs` | Recovered count/error rules, mailbox projection and local queue transactions |
| `browser.rs` | Pages, request generations, history/forward, focus/scroll and forms |
| `music.rs` | Playback controls, seek and repeat; external audio clock in normal runs |
| `utilities.rs`, `converter.rs`, `calendar.rs`, `snake.rs` | Local utility models, date arithmetic and replacement game rules |
| `lib.rs`, `runtime.rs`, `runtime/state.rs` | Public input/time/frame entry point, service/document ports and host-owned user-data checkpoints |
| `platform.rs` | Explicit native folder/audio/HTTP adapter, bound to one runtime |
| `main.rs`, `desktop.rs`, `host/input.rs`, `desktop/smoke.rs` | Desktop entry point, window lifecycle, native events and window smoke run |
| `storage/profile.rs`, `storage/session.rs` | Existing desktop JSON profile and wall-clock utility persistence |
| `phone.rs`, `phone/editor.rs`, `phone/overlays.rs` | Top-level routes and shared editor/menu overlays |
| `phone/sms.rs`, `conversation.rs`, `mail.rs`, `browser.rs`, `phone/browser/data.rs`, `music.rs` | App routes, local mailbox reading and browser persistence |
| `phone/settings.rs`, `organizer.rs`, `contacts.rs`, `tools.rs`, `snake.rs` | Separate settings, organizer, utility and game routes |
| `phone/calendar.rs`, `phone/files/delete.rs` | Inline calendar editing and file checkbox/confirmation state machines |
| `storage/setup.rs` | First-launch host folder choice and persisted path validation |
| `ui/` | Shared controls, typed menu items, focus policy and popup composition; no app I/O or persistence |
| `views/` | App content and control declarations; no network or storage operations |
| `assets.rs`, `canvas.rs`, `ui/candidates.rs`, `theme.rs`, `menu.rs` | Fonts, pixels, prediction panels, recovered colors and observed menu order |
| `gpu.rs`, `display.wgsl` | Surface and texture presentation |
| `web.rs`, `web/flow.rs`, `phone-core/browser/flow.rs` | Bounded HTTP worker, DOM control mapping and ordered HTML layout tokens |
| `views/browser/layout.rs`, `views/browser/chrome.rs` | Browser document paint/focus geometry and measured native panel styles |
| `storage/files.rs` | One concrete local folder, direct file streams and bounded file operations |
| `media.rs`, `media/files.rs`, `audio.rs`, `phone/files.rs`, `views/files.rs` | Shared file/library service, audio output and separate file-manager UI |
| `config.rs`, `config/apps.rs` | Bounded runtime JSON, optional app policy, host capabilities and storage settings |
| `storage.rs`, `storage/` | Bounded JSON, schema checks, backup/atomic replacement, profile lock and utility checkpoints |

Canvas clipping keeps scrolling form content inside its viewport, including text and images. Calendar uses the shared fields, editor, popup, softkeys and date/time control; file deletion uses the shared checkbox list. Converter coefficients and bounded formatting live in `phone-core/converter.rs`; `phone/converter.rs` owns pair selection and two-field editing, while `views/converter.rs` uses the shared numeric form/editor and softkeys.

Core code has no USB, modem, network or filesystem access. The browser worker receives only a URL and cannot access the mailbox. Only the recovered T9 routines are interpreted in the bounded guest runtime; other firmware code is not executed.

## Controls

| Input | Action |
|---|---|
| Arrows | D-pad; text cursor; browser focus/scroll; player track/volume |
| Enter / middle mouse | Center |
| F1 / left mouse | Left softkey |
| F2 / Escape / right mouse | Right softkey |
| F3 | Call history on Home; dial action in phone pages |
| Home | End/home; asks about a nonempty draft |
| Digits | Current input mode; hold inserts a digit |
| # (Shift+3) in a text editor | 拼 → 笔 → Abc → abc → ABC → 123 |
| # in the numeric converter | Decimal point; Up/Down changes the edited field |
| Backspace / Delete | Delete text; delete selected local list entries where supported |
| Tab | SMS recipient/body |
| Shift+Enter | Newline |
| Cmd/Ctrl+A | Select all text |
| Window close / Cmd+Q | Save active draft; stay open if dirty data cannot be saved |

Mouse buttons apply anywhere in the client area, including first activation clicks. Focus loss cancels holds without a music skip. System IME is disabled for the entire window. Host text comes only from fixed US-English physical key positions (Shift for uppercase/symbols), never OS composition or event text; clipboard paste is not implemented. Internal pinyin, stroke and multi-tap modes remain available. Editor up/down uses rendered glyph widths. Candidate confirmation takes precedence over sending or closing a field. Displayed preedit participates in SMS counts but is not saved until committed. Stroke uses keys 1–5 for 一丨丿丶乛.

## App boundaries

SMS supports one recipient, peer conversations with an inline composer, drafts, received records and a local unsent queue. Received-message input is an explicit local service method; normal startup generates no incoming traffic. Contacts are created locally; no device contacts are imported. Body Save closes only after successful storage. A failed save keeps editing active, including after a later retry. The dialer is enabled but reports unavailable call service; SMS stays in the local unsent queue.

Browser opens its four-entry start page. `about:demo` is an offline test document. Text pages, links, fields, radios, checkboxes, single/multiple selects and GET forms work. Single-select reopens focused on row zero, separately from its selected marker. A text editor's right softkey deletes until empty, then cancels. Back/Forward restore in-session focus and scroll. Closing a page and exiting the app are separate confirmations. History URLs survive either action and app restart.

HTTP has a 12-second timeout, 2 MiB decoded-body limit and two waiting jobs. Stop invalidates results; it does not abort the worker's underlying request immediately. Back holds 32 pages; persistent history holds 100 URLs, offline storage 20 parsed documents. Offline snapshots restore initial fields and clear passwords. No JS/CSS renderer, POST, WML, images or downloads. Native settings now use one home type, two font sizes and an inline image checkbox. Clear entry/cancel is sampled; local checkbox/commit behavior is documented separately because native commit remains unknown. Restore confirmation remains host policy. Saved pages without flow data still load. See layout evidence (research archive).

Music decodes MP3/WAV from the direct folder access. Playback time comes from the audio sink and continues outside the player; held seek is canceled on navigation. Three synthetic tracks are retained only for screenshot smoke scenes. See [storage](storage.md). Timer/stopwatch state persists; running clocks include elapsed time between launches, while paused clocks stay paused. Radio/flashlight/recording only change local state. Recorder entries contain durations, not sound. Organizer alarms show local dialogs and snooze while running; no OS wake scheduler is implemented. Dates use UTC+8. Snake uses retained maps and artwork with local gameplay code; complete original behavior is not claimed.

## Data

macOS data directory: `~/Library/Application Support/TA1278 Replica/`. Override with `--data-dir PATH`. An OS lock enforces one instance per directory for builds with this persistence update.

- `messages.json`: drafts, received records/read state and outgoing queue; optional timestamps and explicit synthetic sent state.
- `preferences.json`: accent and side shortcuts.
- `browser.json`: bookmarks, history, offline documents and browser preferences.
- `organizer.json`: local contacts/groups, playlists, calendar entries, alarms, pending alerts and snoozes.
- `music.json`: volume, repeat, shuffle, folder-relative track path and paused resume position.
- `utilities.json`: timer, stopwatch/laps and radio preferences.
- `snake.json`: unlocked levels, scores and game settings.
- `storage.json`: chosen absolute host folder. Media remains in that folder; there are no default volumes. Old `files/` and `card/` directories are left untouched.

Writes use unique temporary files, sync and rename, keeping one previous JSON copy in `.bak`. See [persistence and recovery](persistence.md). Corrupt/oversized existing files are retained and startup fails rather than overwriting them. All dirty stores block normal window close on save failure. App data is separate from firmware, acquisition archives and public screenshots.

The prototype's 750 ms multi-tap, 500 ms long press, 120 ms repeat, 4096-unit SMS limit, extra local folder routes, general utility precision, game rules and host persistence are implementation choices. Fixed unit coefficients/translation and binary calculator percent now use bounded firmware evidence (research archive). See fidelity status (research archive) and [asset licenses](../assets/README.md).

[Shared UI controls](shared-ui.md) describe the native framework correlation and the common host implementation. Pages supply actions and control data; they do not own softkey baselines or edit metrics.

## Media and local completion

Image, video, contact-picture, wallpaper and ringtone selection share `phone/media.rs` and `views/media.rs`. `media/visual.rs` owns bounded copies and decoding jobs. The small `native/MediaHelper.swift` uses installed macOS frameworks and is bundled beside the executable. `phone/contacts/groups.rs` handles group/choice routes. `effects.rs` routes cues through recovered audio resources and independent SF2 synthesis; native waveform parity is not claimed. Playlist queue rules remain in `phone-core/music.rs`; UI/data stays in `phone/music.rs`. [Behavior and exact limits](local-completion.md).

[Unified runtime config](config.md) filters the main menu and checks both `go`/`launch` routes. Shortcut IDs remain stable; disabled targets are hidden on Home and inactive in selection. Reminder source tags allow disabled app alerts to stay saved without blocking input or audio. These host policies are separate from native evidence. Radio/music page corrections (research archive) reuse shared popup, waiting, softkey and rounded drawing controls.

The [UI authoring guide](shared-ui.md#adding-a-menu) shows how one typed menu declaration drives both rows and keyboard activation. `ui/navigation.rs` keeps list movement separate from app effects; `ui/popup.rs` composes existing list and softkey controls. Native layout differences and asynchronous return/save policies remain in app modules.

[Embedding contract and limits](runtime.md). The desktop and embedded hosts share the runtime; the runtime does not create or terminate a host event loop.
