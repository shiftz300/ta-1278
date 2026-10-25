# Validation — 2026-10-25

Host: macOS arm64, Rust 1.96.0. Runtime: winit 0.30.13, wgpu 30.0.1, Apple M5 Pro / Metal. Other operating systems have not been tested.

## Automated

```sh
cargo fmt --all -- --check
cargo test --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo build --release --workspace --locked --offline
target/release/phone-desktop --smoke-test /tmp/ta1278-smoke
```

The offline commands require a prior Cargo dependency fetch. The smoke run opens a real window, presents 51 home/menu/SMS/browser/music/theme, organizer, utility and game scenarios through wgpu and exits. Its PNGs are the CPU framebuffer uploaded to the GPU, not GPU readbacks. The latest run reported 52 presented frames using Metal. All 51 previews and a contact sheet are in `docs/images/`, with SHA-256 hashes. Menu, theme, SMS/recipient/prediction, browser controls, music, organizer and utility layouts were inspected. These are replica previews, not handset screenshots.

61 Rust tests pass (27 core, 34 desktop). `scripts/check-recovered-table.py` also checks all ten Latin multi-tap rows against the extracted firmware table.

Tests cover multi-tap replacement/timeout, UTF-16 edits and limits, SMS language/count boundaries, draft replacement and local queueing, independent keyboard/mouse holds, long press/cancellation, stale browser responses, history/error handling, HTML fields, GET query encoding, checkbox/select reset, field cancel/commit, multi-select Back, music seek/release and EOF, storage round-trip/corrupt-file retention, and drawing each screen with SMS overlays. Additional tests cover candidate confirmation before sending, spelling filtering, preview text without committing it, pixel-based cursor movement across short lines, menu focus restoration, insert-submenu return paths, and save failure/retry handling. These are synthetic tests, not equivalence proof against the handset.

## Actual window checks

The earlier baseline checks below passed before this visual correction. This round additionally checked nine-key Chinese candidate entry, spelling-column navigation, commit, the corrected three-item SMS body menu, Save as draft, and reopening the saved recipient/body in the native macOS window. After the black-background change, the rebuilt native window also passed keyboard navigation to Settings, changing the accent, and returning with focus still on Settings. These older checks do not establish a match with the physical handset, and old menu/mouse routes below describe the earlier prototype. The current Home routes are configurable left/right shortcuts and Center opens the grid.


Using native UI input, verified:

- Left mouse enters the menu, right mouse returns home, middle mouse confirms. No button-region hit test is used.
- Keyboard recipient entry `123`; direct text `hi` through the current desktop IME; SMS display `158 / 1`.
- Save a draft, leave the editor and reopen it with the same recipient/body.
- Queue locally and open the outbox entry, which displays **NOT SENT**.
- Open the demo link to `https://example.com` and display the returned Example Domain page.

The first window run found a panic when drawing a compose screen under an overlay. The view used an active-editor accessor which returns `None` while a modal is open. It now reads the compose editor directly; a rendering regression test covers both recipient/body focus and the menu/exit/notice overlays. The full save/reopen/outbox path passed in the rebuilt window.

Native handset pixel matching and physical key timing are unverified. Chinese/English prediction is now a tested replacement using public offline dictionaries; the original T9 engine is not restored. Real SMS/call services and audio playback remain unimplemented. Menu/window and browser behavior includes the explicit host policies in [architecture.md](architecture.md).

## Research and archive

The existing Python application contracts still pass 75 checks. Before app implementation, an investigation checkpoint with explicit unresolved items was recorded in commit `cac59db` and pushed. It did not complete the reverse engineering. The preceding application reconstruction had 79 matching regenerated outputs from public system originals and 63 hash-verified reference texts; that historical result is not presented as a fresh full firmware rebuild in this app phase.

353 ignored local acquisition/reference/intermediate files (739,586,385 bytes) were moved to the sibling `ta-1278-source/workspace/` tree. Each destination was checked against its pre-move SHA-256. Tracked originals and public results were not moved. The private archive manifest is outside Git. New build files stay in ignored `target/`; GUI test data stays outside the repository; selected framebuffer previews are copied into `docs/images/`.

## Captured framebuffer previews

![Menu](images/menu.png) ![SMS editor](images/sms.png)

