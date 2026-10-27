# Local app completion pass

2026-10-26. This pass fills working local app flows. It does **not** establish full handset or firmware equivalence. No handset command, firmware write or personal-data capture was performed.

This is a historical implementation report. Its VFS/mount descriptions were superseded on 2026-10-27 by [direct folder storage](storage.md). The [current native-page audit](native-page-audit.md) records unresolved fidelity gaps; the earlier local-flow checks do not establish full reconstruction.

## Added

| Area | Working behavior | Evidence boundary |
|---|---|---|
| Contacts | Choose a picture from the VFS, keep the form draft, assign groups, create/rename/delete groups, filter members, choose and preview a local ringtone. Group deletion keeps contacts. | The existing nine-field native form and measured geometry are retained. New chooser/group-management flows are local policies. |
| Music | Create/rename/delete playlists, add/remove songs, play their ordered queue, keep queue/repeat/shuffle and resume paths across restart, set a song as the ringtone. | Real MP3/WAV transport; playlist behavior is a local implementation. Missing files are omitted from the visible playlist without erasing stored paths. |
| Images | Browse mounted folders, open PNG/JPEG/GIF/BMP/HEIC, zoom and pan, choose contact pictures or wallpaper. | macOS ImageIO; first image/frame only. JPEG and PNG were exercised. These are not recovered handset codecs. |
| Video | Open local MP4/M4V/MOV/3GP files with the system decoder, show frames in the phone window, play/pause, seek with Left/Right and change volume with Up/Down. | H.264/AAC MP4 was tested. Other containers/codecs depend on AVFoundation and may fail explicitly. Rotation metadata and complete native player layout are not verified. |
| Files | Copy/move a visible file or tree within/between mounts, rename, create folders, inspect and confirm deletion. Renames/moves update playlist, picture and ringtone references. | No overwrite. Copies and cross-volume moves are limited to 10,000 entries, depth 16 and 512 MiB. Same-volume moves use rename. Failed copies attempt to remove newly created entries; inaccessible storage can prevent cleanup. Source cleanup after a move can fail separately. Folder deletion remains nonrecursive. |
| Settings | Greeting, wallpaper, theme, shortcuts, silent mode, ringtone/volume, key feedback, local date/time/timezone/12-hour display, and a gate for new browser requests. Reset restores preferences only. | These settings have local effects. They do not configure a real modem, Bluetooth or SIM. |
| Calendar | Daily/weekly/monthly/yearly events, day markers, reminder choices, a ringtone picker and reminders crossing midnight. | Local scheduling while the app runs. Dates before the first occurrence do not recur; absent monthly dates and non-leap February 29 are skipped. |
| Feedback | Audible Snake food/death cues, optional key tones, message feedback and local alarm/reminder output. Silent mode and volume apply. | Original Snake MIDI and the recovered SF2 bank now drive independent synthesis; native preset/click resources are available. Some preset-name/event bindings and exact DSP output remain unproven. See [native audio](native-audio.md). |

All these pages use the existing title, list, popup, form/editor and softkey controls. Pictures, wallpaper and ringtone selection share one VFS browser. The app has no import button.

## Implementation

- `phone/media_browser.rs`: chooser state, navigation, zoom, transport keys and return to the calling form.
- `views/media_browser.rs`: phone-sized media presentation through shared controls.
- `visual_media.rs`: bounded VFS copies, decoding jobs, preview cache and native video process lifecycle.
- `native/MediaHelper.swift`: our source, compiled with the installed Apple SDK. Uses ImageIO and AVFoundation; accepts local files only. It is bundled and signed with the app.
- `phone/music.rs`: player and playlist interaction; `phone-core/music.rs`: queue/end/repeat rules.
- `phone-vfs`: stream access, mount permissions, bounded copy/move and rollback.
- `media/files.rs` and `phone/contact_groups.rs`: separate file I/O/reference updates and group interaction.
- `effects.rs` / `native_audio.rs` / `midi_synth.rs` / `audio.rs`: original audio resources, independent SF2 synthesis and output, separate from UI state. See [native audio](native-audio.md).

The helper receives a private temporary copy of the selected file, with a size cap of 16 MiB for images or 512 MiB for video. It never receives an external URL. Copies are removed on normal completion/drop; a forced process termination may leave a temporary file. Video output is capped at ten 240×240 RGBA frames per second. Only one video process runs at a time. Preview cache images are reduced to at most 240×320, with 32 entries. Drawing scaled images clips and samples the source directly without allocating a resized image every frame.

