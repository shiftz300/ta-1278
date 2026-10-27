# Phone storage

The desktop uses one local folder directly through `std::fs`. There is no VFS crate, mount table, `Volume` trait, memory-card slot, disk image or import library.

## First launch

On macOS, first launch opens a folder chooser. Choose an existing folder or create one in the chooser. Cancel exits without saving a selection. The app remembers its absolute location in `storage.json` under the private profile. Later launches reuse it. `--storage-dir DIRECTORY` explicitly selects or changes the folder and saves the choice. Other desktop platforms require this flag for first setup.

Only **手机存储** appears on the Files start page. Opening it shows the selected folder's contents. Music, pictures, video, wallpaper and ringtone selection all use that same folder. File operations directly affect the selected files. There is no import or copy-on-import step.

Private JSON records remain in the profile (`~/Library/Application Support/TA1278 Replica/` by default on macOS), outside the browsable folder. A folder containing the private profile is rejected. Back up the profile and selected folder separately. The app does not upload either.

[Unified runtime config](config.md) can also set the directory, read-only mode, capabilities and optional apps.

## Controls and flags

- `--storage-dir DIRECTORY`: choose the single folder; it must exist.
- `--read-only-storage`: disallow create, rename, delete, copy and move. Messages and settings can still save.
- `--data-dir PATH`: independent private profile, including its own folder choice.
- `--disable music,files,pictures,video,sounds,network`: existing runtime feature controls.
- `--mount-card` and `--no-card` have been removed.

An unavailable saved folder stops startup with an error instead of silently creating empty storage. Reconnect the drive or explicitly select its new location with `--storage-dir`. Existing `files/` and `card/` folders are never moved or deleted. Old volume-prefixed playlist/picture references are retained but are not automatically rebound; choose media again or rebuild playlists after selecting storage.

## Implementation

- `storage/setup.rs`: first-launch choice, validation and atomic configuration save.
- `storage/files.rs`: one concrete folder; direct file streams, listing, permissions and bounded copy/move. No replaceable filesystem backends.
- `media.rs` and `media/files.rs`: background music scan and file operations.
- `phone/files.rs` and `views/files.rs`: navigation, confirmations and native-shaped storage overview.
- `audio.rs`: MP3/WAV decoding from `std::fs::File`.
- `media/visual.rs`: bounded temporary copies for the existing macOS image/video helper; these are decoder inputs, not an imported media library.

Internal saved media names are slash-prefixed paths relative to the chosen folder (`/Music/song.wav`), not host absolute paths. Root writes/removal, traversal, symlinks and special files are rejected. Names must be UTF-8. This is a trusted local-folder interface, not an adversarial OS sandbox; concurrent host path replacement remains outside its guarantee.

Copy is bounded to 10,000 visible entries, depth 16 and 512 MiB. Failed copies attempt to remove their newly created files. Existing destinations are never overwritten. Move uses filesystem rename; crossing host filesystem boundaries fails without deleting the source. Folder deletion is nonrecursive. Successful rename/move updates media references. Music refresh rescans external edits; physical hot-unplug recovery remains limited.

## File page behavior

The overview shows one storage row. Capacity is taken from the filesystem containing the selected folder, including usage outside that folder. Format is grey and inactive. Multi-selection checks all selected paths (up to 1,000) before deletion; directories must be empty. A preflight error changes nothing. Concurrent external changes can still cause a partial result, which reports successes/failures and retains failed selections. Background work survives returning Home.

## Verification

The workspace tests cover chooser cancellation/reuse, missing storage, profile isolation, read-only operations, traversal and link rejection, no-overwrite copies, subtree-copy refusal, nonrecursive deletion, file-to-player return, failed-navigation retention and media persistence. `--smoke-test` uses synthetic pages without opening a folder picker or reading user media.
