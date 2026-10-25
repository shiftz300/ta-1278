# Validation — 2026-10-25

Host: macOS arm64, Rust 1.96.0. Runtime: winit 0.30.13, wgpu 30.0.1, Apple M5 Pro / Metal. Other operating systems have not been tested.

## Automated

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo build --workspace --locked --offline
target/debug/phone-desktop --smoke-test /tmp/ta1278-smoke
```

The offline commands require a prior Cargo dependency fetch. The smoke run opens a real window, presents home/menu/SMS/browser/music through wgpu and exits. Its PNGs are the CPU framebuffer uploaded to the GPU, not GPU readbacks. The run reported six presented frames using Metal. All five images were checked for readable layout, including Chinese text and the SMS cursor.

20 Rust tests pass. `scripts/check-recovered-table.py` also checks all ten Latin multi-tap rows against the extracted firmware table.

Tests cover multi-tap replacement/timeout, UTF-16 edits and limits, SMS language/count boundaries, draft replacement and local queueing, independent keyboard/mouse holds, long press/cancellation, stale browser responses, history/error handling, HTML fields, GET query encoding, checkbox/select reset, field cancel/commit, multi-select Back, music seek/release and EOF, storage round-trip/corrupt-file retention, and drawing each screen with SMS overlays. They are synthetic tests, not equivalence proof against the handset.

## Actual window checks

Using native UI input, verified:

- Left mouse enters the menu, right mouse returns home, middle mouse confirms. No button-region hit test is used.
- Keyboard recipient entry `123`; direct text `hi` through the current desktop IME; SMS display `158 / 1`.
- Save a draft, leave the editor and reopen it with the same recipient/body.
- Queue locally and open the outbox entry, which displays **NOT SENT**.
- Open the demo link to `https://example.com` and display the returned Example Domain page.

The first window run found a panic when drawing a compose screen under an overlay. The view used an active-editor accessor which returns `None` while a modal is open. It now reads the compose editor directly; a rendering regression test covers both recipient/body focus and the menu/exit/notice overlays. The full save/reopen/outbox path passed in the rebuilt window.

Native handset pixel matching, physical key timing, Chinese T9 prediction, real SMS/call services and audio playback were not tested or implemented. Menu/window and browser behavior includes the explicit host policies in [architecture.md](architecture.md).

## Research and archive

The existing Python application contracts still pass 75 checks. Before app implementation, the necessary investigation was closed with explicit unresolved items in commit `cac59db` and pushed. The preceding application reconstruction had 79 matching regenerated outputs from public system originals and 63 hash-verified reference texts; that historical result is not presented as a fresh full firmware rebuild in this app phase.

353 ignored local acquisition/reference/intermediate files (739,586,385 bytes) were moved to the sibling `ta-1278-source/workspace/` tree. Each destination was checked against its pre-move SHA-256. Tracked originals and public results were not moved. The private archive manifest is outside Git. New build files stay in ignored `target/`; GUI test frames/data are outside the repository.

## Captured framebuffer previews

![Menu](images/menu.png) ![SMS editor](images/sms.png)

A further 47,021 task-specific temporary files (1,190,437,786 bytes), including earlier isolated rebuild exports and GUI test frames, were moved from `/private/tmp/ta1278-*` to the external archive `temporary/` tree and verified individually. `temporary-manifest.json` retains their hashes. The final public research manifest verifies 6,670 files.
