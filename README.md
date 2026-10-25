# Nokia TA-1278

[简体中文](README.zh-CN.md)

Firmware research and a working desktop phone prototype for the **Nokia 215 4G (2020), TA-1278**, firmware **17.00.17.02**.

## Run

Requires Rust and a graphics driver. Tested on macOS arm64.

```sh
cargo run --locked -p phone-desktop
```

For a local macOS app: `sh scripts/package-macos.sh`, then open `target/TA1278.app`.

- **Messages:** Chinese pinyin/stroke, Abc/abc/ABC multi-tap and direct typing, cursor movement, line breaks, character counts, drafts and a local outbox.
- **Browser:** HTTP(S) text pages, links, fields, radios/checkboxes, selects and GET forms; saved bookmarks/history/offline pages. The demo works offline.
- **Music:** play/pause, track changes, long-press seek and repeat. Progress is simulated; no audio output.
- Calendar, alarms, local contacts, calculator, timer, stopwatch and a replacement Snake game are available. Calls have no backend. **No calls or SMS are sent.**

Arrow keys navigate. Enter confirms. F1/F2 are the softkeys; Escape is also F2. Mouse **left/right/middle** map to **left softkey/right softkey/confirm**, anywhere in the window. Home returns home or asks to save a message.

In an editor: 0–9 use the selected input mode; hold for a digit. F4 switches 拼 → 笔 → Abc → abc → ABC → 123. In a candidate popup, up/down selects, left/right switches spelling/word columns, and confirm chooses. Letters and the desktop IME also work. Tab switches recipient/body. Shift+Enter inserts a line break. Backspace deletes; Cmd/Ctrl+A selects all. [Full controls and design](docs/architecture.md).

Local messages, preferences, browser data and organizer records are saved in `~/Library/Application Support/TA1278 Replica/` on macOS. Use `--data-dir PATH` for separate test data. Firmware and personal phone files are never opened by the app.

## Research

**Research is incomplete; the app has not passed full handset comparison.** Live capture and remote keys now work; 160 reviewed handset screens and a tested SMS replay are available. The app uses a black background, six recovered accent palettes, a three-column menu, floating candidates, and corrected SMS menus and save/exit flow. Prediction is a replacement implementation; original fonts take priority with Noto fallback. See [browser/SMS layout comparisons](docs/browser-layout.md), [new evidence and changes](docs/native-ui.md) and [remaining gaps](docs/fidelity.md).

- [Evidence and limits](nokia-ta1278-research/analysis/ui/replica-readiness.md)
- [SMS, browser and music rules](nokia-ta1278-research/analysis/ui/apps/interaction-core.md)
- [Remaining unknowns](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [Firmware structure](nokia-ta1278-research/analysis/firmware-structure.md), [sources](nokia-ta1278-research/sources.md) and [rebuild steps](PUBLICATION.md)
- [Actual handset screenshots and findings](docs/handset-observations.md) — live capture and keys work; full comparison is still in progress.
- [Handset automation and sampling](docs/handset-automation.md)
- [Handset screenshot investigation](docs/handset-capture.md)
- [Validation](docs/validation.md)

Original system firmware and derived assets remain in `nokia-ta1278-research/`. Private NAND/NV and temporary acquisition files are excluded. See the [firmware manifest](nokia-ta1278-research/firmware/public-system-manifest.json) and [third-party notices](THIRD_PARTY_NOTICES.md).