A further 47,021 task-specific temporary files (1,190,437,786 bytes), including earlier isolated rebuild exports and GUI test frames, were moved from `/private/tmp/ta1278-*` to the external archive `temporary/` tree and verified individually. `temporary-manifest.json` retains their hashes. The final public research manifest verifies 6,670 files.

## Handset automation tooling and actual captures

2026-10-25: runtime version, screenshot and remote-key traffic succeeded on firmware 17.00.17.02, USB 2e04:0019, interface 1. This is separate from desktop replica validation.

- `PYTHONPATH=scripts python3 -m unittest handset.test_handset`: 14 host protocol/failure tests pass.
- `python3 scripts/recover-runtime-entry.py`: verifies entry/side-effect code and generated contact formats against the original firmware hash.
- `PYTHONPATH=scripts python3 -m handset.compare_theme`: each of the six native screenshot swatches is nearest to shade 3 of its independently recovered palette. JPEG conversion remains lossy.
- 200 numbered sessions: 200 verified version replies, 2,055 decoded captures, 927 key ACKs, 183 completed plans, six capture-only completions and 11 stopped attempts. These are execution counts, not feature coverage percentages. Four invalid-image attempts are included in the stopped cases; originals were retained for diagnosis.
- [Numeric SMS regression](handset-plans/sms-numeric-boundary-result.json): 172 steps, seven matching image postconditions, generated text cleared. Menu reentry in session 192 adds one matching postcondition.
- 112 explicitly reviewed public screenshots in [sample index](handset-samples/20261025/index.json). Original contacts/alarms, inbox previews, personal wallpaper, call history and media names are excluded. One verified test-mode-generated contact is included as a synthetic fixture.
- Follow-up sessions 201–270: 70 version gates, 890 decoded captures, 410 acknowledged key actions, 64 completed plans and six capture-only completions; no stopped runs. SIM insertion/reboot preceded session 204. Browser text, forms, links, Back/Forward and a single-select now have live evidence. Five image assertions passed, including the two published browser replay plans (two assertions each). There are now **136** reviewed public screenshots, 24 added in this follow-up. These counts measure executions, not feature coverage. Original raw responses and host test-page snapshots remain in the external hash-checked archive.
- 5,677 files are in the private local session archive; 5,647 belong to the numbered sessions. Copied files were SHA-256 checked. Raw images and original/private screens were not added to Git.

See [observations](handset-observations.md) for state effects, partial paths and failures. Full handset sampling and replica comparison remain incomplete. Web content did not load in the early pre-SIM state; later post-SIM runs above loaded public HTML/HTTPS pages. Active recording, real calling and sending, existing personal records, exhaustive settings and all input boundaries were not exercised. No desktop layout equivalence is claimed from these protocol tests.

Follow-up 271–283 verified the configurable lock-screen shortcuts without changing them: 13 version gates, 97 captures, 42 key actions, 12 completed plans and one capture-only completion. Six reviewed settings images bring the public total to **142**. This resolves the earlier Home shortcut ambiguity through a settings read and owner clarification, not a speculative key remap.


## Earlier implementation pass (before the layout follow-up)

This pass used existing firmware and captured handset evidence; no new handset command was sent. Added tests cover six input modes and stroke selection, URL wrap, readonly controls, radio grouping, label association, browser/offline persistence, settings cancellation, local organizer save/cancel/recipient selection, timer background state, calculator operations, leap dates, Snake pause/reversal, alarms and every non-excluded menu route. See [current gaps](fidelity.md).

The first 33-scene run was occluded: no GPU frame was presented and it repeatedly requested redraw. A process sample identified the redraw loop. Bringing that test window forward allowed all scenes to finish. The host now throttles redraw requests, skips occluded drawing, advances smoke stages only after a new presented frame and fails after 30 seconds rather than waiting forever. The rebuilt run completed automatically with **34 presented frames**, without a manual activation step. This is distinct from the older checks above.

The final commands passed: format check, 51 Rust tests, Clippy with warnings denied, asset SHA/glyph coverage (8,744 → 49,332 codepoints), all ten recovered multi-tap rows and `git diff --check`. Private handset images were not added to the screenshot collection. Music layout was compared using a local masked geometry view; its filename was not copied into app fixtures or public previews.


## Browser/SMS layout follow-up