## Additional code evidence

`scripts/recover-snake-random.py` checks both firmware hashes and follows:

1. Food generation makes **two** calls at `0x886902dc` and `0x886902ea`.
2. The ARM veneer at `0x88100a4c` jumps to Thumb `0x8165ab59`.
3. Kernel code `0x8165ab58..0x8165ab86` adds two words in a **55-word ring**, stores the full uint32 sum, advances/wraps both pointers, and clears bit 31 in the return value.
4. Coordinates are `draw1 % 12`, `draw2 % 14 + 2`.

The replica now uses this ring update and two-draw mapping. Its initial words and pointer separation remain a deterministic local policy. The native initial RAM table, seeding and draws consumed by other apps are unknown. Sequence equivalence is **not** claimed. See [verified bytes and call chain](evidence/snake-random.json) and [bounded listings](evidence/snake-random.txt).

## Explicit limits

- Calls stay grey and inert. SMS uses local incoming records and an unsent queue, with no modem backend.
- Browser scope remains text, links, basic GET forms and saved pages; it is not the original browser engine.
- Original prediction internals, font hinting, all native error/interrupt branches, exact status/lock behavior and every scroll edge are not recovered.
- Snake boot randomness, exact scheduler timing, exact native DSP waveforms, unsampled end-state transitions and overflow/full-board parity are not established.
- Bluetooth, SIM PIN/STK, USB device mode, LED/FM and microphone services have no host backend. Recorder timing is not an audio recording. Speed dial stays disabled with calling.
- Alarms/reminders require the app to be running. There is no system wake service. Pending alerts and snoozes now survive quitting; overdue snoozes appear on restart. Scheduled occurrences entirely missed while closed are not replayed. See [persistence](persistence.md). Music/video pause for an alert and require explicit resume.
- The UI is Chinese. Additional system-language UI and unsupported native security settings are not implemented.
- This pass does not add original native codecs, animated GIF playback, tags/album art, FAT/disk-image backends or physical hot-unplug recovery.

These are declared implementation or evidence boundaries, not silently completed features. This is a working local implementation within the stated limits. Complete firmware reverse engineering and complete handset comparison are not established.

See [validation](validation.md), [current fidelity](fidelity.md), [storage](storage.md) and [delivery scope](remaining-work.md).

## Reproduce checks

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 scripts/recover-snake-random.py
sh scripts/package-macos.sh
target/TA1278.app/Contents/MacOS/phone-desktop --smoke-test /tmp/ta1278-screens
```

The generated video used `ffmpeg -f lavfi -i testsrc2=size=240x180:rate=10 -f lavfi -i sine=frequency=440:sample_rate=16000 -t 6 -c:v libx264 -pix_fmt yuv420p -c:a aac clip.mp4`. Run `TA1278_VIDEO_FIXTURE=/absolute/path/clip.mp4 cargo test -p phone-desktop --locked native_video_frames_pause_seek_and_cleanup -- --ignored --nocapture` in a normal macOS session. The separate `real_output_pause_seek_and_eof` test generates a WAV, or accepts `TA1278_AUDIO_FIXTURE` for MP3. These tests do not use the handset.

The current distributable is about 18.30 MiB after resource packing, font deduplication, the owner-selected wallpaper and replacement of the older dictionaries with the original T9 engine data. Library paths are indexed once per scan/update, and visible music rows borrow track titles. This avoids per-frame full-library title copies and repeated linear path searches. [Size and image comparison](build-size.md).

## Input and browser follow-up

Shared prediction, bounded persistent selection/context learning, password learning exclusion and an engine replacement interface are now implemented. The subsequent runtime pass integrates the original ARM candidate engine, spelling/page selection, context associations and native dictionary persistence through bounded interpretation. Complete semantic decompilation and exhaustive handset comparison are not claimed. [Algorithm evidence](t9-recovery.md) and [engine contract](input-engine.md).

Browser Clear now toggles history/cache and commits locally. Cache clearing retains the visible page and saved pages/bookmarks. Sharing a URL opens a new SMS draft and preserves unfinished text without sending or duplicating an already queued message. These are usable host policies, not newly established native branches.

The Home clock and date were adjusted to the stored native reference `ta1278-native-187/home-left-softkey-before.png`: clock top 42 at size 40; date top 96 at size 14 with weekday and numeric date. This does not reproduce native font rasterization, status icons or emergency-call overlays.
