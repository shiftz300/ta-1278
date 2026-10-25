# Nokia TA-1278

[简体中文](README.zh-CN.md)

Firmware research and a working desktop phone prototype for the **Nokia 215 4G (2020), TA-1278**, firmware **17.00.17.02**.

## Run

Requires Rust and a graphics driver. Tested on macOS arm64.

```sh
cargo run --locked -p phone-desktop
```

For a local macOS app: `sh scripts/package-macos.sh`, then open `target/TA1278.app`.

- **Messages:** multi-tap and direct typing, cursor movement, line breaks, character counts, drafts and a local outbox.
- **Browser:** HTTP(S) text pages, links, fields, checkboxes, selects and GET forms. The demo works offline.
- **Music:** play/pause, track changes, long-press seek and repeat. Progress is simulated; no audio output.
- Calls and contacts have no backend. **No calls or SMS are sent.**

Arrow keys navigate. Enter confirms. F1/F2 are the softkeys; Escape is also F2. Mouse **left/right/middle** map to **left softkey/right softkey/confirm**, anywhere in the window. Home returns home or asks to save a message.

In an editor: 0–9 use multi-tap; hold for a digit. F4 switches abc/ABC/123. Letters and the desktop IME also work. Tab switches recipient/body. Shift+Enter inserts a line break. Backspace deletes; Cmd/Ctrl+A selects all. [Full controls and design](docs/architecture.md).

Local messages are saved in `~/Library/Application Support/TA1278 Replica/` on macOS. Use `--data-dir PATH` for separate test data. Firmware and personal phone files are never opened by the app.

## Research

The scoped investigation is ready for this prototype. **The complete original OS and interaction logic are not recovered.** T9 prediction, exact layout/timing, Dorado internals and audio services remain unresolved. The prototype uses recovered fonts, icons and selected rules, plus documented design choices.

- [Evidence and limits](nokia-ta1278-research/analysis/ui/replica-readiness.md)
- [SMS, browser and music rules](nokia-ta1278-research/analysis/ui/apps/interaction-core.md)
- [Remaining unknowns](nokia-ta1278-research/analysis/ui/reverse-engineering-status.md)
- [Firmware structure](nokia-ta1278-research/analysis/firmware-structure.md), [sources](nokia-ta1278-research/sources.md) and [rebuild steps](PUBLICATION.md)
- [Validation](docs/validation.md)

Original system firmware and derived assets remain in `nokia-ta1278-research/`. Private NAND/NV and temporary acquisition files are excluded. See the [firmware manifest](nokia-ta1278-research/firmware/public-system-manifest.json) and [third-party notices](THIRD_PARTY_NOTICES.md).
