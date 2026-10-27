# Native page audit — 2026-10-27

This is an evidence inventory and correction pass, **not a claim of complete reverse engineering**. A working local route, a firmware registration, a handset screenshot and a verified transition are different evidence.

## Quiet-time boundary

The owner asked for silence while sleeping. No more handset key injection, music/ringtone preview, alarm/timer start, audio-output test or vibration test is performed during this period. The test timer had already been stopped; the final passive screenshot shows the empty timer form, disabled Start and a muted status icon. That observation does not prove that every possible unsolicited handset alert is disabled. No new silent-mode setting was sent.

Automated desktop checks instantiate data models and CPU image rendering only. They do not open an audio device or connect to USB. Tests marked as requiring real audio/video output remain ignored. Do not run them with `--ignored` during quiet time. Any later window check must use an isolated profile with `--disable sounds,music,video,network`.

## Fresh handset evidence

The existing hub connection was verified as Queen2 Phone `2e04:0019`, diagnostic interface 1 (IN `83`, OUT `04`). Firmware replied **17.00.17.02**. A raw tap ACK did not change the initial menu; explicit down/up was required. Some short presses were missed, so observed focus—not ACK count—was used before later actions. No USB reset, driver installation, firmware load or flash operation was involved.

[14 retained native PNGs and hashes](handset-samples/20261027/index.json) cover:

- Files → 手机存储; seven folder options; move destination; folder-name editor.
- Other → Timer; empty name disables Start; entering synthetic `A` enables it; Start returns to Other with remaining time.
- Opening a running timer shows 编辑定时器 / 停止计时. Expiration uses a dedicated hourglass panel: Center 重计时, Right 停止. After stopping, reopening shows an empty name.

Two capture names were wrong and have been corrected in the index. A missed Down opened Delete selection instead of Move; nothing was selected or deleted, and the page was cancelled. A stop attempt found that the timer had already expired; the screenshot guard rejected its key. The following session stopped the alert. These actions all preceded the quiet-time request. No personal message/contact/file content was exported.

## Application boundaries

References under `handset-samples/20261025/` are earlier physical captures. The entries below cover every retained top-level application, plus shared navigation/input and media subpages. An entry in this table does not mean every branch has been sampled.

| Application | Evidence used | Replica / unresolved boundary |
|---|---|---|
| Home / menu | `lock-screen-shortcuts.png`; [observations](handset-observations.md) | Observed order and configurable shortcuts; exact lock/status behavior and vertical edges remain open. |
| Text input | [Native T9 code, dictionaries and runtime](t9-recovery.md) | Original candidate engine integrated. Complete semantic decompilation, all native input widgets and font rasterization remain unproven. |
| Messages | [Real loopback](sms-handset-loopback.md), [composer](sms-composer-layout.md), [conversation limits](sms-conversations.md) | Local send/receive lifecycle and service interface; no real backend. Bubble direction/interruption branches need more physical evidence. |
| Browser | [Code checks](evidence/browser-code-audit.json), [capabilities](browser-capabilities.md), [layout](browser-layout.md) | Basic text/link/form/history/bookmark/offline paths. Clear/share behavior uses stated local policies. No original rendering engine, JavaScript or complex-page parity. |
| Calendar | `calendar-root.png`, `calendar-options.png`, `calendar-new-event.png`, repeat/ringtone/note captures | Inline text/date/time/choice form now uses shared controls and clipped scrolling; three 40-unit text fields and center Save are firmware-verified. Local recurrence, digit validation, defaults and Save popup; lunar/holiday data and complete native edit transitions remain open. |
| Alarms | [Recovered descriptors/handlers](alarm-recovery.md), repeat/day/snooze captures | Shared inline form, repeat/snooze and local persistence. Full interruption/audio/wake parity unproven. No live alert test during quiet time. |
| Contacts | [Code and layout](contact-torch-layout.md), test-contact detail/options/delete captures | Native-shaped fields/list/details; photo/group/ringtone management and matching rules remain partly local. Calls disabled. |
| Settings | [Palettes](evidence/live-theme-comparison.json), `system-settings.png`, personalization/shortcut captures | Supported settings have local effects. Security/SIM/USB/Bluetooth/native advanced branches not reproduced. |
| Music | `music-options.png`, `music-refresh.png`; [local playback](storage.md) | Actual MP3/WAV playback exists, but it is not exercised during quiet time. Playlist policy, tags/art and full interruption/player parity remain gaps. |
| Files | New folder/menu/name/move captures; earlier storage Details capture | VFS removed; one directly selected folder. Native-shaped overview/options/name/destination and checkbox selection. Multi-delete is implemented with confirmation and empty-directory preflight; selection actions and completion/copy/move outcomes are local policies, not fully sampled native flows. |
| Pictures / video | [Local media implementation](local-completion.md) | Shared folder browser and system decoders. Full native pages, rotation and codec parity remain unverified; no video playback during quiet time. |
| Flashlight | [Original images and handler evidence](contact-torch-layout.md) | Recovered power images and colors; host has no LED service. |
| Snake | [Recovered board, states, maps and resources](snake-recovery.md) | Observed game-over/level behavior, original art and RNG update. Seed/scheduling/full-board/error parity remain open. Audio output not exercised. |
| Calculator | `calculator-root.png`, add/chain/delete captures; native window handler `0x88b919a1` | Corrected expression/result/control placement. % no longer exits; unary x/100 is explicitly a local arithmetic policy. Full native percent chaining/precision/error rules not recovered. |
| Other / Timer | New physical transitions; [verified descriptors/code](evidence/native-timer.json) | Corrected two-line root rows, inline time/name, running menu and dedicated expiration. Name limit 20 UTF-16 units and center Start are code-verified. All editing/error limits and native timing internals remain open. |
| Stopwatch | root/running/lap/stopped/reset captures | Corrected one-line elapsed time, lap rows and state-dependent softkeys. Local clock and restart policy; native maximum/overflow behavior unknown. |
| Converter | `unit-converter.png`, temperature fields/result/reverse captures | Local arithmetic; remaining unit lists/rates and all native numeric conventions not recovered. |
| FM radio | `radio-root.png`, `radio-options.png`, close-option capture | Display/control placement revised; local frequency/favorites only. No FM backend or claim of received audio. |
| Recorder | `recorder-root.png` | Native-shaped rings, timer and controls. Existing recorder stores elapsed time only; it does not record microphone audio. Full recording/file/error pages remain open. |
| Counter | `counter-empty.png` | Empty local page only; useful underlying native behavior remains unknown. |
| Telephone | Explicit owner exclusion | Grey and inert; no call screen/backend. Alipay and Ximalaya are removed entirely. |

