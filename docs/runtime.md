# User-space runtime

The phone is a library with a desktop host. It does not provide a kernel or own
its host's main loop. App rules remain phone rules; external content and service
implementations stay outside the UI.

The existing `phone-desktop` Cargo package now exports the `phone_runtime`
library. Keeping the package name preserves current build/package commands.
The public entry points are `Runtime`, `Time`, `Frame`, `Request`, `config`,
`messaging`, `platform` and `persistence`. `take_cues` exposes bounded short sound effects when a custom host handles them. Internal `Shell` fields are not a public integration API.

## Use without a window

```toml
[dependencies]
phone-runtime = { package = "phone-desktop", path = "../ta-1278/crates/phone-desktop", default-features = false }
```

```rust
use phone_runtime::{Runtime, Time, config::RuntimeConfig, input::Key};

let mut phone = Runtime::new(RuntimeConfig::default(), Time {
    elapsed_ms: 0,
    unix_ms: 1_793_000_000_000,
})?;
phone.press(1, Key::Center);
phone.release(1);
phone.advance(Time { elapsed_ms: 16, unix_ms: 1_793_000_000_016 })?;
let frame = phone.render();
// Upload/copy frame.pixels to the host's texture: 240x320 RGBA8, stride 960.
# Ok::<(), String>(())
```

Rendering does not advance time. `advance` handles held keys, editors, timers,
alarms and animation. The elapsed clock must not go backwards. The wall clock
controls the display, new message timestamps and alert expiry. Existing phone
clock/timezone settings apply on top of it. To pause the model, cancel held input
and stop advancing the clocks; resume using virtual elapsed time that excludes
the pause. The host separately decides whether to keep polling incoming services. Painting is read-only; input measures derived geometry separately without repainting pixels.
Native media playback has its own clock; freezing `Time` does not pause an audio
sink or video helper.

`press`/`release` use host-assigned source IDs, so multiple input sources can map
to the same phone key. `cancel_input` releases them without confirming a press.
`text` accepts ASCII keyboard text only; phone-key events still use the internal
IME. `#` remains the mode key. The library opens no window, profile, folder picker,
network connection, media worker or audio output when constructed.

Run the external-API example:

```sh
cargo run -p phone-desktop --no-default-features --locked --example embed -- /tmp/ta1278-embed
```

It writes four PNGs from synthetic content, sends key events, supplies a local
browser document, receives a synthetic message and saves/restores in memory.
It creates no window and uses no network/audio adapter. `--no-default-features`
excludes winit, wgpu, pollster and the Swift-helper build. Image/font/input codecs
and the existing native media/HTTP adapter dependencies are still compiled;
this is not a minimal-dependency or bare-metal build.

## Host services and documents

Construct with `Runtime::with_backend(config, time, Box<dyn messaging::Backend>)`
to provide message/alert events. Call transport remains reserved; the local dialer does not dispatch call commands.
Call `poll_services` from the host loop. A `pending_commit` means that the mailbox
must be saved before outbound submission or acknowledgment can proceed. The
[existing ordering and replay contract](services.md) still applies. Do not create
another message store alongside this one.

`navigate` and ordinary browser key navigation produce a `Request` through
`take_request`. The host can parse HTML with `platform::parse` and return a
`Page` through `complete`, or return an error. Each request is bound to its
runtime and generation. Responses after cancel/replacement, or from another
runtime, cannot overwrite the active document. Host-provided pages use the same
layout, links, fields and forms as the standalone browser. Do not mutate request
URLs after dispatch. Content providers are responsible for bounded inputs.

`platform::Native` implements `Platform`/`Documents` and supplies the shared `Sound` and decoder/session ports without a winit window. Custom hosts can supply the same ports. Drive them with `Runtime::poll_platform`; Native::poll remains a compatibility wrapper. Its directory is a direct filesystem mapping. Constructing
it is explicit, polling may perform requested IO/playback, and configuration must
match the capabilities the host intends to provide. Runtime construction does not open `config.storage.directory`; pass the directory/read-only choice explicitly to Native. It is bound to one runtime;
create a fresh adapter for a restored instance. It owns no persistence or application state. Use `Runtime::flush` with a Store to retain the existing music preference file. macOS image/video support requires the separately built
media helper beside the host executable; the no-desktop build does not build it.
No new VFS, plugin loader or abstract hardware-driver framework was introduced.

## State save contract

`Runtime::save` serializes a versioned JSON bundle and passes its bytes to the
host's callback. The callback must atomically commit these bytes together with
any related host state before returning `Ok(())`. On failure, pending Sends roll
back to editable drafts; receive acknowledgments stay blocked. Bytes are limited
to 32 MiB. Config and media files are not embedded in the bundle.

```rust,ignore
phone.poll_services()?;
if phone.pending_commit() {
    phone.save(|bytes| host.commit_phone_state(bytes))?;
}
// Also save at host-selected checkpoints for settings, drafts and utility state.
```

Saved data includes the mailbox and service cursors, confirmed SMS draft text,
settings, contacts/calendar/alarms/reminders/playlists, browser bookmarks/history/
offline documents, Snake settings/scores, timer/stopwatch data and native input
learning. Unconfirmed input candidates block saving. File mutations in progress
also block it. A failed callback leaves the live instance available for retry.

