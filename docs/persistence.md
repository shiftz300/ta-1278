# Saving and recovery

2026-10-26. Local persistence update after release 0.2.0. These are desktop policies; no new handset behavior is claimed.

## Records

All records live in the app data directory, outside the VFS volumes. Use `--data-dir PATH` for an independent profile. The default macOS location is `~/Library/Application Support/TA1278 Replica/`.

| File | Saved data |
|---|---|
| `messages.json` | Messages, read state, saved drafts and the local outgoing queue |
| `preferences.json` | Theme, shortcuts and supported system settings |
| `browser.json` | Bookmarks, history, offline pages and browser settings |
| `organizer.json` | Contacts/groups, playlists, events, alarms, pending alerts and snoozes |
| `music.json` | Playback options, queue, virtual path and paused resume position |
| `utilities.json` | Timer, stopwatch/laps, radio frequency, favorites and speaker preference |
| `snake.json` | Levels, scores and settings |

Organizer and utility records use schema version 1. Older organizer files without the version or reminder fields load with defaults. Unsupported future versions and invalid state fail explicitly, preserving the original file. Other stores keep their existing formats; there is no global database migration.

## Restart behavior

- A fired one-shot alarm's disabled flag and its pending alert save together in `organizer.json`. Stopping or snoozing the alert updates that same record.
- Pending alerts and overdue snoozes appear after restart. A persisted trigger marker prevents ordinary alarm/event duplicates on another launch within the same minute.
- Running timers and stopwatches include positive wall-clock time since their saved checkpoint. Paused clocks do not advance. A clock change backwards adds no offline time. Stopwatch laps and timer labels are retained.
- An expired timer remains pending until acknowledged, including across restart. It waits while an editor or another overlay is active rather than replacing that overlay. Reopening an active or paused timer restores its controls.
- Utility actions save on the next flush. Running clocks checkpoint at most once every five seconds and on normal exit. Unchanged records are not rewritten. Music retains its existing periodic and exit saves.
- Failed writes retain dirty state for retry. Normal close stays open if saving fails. This does not guarantee preservation of every last action after a crash: an unflushed action or unsaved editor text may still be lost.

There is no background process or OS wake scheduler. Alarms and calendar occurrences that are entirely missed while the app is closed are not replayed; pending alerts, snoozes and timer expiration are handled on launch. Recorder timing, flashlight state, calculator input and open UI pages remain temporary. Radio preferences do not add an FM backend.

## Writes, backups and recovery

`storage/atomic.rs` bounds reads, writes a uniquely created temporary file, syncs it, and renames it over the destination. It keeps one previous JSON file as `NAME.json.bak`. A failed backup prevents replacing the current record. A leftover temporary file is ignored. Each file is atomic on its own; separate stores and VFS mutations are not one transaction.

An OS file lock on `.instance.lock` prevents two updated app instances from using one profile. The lock releases on exit or crash; the pathname can remain. Older builds do not honor this lock, so do not run them against the same profile simultaneously.

Malformed or oversized files are retained and reported. Backups are not loaded automatically: that could silently restore deleted messages or older settings. To recover, close the app, copy the whole data directory somewhere safe, retain the damaged file under a different name, then copy the matching `.bak` to the original filename. Relaunch to validate it. A backup may also be invalid and is only one previous version, not a full history. Backups may contain deleted messages and other private records; protect them like the main files.

Back up `files/` and `card/` with the profile when using default volumes. Back up an external `--mount-card` directory separately. The app does not upload these records.

## Verification

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
sh scripts/package-macos.sh
target/TA1278.app/Contents/MacOS/phone-desktop --smoke-test /tmp/ta1278-persistence-smoke
```

`storage/tests.rs` covers old-schema loading, future-version refusal, restart across midnight, same-minute duplicate suppression, running/paused clocks, pending timer acknowledgment, editor retention, backup failure, corrupt-file protection, lock release and failed-save retry. Existing SMS transaction and music-save failure tests now use a blocked backup destination rather than assuming a fixed temporary filename.

These are synthetic local fixtures. JSON reload tests exercise fresh state models; window smoke exercises real winit/Metal rendering. The initial [validation record](evidence/persistence-validation.json) predates the normal-process checks below. Neither establishes handset equivalence.

## Normal-process restart checks

The follow-up used a separately identified app bundle and profile with network and sound disabled. Actual keyboard input and window closing tested these paths:

- A draft survives closing the window and reopening the process.
- Cmd-Q saves a new draft, which is visible in Messages after relaunch.
- Blocking `messages.json.bak` with a nonempty directory makes Quit refuse to exit. The old JSON stays unchanged and the editor retains its text. Removing the synthetic blocker allows saving and exit.
- A running timer with 34.99 seconds remaining at exit expires while the process is closed. Relaunch shows its named alert. Acknowledging it and restarting again does not repeat the alert.
- A paused stopwatch remains at 12.345 seconds with both laps. Resuming it through the keyboard, quitting and reopening includes elapsed time outside the process.
- A second process using the same profile refuses to start.

This found and fixed a real Cmd-Q data-loss bug: AppKit's default Quit menu bypassed winit keyboard handling and the fallible save path. `macos_lifecycle.rs` now routes the application's `terminate:` action through that same save path. Winit still manages the window and delegate. The Dock uses this action too, but a separate Dock click was not tested. Force Quit, power loss and OS logout are not covered.

[Evidence and hashes](evidence/restart-validation.json), [save failure](images/restart-quit-refused.png), [timer restored](images/restart-timer-expired-on-restart.png), [paused stopwatch](images/restart-stopwatch-paused-restored.png). Utility records were synthetic seeds; all UI actions used the isolated profile. The final build passed 141 default tests and warnings-denied Clippy.
