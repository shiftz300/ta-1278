# TA-1278

[简体中文](README.zh-CN.md)

[![CI](https://github.com/shiftz300/ta-1278/actions/workflows/ci.yml/badge.svg)](https://github.com/shiftz300/ta-1278/actions/workflows/ci.yml)
[![Android](https://github.com/shiftz300/ta-1278/actions/workflows/android.yml/badge.svg)](https://github.com/shiftz300/ta-1278/actions/workflows/android.yml)

A user-space phone interaction runtime and desktop/Android app, with a classic keypad-phone interface.

[Phone](docs/calls.md) · [Service interface](docs/services.md) · [Input engine](docs/input-engine.md) · [Architecture](docs/architecture.md)

## Screenshots

The app display, without the system titlebar. Messages and browser pages use demo data.

| Messages | Snake | Browser |
| --- | --- | --- |
| <img src="docs/screens/compose.jpg" width="240" alt="Desktop app: message editor"><br>Message editor | <img src="docs/screens/snake.jpg" width="240" alt="Desktop app: Snake running"><br>Playing | <img src="docs/screens/browser.jpg" width="240" alt="Desktop app: browser text and form"><br>Text and forms |
| <img src="docs/screens/messages.jpg" width="240" alt="Desktop app: message conversation"><br>Conversation | <img src="docs/screens/menu.jpg" width="240" alt="Desktop app: Snake menu"><br>Menu | <img src="docs/screens/script.jpg" width="240" alt="Desktop app: JavaScript content and click result"><br>JavaScript and events |

## Embed

The user-space `phone_runtime` library can run without a window. The host supplies
keys, time, message services and a place to save state. It returns a 240×320 RGBA
frame. [Embedding API and current limits](docs/runtime.md).

```sh
cargo run -p phone-desktop --no-default-features --locked --example embed -- /tmp/ta1278-embed
```

## Run

Targets macOS (arm64/Intel), Windows x64 and Linux x64. [Downloads](https://github.com/shiftz300/ta-1278/releases/latest) · [Platform limits, CI and releases](docs/release.md). Building requires Rust, a graphics driver and the platform C/C++ tools; macOS also needs Apple Command Line Tools for the media helper.

```sh
cargo run --release --locked -p phone-desktop
```

Android ARM64 APKs are also available in [Releases](https://github.com/shiftz300/ta-1278/releases/latest). `python3 scripts/android.py` builds a locally signed APK. [Setup, signing, controls and platform limits](docs/android.md). On desktop, add `--keypad` for the same on-screen handset buttons. The titlebar is hidden by default; `--titlebar` restores it.

For an optimized local macOS app: `sh scripts/package-macos.sh`, then open `target/TA1278.app`.

- **Messages:** Chinese pinyin/stroke, Abc/abc/ABC multi-tap and direct typing, cursor movement, line breaks, character counts, drafts, conversations with incoming/outgoing bubbles, inline replies, forwarding, confirmed deletion and an unsent queue.
- **Browser:** HTTP(S) text pages, bounded basic JavaScript/DOM, links, fields, radios/checkboxes, selects and GET forms; saved bookmarks/history/offline pages. The demo works offline. [Browser capabilities](docs/browser-capabilities.md) · [Script security and limits](docs/javascript.md).
- **Music:** real MP3/WAV/MIDI playback from the chosen phone-storage folder; pause, track changes, long-press seek, volume, repeat, shuffle and saved playlists. Local images and video open from Files; the same file picker supplies contact pictures, wallpaper and ringtones.
- Calendar, alarms, local contacts, calculator, timer, stopwatch and Snake with recovered maps/artwork are available. Dialing and local attempted-call history are enabled; no call transport is connected. **The desktop app makes no calls and sends no real SMS.**

Arrow keys navigate. Enter confirms. F1/F2 are the softkeys; Escape is also F2. Without `--keypad`, mouse **left/right/middle** map to **left softkey/right softkey/confirm**, anywhere in the window. Home returns home or asks to save a message.

In an editor: 0–9 use the selected input mode; hold for a digit. # (Shift+3 on a computer keyboard) switches 拼 → 笔 → Abc → abc → ABC → 123. In a candidate popup, up/down selects, left/right switches spelling/word columns, and confirm chooses. Host typing uses fixed US-English key positions (Shift for uppercase/symbols). System IME composition is disabled; Chinese input uses the phone’s internal modes. Tab switches recipient/body. Shift+Enter inserts a line break. Backspace deletes; Cmd/Ctrl+A selects all. [Full controls and design](docs/architecture.md).

Local messages, preferences, browser data and organizer records are saved in `~/Library/Application Support/TA1278 Replica/` on macOS. Use `--data-dir PATH` for separate test data. Timers, stopwatch laps and pending reminders also survive restart. [Saving and recovery](docs/persistence.md). Firmware and acquisition archives are not opened by the app. Media access is limited to the selected local folder.

On first launch, choose a folder for **Phone storage**. The app remembers it. Files, music, pictures and video use this folder directly; changes affect its contents. There is no VFS or import button. Put music in it, then use Music → menu → Refresh. MIDI extensions (`.mid` / `.midi`) are case-insensitive and use the bundled SF2 sounds. [MIDI support and limits](docs/storage.md#midi-music).

```sh
cargo run --release --locked -p phone-desktop -- --storage-dir /path/to/phone-files
# Optional: --read-only-storage
```

[Unified config](docs/config.md) controls optional apps, capabilities and storage. All apps stay enabled by default; core apps remain. Use `--config config/default.json` or put `config.json` in the profile.

`--storage-dir` changes the saved folder. Private settings and messages stay in the app profile. [Storage, flags and limits](docs/storage.md).

[SMS conversations](docs/sms-conversations.md) · [Build size](docs/build-size.md)

[Delivery scope](docs/remaining-work.md) · [Local app additions and exact limits](docs/local-completion.md)

[UI review](docs/review.md) · [Local acceptance and repeatable checks](docs/acceptance.md)

The target is a faithful standalone phone interaction replica. Exhaustive firmware reconstruction is not a release requirement.

## Repository scope

This repository contains the application, runtime resources and regression tests. Firmware images, research tools, disassembly, handset captures and analysis reports are maintained separately. Building and testing require neither the research archive nor a connected phone. Original fonts, artwork, tones and T9 data remain bundled; their rights and provenance are retained in [third-party notices](THIRD_PARTY_NOTICES.md) and [assets](assets/README.md).