2026-10-25: sessions 284–312 added actual observations instead of inferring settings from the old replica. 29 version gates, 107 captures, 39 key ACKs, 27 completed plans and two capture-only sessions. The public allowlist now has 160 native screens. No clear/reset or network form submission was performed. The temporary image toggle was restored OFF. See [observations](handset-observations.md).

Final checks: 56 Rust tests (24 core, 32 desktop), Clippy with warnings denied, format check, asset SHA/glyph coverage and diff whitespace checks passed. New tests cover DOM-control identity across skipped nodes, inline form coordinates, wrapped-link focus bounds, old offline JSON compatibility, native settings toggle/cancel and Center-only browser exit. The 14 handset host-protocol tests also pass. Python syntax checking first hit the restricted macOS bytecode-cache path; it passed with `PYTHONPYCACHEPREFIX=/private/tmp/ta1278-python-cache`.

The final winit/wgpu Metal run generated **43 framebuffer previews with 44 presented frames**. Eleven labeled handset/replica comparisons were generated with `scripts/compare-handset-layout.py` and visually reviewed. These PNGs remain CPU framebuffers presented through wgpu, not GPU readbacks. The final local `target/TA1278.app` was rebuilt and its ad-hoc signature verified. No install or push was performed.

The browser-layout pass used several measured iterations: inline flow first, then footer/popup geometry, then focused settings row heights. The final previews include the actual two-choice font and clear menus. Clear activation remains unavailable, rather than pretending the previously invented bookmarks/offline clear list is native. Fonts, rounded controls, complex HTML and unobserved app branches still differ; test success is not full replica acceptance.

The 256 temporary layout-run files were moved to `../ta-1278-source/replica-validation/20261025-browser-layout/` and checked by SHA-256 before removing their temporary copies. Raw device sessions 284–312 were likewise retained in the verified external handset archive; duplicate temporary directories were removed. Public comparisons and replica previews remain in the project.

## Release build and recovered Snake follow-up

The previous working state was committed and pushed as `a1efd51` before this follow-up. The packaging script now uses release optimization; bundled executable is 36,772,320 bytes, about 48.5% below the old debug executable. [Size and code audit](build-size.md) records limits and hashes.

New Snake tests cover extracted timer/map values, initial body, wrap, old-tail collision, queued turns, fruit/bonus rules, settings preview/cancel, paused-game reset, per-map score insertion and corrupt-settings retention. 61 Rust tests (27 core / 34 desktop), Clippy with warnings denied, formatting, resource hashes/glyph coverage, recovered multi-tap rows and whitespace checks passed. The final signed app produced 51 scenes / 52 presented frames on Metal. Fifteen native/replica pairs include four Snake comparisons.

`recover-snake.py` checks original hashes, pointers and selected instruction operands, and records bounded code bytes. It verifies the extracted inputs, not full execution equivalence. Native game PRNG, timing parity, sound and death/win effects remain open. Complex browser compatibility was deprioritized at the user's request.

The 192 validation files from this follow-up were hash-verified into `../ta-1278-source/replica-validation/20261025-release-snake/`; temporary copies were removed. The 146 files in temporary handset sessions 313–329 were removed only after comparison with the external archive. Public evidence and previews remain in the repository.

## Alarm code and control follow-up

`recover-alarm.py --disassemble` verifies the original firmware and extracts seven weekday masks and five snooze choices. [Code and handset evidence](alarm-recovery.md) separate native behavior from the local scheduler. 27 new connections generated 83 captures and 28 guarded key actions; seven reviewed images bring the public total to 172. Session 338 was rejected locally before any device action. One newly created test alarm was removed immediately; original records remained unchanged.

68 Rust tests (27 core / 41 desktop), warnings-denied Clippy, formatting, asset hashes/coverage, 14 handset host-protocol tests and whitespace checks pass. The earlier test expecting the old generic alarm notice failed after the dedicated alert queue was introduced; it was updated to check the queue and same-minute deduplication. New tests cover draft cancellation, picker apply/cancel differences, name limits, transient time edits, native weekday bits, multiple alerts, midnight snooze, interrupted editor retention and capacity/delete-all cancellation.

The final signed release produced 60 synthetic scenes / 61 presented Metal frames. Twenty labeled comparisons include five new alarm pairs; checkbox marks, selected radio contrast, footer icons and bottom menu geometry were adjusted after visual review. The complete app is 36,873,860 bytes. These are local implementation checks, not complete handset acceptance. Original alarm records were not copied into the app or public screenshots.

