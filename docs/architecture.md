# Desktop phone

`winit` owns the window and events. `wgpu` presents a 240×320 RGBA texture with nearest sampling and integer scaling where space allows. Resizing adds letterboxing. CPU code draws extracted font outlines and six original menu icons into that texture. This is a new interaction program, not execution of the ARM firmware.

## Modules

| Path | Responsibility |
|---|---|
| `crates/phone-core/src/input.rs` | Shared key holds, release, long press, repeat and focus cancellation |
| `editor.rs` | UTF-16 cursor/selection, limits, multi-tap and deletion |
| `sms.rs` | Recovered character classification/count rules; local draft/outbox data |
| `browser.rs` | Documents, history, request generations and form values |
| `music.rs` | Queue, short/long keys, repeat and simulated position |
| `crates/phone-desktop/src/main.rs` | winit host, IME, clocks and adapter scheduling |
| `host_input.rs` | Desktop key/button mapping |
| `phone.rs` | Screen transitions, overlays and app commands |
| `views.rs`, `canvas.rs` | Layout, text, extracted assets and framebuffer |
| `gpu.rs`, `display.wgsl` | wgpu surface, texture and presentation |
| `web.rs` | Bounded worker, HTTP(S), HTML and offline demo |
| `storage.rs` | Local JSON loading and atomic replacement |

No core module accesses USB, a modem, the filesystem or the network. The browser worker receives only a URL. Messages are stored separately; it cannot access the mailbox or firmware. No firmware code is executed.

## Controls

| Desktop input | Phone action |
|---|---|
| Arrow keys | Direction pad; browser line/page navigation; music volume/track/seek |
| Enter / middle mouse | Center / confirm |
| F1 / left mouse | Left softkey |
| F2 / Escape / right mouse | Right softkey |
| F3 | Call key; service is a placeholder |
| Home | End/home; nonempty message asks save/discard/keep |
| 0–9 / numpad | Multi-tap in text, numbers in numeric mode; hold inserts digit |
| F4 | abc → ABC → 123 |
| Backspace / Delete | Delete previous character/selection; delete highlighted draft/outbox entry |
| Tab | SMS recipient/body |
| Shift+Enter | New line |
| Cmd/Ctrl+A | Select all in the current editor |
| Cmd/Ctrl+Q / window close | Save active draft and exit; stay open on save failure |

Direct letters and desktop IME commits are supported. IME preedit is shown separately from committed text. Clipboard paste is not implemented. Finish IME composition before switching views. Mouse buttons act anywhere in the client area, including macOS's first activation click. Focus loss cancels holds without triggering a music track change. Keyboard and mouse holds for the same phone key share a release state.

## Implemented flows

Messages → Write → recipient → center/Tab → body. Options can queue locally, save, insert a new line, clear, change mode or close. Right softkey clears a character when text is present. Drafts reopen in the editor. Outbox entries show **NOT SENT** and can scroll with up/down. Only one recipient is supported in this first desktop implementation. No inbox reception, SIM, modem or delivery status is simulated as real.

Browser → offline demo. Center activates the selected link/field. Field Cancel keeps its old value. Single-select Confirm commits; Cancel discards. Multi-select Confirm toggles and stays open; Back commits. GET forms include named supported fields and hidden inputs. Reset restores defaults. Right softkey stops loading, otherwise returns through history, otherwise asks to close. Stale responses are ignored. New pages close old field overlays.

HTTP requests have a 12-second global timeout and a 2 MiB decoded-body limit. The worker queue holds at most two waiting requests. Stop discards a result; it does not immediately abort the worker's network operation. History holds at most 32 pages. Only HTTP(S), `about:demo` and the local demo result are supported. No JavaScript, CSS layout, WML, downloads, password manager, radio/date/file inputs or POST forms. Redirects use the final URL as the relative-link base. Native browser encoding and layout are not reproduced.

Music uses three synthetic tracks. Center plays/pauses, short left/right changes track on release, long left/right seeks without changing track on release, up/down changes test volume. Repeat supports off/all/one; shuffle uses a host PRNG, not the unresolved firmware shuffle algorithm. Playback continues as simulated state after returning to the menu. No decoder or audio device is connected.

## Evidence versus policy

See [scope](../nokia-ta1278-research/analysis/ui/replica-readiness.md). The Latin multi-tap table is the recovered table at `user+0x21d592`. Default SMS settings are explicitly `alphabet_management=1`, `sms_encode=0`; the Rust counter follows the recovered classification and native remaining/page asymmetry. Native charsets are not replaced with a generic GSM counter.

Host choices: 750 ms multi-tap, 500 ms long press, 120 ms repeat; menu order, colors, layout, focus and list scrolling; 4096-unit message limit; Unicode scalar-safe deletion rather than creating isolated surrogate halves; direct keyboard/IME input; local JSON persistence; HTTP/HTML adapter and synthetic music. Native timer units, exact rasterizer and full window stack remain unresolved. T9 is not substituted with a guessed algorithm.

## Data

macOS: `~/Library/Application Support/TA1278 Replica/messages.json`. Other platforms default to `$XDG_DATA_HOME/ta1278-replica` or `~/.local/share/ta1278-replica`; these hosts are not validated. `--data-dir PATH` overrides it. Run one app instance per data directory. Storage uses a temporary file, sync and rename; corrupt existing JSON is retained and startup reports an error. Test messages never enter the Git repository.

Acquisition files were moved to the sibling `ta-1278-source/workspace/` directory with their relative paths retained. `archive-manifest.json` stores each moved file's SHA-256. Published originals remain in this repository. The archive is not a runtime dependency. Build output remains in ignored `target/`.
