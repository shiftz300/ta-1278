# Nokia TA-1278

[简体中文](README.zh-CN.md)

Firmware research and a working desktop phone prototype for the **Nokia 215 4G (2020), TA-1278**, firmware **17.00.17.02**.

## Run

Requires Rust and a graphics driver. macOS builds also need Apple Command Line Tools for the small media helper. Tested on macOS arm64.

```sh
cargo run --release --locked -p phone-desktop
```

For an optimized local macOS app: `sh scripts/package-macos.sh`, then open `target/TA1278.app`.

- **Messages:** Chinese pinyin/stroke, Abc/abc/ABC multi-tap and direct typing, cursor movement, line breaks, character counts, drafts, conversations with incoming/outgoing bubbles, inline replies, forwarding, confirmed deletion and an unsent queue.
- **Browser:** HTTP(S) text pages, links, fields, radios/checkboxes, selects and GET forms; saved bookmarks/history/offline pages. The demo works offline. [Browser capabilities](docs/browser-capabilities.md).
- **Music:** real MP3/WAV playback from virtual phone storage and memory card; pause, track changes, long-press seek, volume, repeat, shuffle and saved playlists. Local images and video open from Files; the same file picker supplies contact pictures, wallpaper and ringtones.
- Calendar, alarms, local contacts, calculator, timer, stopwatch and Snake with recovered maps/artwork are available. Phone/call-history entries are grey and disabled. **The desktop app makes no calls and sends no real SMS.**

Arrow keys navigate. Enter confirms. F1/F2 are the softkeys; Escape is also F2. Mouse **left/right/middle** map to **left softkey/right softkey/confirm**, anywhere in the window. Home returns home or asks to save a message.

In an editor: 0–9 use the selected input mode; hold for a digit. F4 switches 拼 → 笔 → Abc → abc → ABC → 123. In a candidate popup, up/down selects, left/right switches spelling/word columns, and confirm chooses. Letters and the desktop IME also work. Tab switches recipient/body. Shift+Enter inserts a line break. Backspace deletes; Cmd/Ctrl+A selects all. [Full controls and design](docs/architecture.md).

Local messages, preferences, browser data and organizer records are saved in `~/Library/Application Support/TA1278 Replica/` on macOS. Use `--data-dir PATH` for separate test data. Timers, stopwatch laps and pending reminders also survive restart. [Saving and recovery](docs/persistence.md). Firmware and acquisition archives are not opened by the app. Media access is limited to the configured VFS volumes.

Storage is an in-process VFS, with no import button or OS driver. By default, `files/` and `card/` under the data directory are mounted as `/phone` and `/card`. Put music there, then select Music → menu → Refresh. The Files app browses the same volumes.

```sh
cargo run --release --locked -p phone-desktop -- --mount-card /path/to/card
# Optional: --read-only-storage --disable network,music,files,pictures,video,sounds
# No memory card: --no-card
```

`--mount-card` uses that directory directly: file operations affect its contents. Read-only storage still allows app preferences and messages to save. [Storage, flags and limits](docs/storage.md).

[Snake code evidence](docs/snake-recovery.md) · [Alarm code evidence](docs/alarm-recovery.md) · [SMS lifecycle](docs/sms-lifecycle.md) · [SMS conversations](docs/sms-conversations.md) · [Build size and checks](docs/build-size.md)

[Delivery scope](docs/remaining-work.md) · [Local app additions and exact limits](docs/local-completion.md)

The target is a faithful standalone phone interaction replica. Exhaustive firmware reconstruction is not a release requirement.

## Research

**Research is incomplete; the app has not passed full handset comparison.** Live capture and remote keys now work; 172 reviewed handset screens and a tested SMS replay are available. The app uses a black background, six recovered accent palettes, a three-column menu, floating candidates, and corrected SMS menus and save/exit flow. Prediction is a replacement implementation; original fonts take priority with Noto fallback. See [browser/SMS layout comparisons](docs/browser-layout.md), [new evidence and changes](docs/native-ui.md) and [remaining gaps](docs/fidelity.md).

- [Evidence and limits](nokia-ta1278-research/analysis/ui/replica-readiness.md)
- [SMS, browser and music rules](nokia-ta1278-research/analysis/ui/apps/interaction-core.md)
- [Remaining unknowns](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [Firmware structure](nokia-ta1278-research/analysis/firmware-structure.md), [sources](nokia-ta1278-research/sources.md) and [rebuild steps](PUBLICATION.md)
- [Actual handset screenshots and findings](docs/handset-observations.md) — live capture and keys work; full comparison is still in progress.
- [Handset automation and sampling](docs/handset-automation.md)
- [Handset screenshot investigation](docs/handset-capture.md)
- [Validation](docs/validation.md)

Original system firmware and derived assets remain in `nokia-ta1278-research/`. Private NAND/NV and temporary acquisition files are excluded. See the [firmware manifest](nokia-ta1278-research/firmware/public-system-manifest.json) and [third-party notices](THIRD_PARTY_NOTICES.md).

Original MIDI notes and the recovered sound bank now drive local ringtone and Snake synthesis. [Audio evidence and limits](docs/native-audio.md).

[Full evidence audit and implementation mapping](docs/reverse-audit.md) · [Current package composition](docs/build-size.md)
