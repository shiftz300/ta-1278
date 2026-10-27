# User-space runtime

The phone is a library with a desktop host. It does not provide a kernel or own
its host's main loop. App rules remain phone rules; external content and service
implementations stay outside the UI.

The existing `phone-desktop` Cargo package now exports the `phone_runtime`
library. Keeping the package name preserves current build/package commands.
The public entry points are `Runtime`, `Time`, `Frame`, `Request`, `config`,
`messaging` and `platform`. `take_cues` exposes bounded short sound effects when a custom host handles them. Internal `Phone` fields are not a public integration API.

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
the pause. The host separately decides whether to keep polling incoming services.
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
to provide message/alert events. Calls remain reserved and their UI stays disabled.
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

`platform::Native` is the existing concrete folder/audio/HTTP adapter, now usable
without a winit window. Its directory is a direct filesystem mapping. Constructing
it is explicit, polling may perform requested IO/playback, and configuration must
match the capabilities the host intends to provide. Runtime construction does not open `config.storage.directory`; pass the directory/read-only choice explicitly to Native. It is bound to one runtime;
create a fresh adapter for a restored instance. Its `save` retains the existing
music preference file. macOS image/video support requires the separately built
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
This is a user-data checkpoint, not a process snapshot. The standalone host keeps
its existing separate JSON files and wall-clock catch-up policy; it does not
silently migrate existing profiles to this bundle.

The supplied backend must agree with the host's restored history. Real-world
messages already sent cannot be undone. Restoring a saved request counter into
an existing provider namespace still requires external reconciliation/deduplication;
new runtime request IDs only isolate local browser completions.

## Current boundary

This first step provides the shared execution/input/frame entry point, a supplied
clock, message and document ports, optional native adapters and user-data saves.
The desktop window now uses the same Runtime and Native objects. It retains native
window/quit behavior and legacy profile persistence. Full transient UI snapshots,
media pause/restore coordination, and replacing every media service with a custom
host implementation are separate follow-up work. There is no new actual messaging
backend, device access or claim of complete firmware recovery.
