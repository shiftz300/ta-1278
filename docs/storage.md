# Phone storage

The app uses a **VFS inside the process**. It does not mount a disk in macOS, use FUSE, or import/copy media. The selected directory is a volume; file browsing and music share it. This replaces the earlier import-copy proposal at the owner's request.

| Virtual path | Default backend |
|---|---|
| `/phone` | `<data-dir>/files/` |
| `/card` | `<data-dir>/card/`, or the existing directory passed to `--mount-card` |

Private app records (messages, contacts, settings, browser data and music settings) remain beside these folders, outside the browsable volumes. Existing records are preserved. An app-owned default data directory can be backed up as a whole. An external card directory must be backed up separately.

## Flags

| Flag | Effect |
|---|---|
| `--mount-card DIRECTORY` | Mount an existing directory directly at `/card`. No copy. Writes affect that directory. |
| `--no-card` | Do not mount `/card`. Conflicts with `--mount-card`. |
| `--read-only-storage` | Block mkdir, rename and delete in both media volumes. Does not disable private app-record saves. |
| `--disable music` | Block music launch and file-to-player dispatch; no scanning or audio output. |
| `--disable files` | Block file-manager launch and all file-manager service requests. Music may still read the volume. |
| `--disable network` | Reject external browser requests before dispatch. Local demo and saved pages remain usable. |

Combine disabled features with commas. These are runtime capabilities, not native phone settings. They keep the sampled menu order and reject unavailable actions instead of rearranging the menu. They do not remove compiled dependencies.

## Modules

- `phone-vfs`: mount table, `Volume` backend interface, seekable streams, virtual path validation, per-mount write permission and unmount API.
- `phone-vfs/directory.rs`: local directory implementation. Does not return host paths to consumers.
- `media.rs`: shared VFS service, bounded background music scan, file operations and playback settings persistence.
- `audio.rs`: Rodio/Symphonia decoding and real output. Uses stream length and seek support from the VFS.
- `phone/files.rs`, `views/files.rs`: file-manager state and rendering, separate from filesystem operations.

Another backend can implement `Volume` and be mounted without changing the player. **Only directory backends are shipped.** There is no FAT image, NAND, OS disk or USB mass-storage emulation. The mount table supports unmounting; desktop live insertion/ejection controls are not yet provided.

## Files and playback

Files can browse folders, inspect size, create a folder, rename without overwrite and delete with confirmation. A failed folder read leaves the path, selection and Back history intact; leaving Files cancels pending file navigation. Deleting a nonempty directory fails; no recursive erase. Opening MP3/WAV enters the music player; Back returns to the originating folder. Unsupported formats report an error. Folder menus/geometry are conservative local implementations; their native fidelity is not claimed.

Music scans the volumes on startup and with Music → menu → 手动刷新. External edits need refresh. A refresh pauses playback immediately and preserves the same virtual path/position when available. Playback requests wait for the scan. A file mutation invalidates older scan results; folder renames update the saved virtual path, and deletion clears it. Volume/repeat/shuffle still save if the app closes during a scan. The scan has a 10000-entry and 16-level bound, skips hidden entries, and reports invalid files. Files are streamed, not loaded wholly into memory. Track names currently come from filenames. MP3 and PCM WAV are enabled; other codecs, playlist editing and cover art remain unimplemented.

Center pauses/resumes; Up/Down changes volume; a short Left/Right changes track; a held Left/Right seeks. Repeat and shuffle follow the existing control model. Elapsed time comes from the audio sink, including outside the player. Volume, repeat, shuffle, virtual path and position save periodically and on normal close. Restart restores a paused track. Music-save failure blocks normal close. Audio initialization/seek errors stop playback visibly.

Directory backends reject `..`, empty path segments, links and special files. Names must be UTF-8. Only mounted roots are visible. This adapter is for trusted user-selected local folders, not an adversarial OS sandbox: concurrent host replacement between validation and I/O is not prevented. Mounting a real external volume by its directory does not provide robust physical hot-unplug detection; use a stable local directory for now.

See [validation evidence](evidence/storage-audio-validation.json) for the tested build and fixture hashes.

## Verification

`cargo test --workspace --locked` covers virtual roots, read-only mutations, traversal/link rejection, no-overwrite rename, nonempty-folder refusal, scanning/decoding errors, file navigation/delete cancel, file-to-player return, disabled services and paused restart.

An explicit hardware test opens the real CoreAudio stream with a generated silent WAV and checks advancing time, pause, seek and EOF:

```sh
cargo test --locked -p phone-desktop real_output_pause_seek_and_eof -- --ignored --nocapture
```

The same test accepts `TA1278_AUDIO_FIXTURE=/absolute/path/to/generated.mp3`. Both WAV and a locally generated MP3 passed on this Mac outside the sandbox. This proves output-stream operation, not a listening-quality assessment or complete native-player fidelity. No private media or phone data was used.

Primary API references: [Rodio sink](https://docs.rs/rodio/0.21.1/rodio/struct.Sink.html), [decoder builder](https://docs.rs/rodio/0.21.1/rodio/decoder/struct.DecoderBuilder.html). Versions and checksums are pinned in Cargo.lock.
