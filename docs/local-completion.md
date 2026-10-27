# Local applications

These are local application flows, not a claim of complete handset equivalence.

| Area | Behavior and limits |
|---|---|
| Contacts | Pictures, groups, local ringtones and editable fields; deleting a group keeps contacts. |
| Music | MP3/WAV playback, playlists, repeat/shuffle, track selection and saved paused resume position. Missing files are omitted from visible lists without erasing stored paths. |
| Images | Browse the selected folder, open images, zoom/pan and select wallpaper or contact pictures. macOS ImageIO supplies decoding; only the first image/frame is shown. |
| Video | Local files through AVFoundation; play/pause, seek and volume. Available formats depend on system codecs. |
| Files | Direct folder access, rename, copy/move, create folders, details and confirmed nonrecursive deletion. No overwrite or VFS. See [storage](storage.md). |
| Settings | Greeting, wallpaper, colors, shortcuts, sound, clock display and browser access. No modem, SIM or Bluetooth configuration. |
| Calendar and alarms | Recurring events and reminders while running; no OS wake scheduler. |
| Tools | Calculator, converter, timer, stopwatch and Snake. Radio, flashlight and recorder have local UI state only; no hardware recording or radio reception. |

Pages share title, lists, popups, editors and softkeys. File, image, wallpaper and ringtone selection share one folder browser. There is no import button.

## Media implementation

`phone/media.rs` owns selection and transport state; `views/media.rs` draws it. `media/visual.rs` owns decoding jobs, preview cache and helper lifecycle. `native/MediaHelper.swift` uses installed macOS frameworks and receives bounded temporary copies of local files, never an external URL. Image inputs are limited to 16 MiB, videos to 512 MiB. Video output is capped at ten 240×240 RGBA frames per second. Only one video process runs at a time. Preview images are limited to 240×320, with 32 cache entries.

`audio/native.rs` binds retained sound resources. `audio/midi.rs` uses independent SF2 synthesis; exact original DSP output is not claimed. Silent mode and volume apply. Temporary decoder files are removed on normal completion; a forced termination can leave temporary files.

## Boundaries

Calls remain disabled. Messages use the local mailbox and [service interface](services.md), with no real transport. The [browser](browser-capabilities.md) handles basic text pages and GET forms. The [input engine](input-engine.md) retains T9 resources and private learning. [Runtime](runtime.md) describes embedding and save/restore limits.
