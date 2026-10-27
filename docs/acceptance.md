# Local acceptance

Run `sh scripts/accept.sh` from this repository. It needs the existing Rust toolchain, Python 3, FFmpeg and an audio output device. It installs nothing. It uses an isolated temporary profile, generated quiet tones and a localhost HTTP server. No phone connection, personal files or external service is used. Evidence is kept at the printed path.

This opt-in test briefly plays WAV and MP3 through the desktop audio adapter at volume 1/10. The generated source peaks below -40 dBFS. It checks:

- Folder selection, saved-folder reuse and a file operation within that folder.
- Message editing and draft restore; mock send, receive, acknowledgement, read, deletion and restart persistence.
- HTTP links, text/check/select fields, GET submission, back navigation and retained field values.
- WAV/MP3 decoding and actual output, pause, next track, saved position, restart and end of playback.
- Settings and runtime state after reconstructing both the runtime and desktop adapters from disk.

Messages use a test backend. The normal application remains offline until a backend is connected. Playback checks measure the output clock and sink state; they are not an acoustic recording or an assessment of handset sound quality.

## Desktop window check, 2026-10-27

The native macOS application was also operated by keyboard with a separate temporary profile. Confirmed in the actual window: explicit storage-folder selection, reopening with only `--data-dir` to reuse that folder, editing and saving the greeting, editing a local message and receiving the offline-send notice, localhost navigation and form submission, and music play/pause/track/restart behavior.

The first-launch native folder chooser opened, but its auxiliary process could not be controlled by automation. Its final folder-selection click has **not** been manually accepted. The `--storage-dir` route and subsequent saved-folder reuse passed; the chooser's cancel/missing-directory logic has automated coverage. No user action is required to repeat the CLI route.

This check found and fixed premature ellipsis in shared fitted labels, including browser Submit buttons. Regression coverage checks a fitting label remains intact and an overflowing label stays within its bounds.

Logs, screenshots and fixture hashes are kept outside the application repository in `ta-1278-source/validation/acceptance-20261027/`. The earlier handset comparison is recorded in [review.md](review.md); its remaining fidelity limits still apply.