`Runtime::restore(config, time, bytes, backend)` creates a **new** instance at Home.
It preserves saved timer remaining time without adding time spent outside the
host. A confirmed open SMS draft returns to the draft list. It does not restore
an arbitrary UI frame, caret/candidate list, unfinished organizer form, live Snake
round, current web request, music/video decoder position or external file changes.
This is a user-data checkpoint, not a process snapshot. The standalone host keeps its separate JSON layout and wall-clock catch-up policy through the same Store and state capture/restore engine. `Resume::Paused` and `Resume::WallClock` select the timer policy explicitly; profiles are not silently migrated to a bundle.

The supplied backend must agree with the host's restored history. Real-world
messages already sent cannot be undone. Restoring a saved request counter into
an existing provider namespace still requires external reconciliation/deduplication;
new runtime request IDs only isolate local browser completions.

## Current boundary

This first step provides the shared execution/input/frame entry point, a supplied
clock, message and document ports, optional native adapters and user-data saves.
The desktop window now uses the same Runtime and Native objects. It retains native window behavior; `Runtime::close` now owns draft confirmation and failed-save retention. `persistence::Store` owns I/O while Runtime owns record/checkpoint serialization and dirty acknowledgments. Full transient UI snapshots,
media pause/restore coordination, and coordinating external backend rewind remain separate follow-up work. Media ports are now replaceable; the file capability intentionally remains a direct host folder. There is no new actual messaging
backend, device access or claim of complete firmware recovery.

## Application-side coordination

The public Runtime entry points share one internal `runtime/app` coordinator for
web/media/message integration and save/restore/close policy. Serialization,
dirty acknowledgments and retry/rollback decisions are application concerns;
platform implementations only provide I/O mechanisms. No second system-level
persistence manager is introduced. Store remains separately injectable so hosts
can commit a phone checkpoint and their own state in one transaction.

## Platform and storage ports

`Platform::instance` returns `Runtime::instance()`. `Documents` exchanges opaque Requests and Pages, `Sound` handles audio/control clock synchronization, and `Decoder` returns pixels or a `Video` session. `Devices` transfers a selected real folder, Sound and Decoder once, on the first platform poll. Runtime owns the transferred media lifetime; create a fresh runtime/adapter when changing hosts. No port receives Shell or application UI fields.

`platform::{Store, Record, Directory}` provides the common port import path; the existing `persistence::{Store, Record, Directory}` paths remain aliases of the same types. Store reads/writes bounded Record keys. `Directory::open` owns the profile lock, atomic writes and backups; `Directory::new` is for callers already owning the transaction/lock.

- `Runtime::load_profile(config, time, store, backend, resume)` and `flush(store, force)` use the existing JSON filenames. Background flush does not confirm or save uncommitted draft text.
- `save_to(store)` and `restore_from(config, time, store, backend)` use Record::Snapshot. Old version-1 snapshots without music remain readable; new snapshots also retain paused music resume state.
- Callback `save` and byte-based `restore` use the same capture/validation code. Choose a profile or snapshot layout consistently when restoring; writing a snapshot does not update all profile files.
- The first record failure stops subsequent writes. Earlier successes remain acknowledged; an unattempted mailbox stays pending. Only an attempted failed mailbox write rolls back a queued Send.

See [the current architecture diagram](overview.md).

The native constructor no longer takes a profile path; use `Native::new(&runtime, directory, read_only)` or `Native::from_folder(&runtime, folder)`. `Native::save` has been replaced by Runtime/Store persistence. The callback Runtime::save API and Native::poll convenience remain available.

## Playback observations

Sound adapters now implement `sync(files, path, Playback) -> PlaybackStatus`.
Playback is a small immutable intent with play/pause, volume, position, duration,
generation and seek revision. Status reports optional measured duration/position,
EOF, stop and error. The application applies it to its player model; adapters
never edit queues, repeat mode or navigation. EOF transitions remain application
policy. Errors pause playback while retaining the position; invalid non-finite
or negative clocks are rejected. This replaces the former mutable Music argument
for custom Sound implementations; Runtime entry points remain unchanged.

Decoder implementations may override `audio_duration` for library scans/file
playback. Its default reports unsupported audio decoding; the built-in decoder
supports MP3/WAV/MIDI playback. Visual requests return pixels or video sessions; the
application bridge returns explicit UI effects rather than editing Shell.

## Local backend ownership

Backend means an owner of external state, not necessarily a network service.
`Runtime::with_backend` accepts a Rust `Backend` directly; the trait does not
require a socket, server or `Send`. An embedding host may share local backend
state/debug controls with its adapter (see the `Rc<RefCell<_>>` test adapters),
or provide a worker/network implementation when needed. World/scenario state
and its persistence remain with that backend. Runtime only owns handset state.
Calls into an adapter must return promptly even in the same process.

The Android/desktop hosts share input, keypad rendering and the winit loop.
`Runtime::cancel_source` cancels one pointer without releasing another finger's
key. `suspend_media` cancels held input and pauses/stops media and notification
audio before host suspension. Android drops its GPU surface synchronously on
suspend and recreates it on resume, while keeping Runtime/backend state alive.