## Code and layout corrections

`scripts/check-native-timer.py` verifies the original user image SHA-256, both countdown descriptors and the current form's child controls. The current `MMIACC_PRIMO_CT_WIN_ID` is at file offset `0x710000`, handler `0x880a437f`; its text limit is `0x14` and center resource is Start. The older countdown descriptor has different softkeys and must not be used to infer this form. The two-byte readiness check at `0x880a46fa` has not been fully traced to named fields; the name requirement is supported by the physical empty/A experiment.

The expiration hourglass is original resource `0x4017002e`, a 52×52 image at MMI offset `0xabc90c`, placed at (94,132). It replaces the temporary geometric hourglass. Its source bytes are verified; the image match is visual evidence, not a traced render-call chain.

The storage/folder glyphs, calculator operators, percent, stopwatch Lap/Reset and recorder buttons now use original images through shared controls instead of geometric substitutes. [Fourteen verified control assets](evidence/utility-art.json) are checked against MMI bytes; the binding comparisons are visual, not full render-call recovery.

The shared list/form/editor/softkey modules remain in use. Added tests cover empty-name gating, invalid minute digits, edit cancel, running-menu stop, expiry restart versus stop, restart persistence, transfer-destination cancellation and readonly copy controls. CPU captures check the name caret stays at y=125 as text grows and leaves the time field untouched.

## Reproduce without sound or device access

```sh
python3 scripts/check-calendar-files.py
python3 scripts/check-native-timer.py
python3 scripts/check-utility-art.py
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
TA1278_AUDIT_FRAMES=/tmp/ta1278-audit-frames cargo test -p phone-desktop --locked views::audit_tests
sh scripts/package-macos.sh
```

The optional CPU render tests produce 24 synthetic PNGs without creating a window, media service or audio output. It is not a winit presentation test. First-launch folder selection is covered through an injected chooser, cancellation/reuse and real temporary folders; the actual macOS chooser still needs visual acceptance. Packaging does not automatically launch the app. See [storage](storage.md) for direct-folder behavior and migration limits.


## Earlier validation result

176 default tests passed; four opt-in tests remain ignored. Warnings-denied Clippy, format, diff checks and release packaging/signature verification passed. [Results, hashes and 16 CPU-rendered frames](evidence/native-page-audit-validation.json). The release app was not launched. Existing unrelated edits are preserved; this pass is not committed or pushed.

## Local-page completion follow-up

Calendar now edits title, location and notes inline, uses separate date/time controls, saves choices together and preserves the original start date when editing a recurring occurrence. Scrolling clips content without overwriting the title or softkeys. The three 40-unit text limits and center Save come from [six verified firmware descriptors and bounded code listings](evidence/calendar-files.json). Default values, digit-by-digit validation, recurrence and the one-item Save popup remain local policies.

Files now supports checkbox selection, select all, cancellation and confirmed background deletion. Read-only folders reject mutation. Empty-directory and all-path preflight checks prevent recursive deletion; a completed confirmation survives returning home or trying to reopen Files. Create/rename/delete/copy/move block closing while pending. Filtering file/folder selections restores the original list cursor on cancel.

181 default tests passed, four real-output tests remained ignored; warnings-denied Clippy and optimized packaging passed. Twenty-four CPU frames were produced. No physical-phone command, audio output or vibration was exercised. The isolated macOS first-run chooser process started but computer-use attachment failed (invalid helper app / timeout); both test processes were terminated. This is **not** successful visual acceptance of that dialog. Cancellation, selection, saved-path reuse, unavailable paths and read-only behavior are covered by injected-chooser tests using real temporary folders.

[Current validation record](evidence/local-pages-completion-validation.json). The per-app table above retains native fidelity gaps, including recorder audio, FM reception, calculator percent semantics and unsampled media/error branches. No commit or push was performed.