The 144 temporary validation/disassembly files were SHA-256 checked into `../ta-1278-source/replica-validation/20261025-alarm/`. The 241 raw files from sessions 330–357 were checked against the external handset archive before removing temporary duplicates. That archive now contains 8,607 files. Public images and code evidence remain in this repository; this continuation is not committed or pushed.

## SMS lifecycle follow-up

The queue now waits for the actual local store write before closing its editor. On failure, it restores the removed draft, rolls back only the staged outbox item and keeps the edited text. New local views support reader scrolling, retry without duplicate records, edit-as-new, forwarding and delete confirmation; their geometry is not claimed to be the native conversation UI. Old oversized drafts reopen without silently dropping text.

`recover-sms-lifecycle.py` checks the immutable firmware and reviewed code hashes, regenerates a bounded Thumb listing and evaluates all 65,536 network-reason inputs. It recovers the 19-entry retry table and checks seven UI operation-label literal references. The Rust pure retry decision is separate from desktop no-service handling. No device operation or private-message access occurred.

76 Rust tests (29 core / 47 desktop) pass, including an actual temporary-file write failure followed by rollback/retry, interrupted draft preservation, stale receipt refusal, reader focus/scroll, copy/forward/delete cancellation, native operation-label lookup and exhaustive retry priority. Clippy with warnings denied, formatting and asset checks pass. The first Clippy pass flagged three unnecessary clones in tests; they were removed.

The final signed release produced 67 synthetic framebuffers / 68 presented Metal frames. The new SMS scenes were visually checked. Final comparisons were regenerated after correcting the empty-recipient caret. The app bundle is 36,890,659 bytes (about 35.18 MiB). No SMS reader comparison to native conversation pixels is claimed.

Visual follow-up found long forwarded text covering the recipient field. The existing native recipient-focus capture supplied the y=91 body bound. The fix limits inactive-body height and removes its stray caret; a pixel-region regression checks that the header and recipient cursor remain unchanged. A 21st comparison pairs these focus states, with deliberately different synthetic text.

All four temporary SMS validation runs were SHA-256 checked into `../ta-1278-source/replica-validation/20261025-sms-lifecycle/` before their temporary copies were removed. Current public screenshots and 21 comparison pairs remain in the repository. This continuation is uncommitted.


## VFS, audio and Home footer follow-up (2026-10-25)

- 84 default Rust tests passed (29 core, 52 desktop, 3 VFS); one hardware-dependent test is excluded from the default run. Warnings-denied Clippy, fmt, resource hashes and recovered-table checks passed.
- The explicit audio test passed separately with a generated silent WAV and MP3 on CoreAudio. Advancing clock, pause, seek and EOF were asserted; no listening-quality claim.
- A real winit window browsed the mounted card/Music directory, opened Validation.mp3, displayed EOF at 0:03 and saved the virtual path/position on close. `vfs-*-window.png` are window captures, not handset evidence.
- The signed app completed 67 synthetic scenes / 68 presented Metal frames after the Home footer correction.
- [Home footer bounds](evidence/home-softkeys.json) compare original icon resources with native session 283. The side icons and center text now share the native footer geometry; one-pixel raster differences remain. Other shortcut icon assignments are still fallback artwork.
- [Storage boundaries](storage.md) distinguish the shipped directory VFS from future image and hot-plug backends.


## VFS failure paths and common softkeys follow-up (2026-10-25)

- 88 default tests pass (29 core, 56 desktop, 3 VFS). Four new tests cover failed navigation without losing Back history; pending-scan save and failed-write retention; stale-scan rejection after rename/delete; and not inventing a played track during initial scanning.
- Native menu/back resources 01160 / 01151 replace font-character stand-ins in common footers and browser chrome. They use the same 26×26 slots at (5,287) and (208,287) as the native captures. Music and Files center labels use measured footer alignment.
- Music Home uses the captured 全部播放 label, 84px first row, play triangle and 播放 center action. The triangle is a measured drawing; menu/back icons are recovered assets. The smoke fixture may show a current-track row absent from the empty native library.
- Configurable Home shortcuts other than Opera/Messages still use fallback menu artwork. A targeted literal lookup did not establish their native mapping, so no additional mapping is claimed recovered.
- The previous explicit WAV/MP3 CoreAudio evidence remains separate; the audio decoder/output module did not change in this follow-up.
