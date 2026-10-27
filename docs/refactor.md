# Runtime ownership refactor

Verified locally on macOS on 2026-10-27, against baseline commit `46fefa0`. No dependencies or firmware resources were added. Included in version `0.2.3`.

## Changes

- Phone retains routing/shared interaction; message, browser, player, Snake, contact, call, organizer, settings, media and file state have explicit owners. `organizer::LocalData` is a compatibility serialization DTO, not the live cross-application store.
- Platform ports exchange requests, core playback controls, pixels and session commands. No port receives Phone. App-side application of completions lives under Runtime; selected files remain a real folder mapping.
- Views paint with immutable Phone access. A separate no-raster measurement pass supplies derived geometry before input. Runtime rendering does not mutate app state or advance time.
- Runtime owns profile/checkpoint capture, validation, dirty acknowledgment and close policy through Store. Directory owns atomic JSON replacement, backups and the optional lifetime instance lock. Original profile filenames/schemas, five-second periodic saves and explicit wall-clock/paused resume policies remain.
- The first record failure stops subsequent writes. Mailbox failure rolls back Send; a failure before the mailbox leaves it pending. Submit and receive acknowledgment still require durable mailbox commit.
- Desktop no longer serializes app fields or implements draft/save-failure close decisions. Media no longer saves profiles. Background flush serializes only selected dirty/periodic records rather than cloning the entire state on each loop.

The [updated diagram](overview.md) and [public runtime contract](runtime.md) describe interfaces and retained limits. `Native::new` no longer takes a profile path; `Native::save` is replaced by Runtime/Store saving. Runtime callback save and Native poll convenience remain available.

## Validation

- Workspace tests: **231 passed, 7 opt-in tests ignored**. Includes profile/checkpoint parity, old snapshot compatibility, clock resume policies, pending music resume, storage failure ordering, Send rollback, receive acknowledgment, old document rejection, generation binding, read-only painting and shared close behavior.
- Workspace Clippy with warnings denied, formatting, headless library check and asset SHA-256 checks passed.
- External no-window example implements the new Platform/Documents/Store APIs, renders four frames and restores a synthetic message without network or audio output.
- Compared **282 CPU frame PNGs** with baseline: **281 byte-identical**. The only other frame is the live-clock Home fixture; all differences are confined to clock glyphs (x=94..170, y=53..82). Pixels outside the clock region are identical. No scenes were added or removed.
- Final desktop smoke run: **127 frames presented through Metal**; sound, music, video and network disabled, temporary data directory only. This is presentation evidence, not a GPU readback comparison.
- Release-tool tests: six passed. No remote CI monitoring or Windows/Linux native execution was performed in this refactor.

## Repeat

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check --locked -p phone-desktop --no-default-features --lib
cargo run --locked -p phone-desktop --no-default-features --example embed -- /tmp/ta1278-embed
sh scripts/frames.sh /tmp/ta1278-frames
python3 -S scripts/check-assets.py
python3 scripts/tests.py
cargo run --locked -p phone-desktop -- --smoke-test /tmp/ta1278-window --disable sounds,music,video,network
```

Use new temporary output directories. Real audio, optional external-network checks and the video fixture test stay opt-in. The live-clock fixture should be compared with its clock region excluded, or both baselines must use the same supplied time.

## Application-side coordination follow-up

The local follow-up merges effect integration and persistence policy under
`runtime/app`. Runtime retains one App coordinator instead of separate Host,
Session and Services fields. `app/bridge.rs` handles web/media/message integration;
`app/persist.rs` handles record saves and close policy; `app/state.rs` contains
the unchanged checkpoint implementation. Store/Record/Directory are also exported
from `platform`; existing persistence paths remain the same types. No public
Runtime method, JSON format, dependency or interaction changes.

Verification: 231 workspace tests passed, seven opt-in tests ignored; workspace
Clippy with warnings denied, formatting and headless library checks passed. The
external example uses the common platform Store import, renders four frames and
restores a synthetic message. All 282 CPU frames match the preceding verified
implementation except the live clock (281 byte-identical; the remaining frame's
changes are confined to x=94..170, y=53..82). This follow-up did not perform a new
native GPU or physical-device acceptance run. No network/audio output was needed.

## Shell and explicit application boundaries follow-up

The internal Phone type/module and Runtime field are now Shell/shell. Public
Runtime methods, Cargo package names, config keys and persisted schemas remain.
The prior application-side coordinator merge remains in place.

- Painting borrows Overlay (including text editors); common popup menus and
  contact root/detail menus use static typed Item declarations for labels,
  availability, markers and actions. Production menu labels allocate no strings.
- Contacts owns its Page, draft, selected identity and recipient-picker state.
  Its field initialization, synchronization, validation and record commit operate
  on contacts::State. Calendar/alarms no longer share its draft/return focus.
- Services only receives message state and returns Apply results; Shell owns
  conversation selection and alert presentation. Durable send/ack order remains.
- Sound receives immutable Playback intent and returns PlaybackStatus. The app
  handles duration/position, EOF repeat/queue policy and errors. Custom Sound
  implementations must adopt this port signature; Runtime entry points stay stable.
- VisualMedia receives only media state/context and returns explicit effects.
  The small Media aggregator still coordinates alarms, playback, files and UI.
  Library scans and file audio probes use the supplied Decoder::audio_duration,
  with the built-in MP3/WAV behavior retained.

Local validation: 236 workspace tests passed, seven opt-in tests ignored;
Clippy with warnings denied, formatting, headless library and external embedding
example passed. The example renders four frames and restores a synthetic message
without audio/network. Of 282 CPU PNGs, 281 are byte-identical to the preceding
verified implementation; the remaining difference is only the live Home clock.
The alarm-tone fixture now explicitly seeds its unselected built-in tone rather
than depending on an earlier contact draft; its final pixels match the baseline.
No new dependencies were added. Native audio/video device tests remain opt-in.

Final native window check: 127 frames presented through Metal using temporary
data with sounds/music/video/network disabled. This verifies presentation, not
new physical-device or Windows/Linux native acceptance. Changes remain local.
