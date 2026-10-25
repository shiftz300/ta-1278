# Delivery scope

The owner no longer requires exhaustive firmware or application reconstruction. This remains a standalone phone interaction replica. Its public API, data model and UI describe phone behavior only; intended downstream use does not add fields or features.

The **1–2 day target** guides prioritization, not a guaranteed duration. The earlier 5–10 day estimate is withdrawn because it had no timed task breakdown.

## Required for the next usable version

| Area | Acceptance boundary |
|---|---|
| Messages | Complete local compose/read/reply/forward/draft/delete paths, received-message fixtures, visible unsent results, and persistence without text loss. No real modem. Native conversation layout remains an explicit evidence gap until verified. |
| Basic browser | Simple HTTP(S) text, links, basic forms, history, bookmarks and Back/Forward. No full modern HTML/CSS/JS engine. |
| Input and navigation | Keyboard and mouse-button softkey mapping, text entry, focus/scroll, back/cancel and return to the calling app. No stuck or misleading active controls. |
| Common settings | Theme and left/right shortcuts persist and cancel correctly. Any additional setting must have a concrete local effect; unsupported settings remain explicit. |
| Local organizer | Contact create/edit/delete/recipient selection, dated calendar entries, alarm editing and local alerts. Full native recurrence, groups, wake service and ringtone behavior are not required for this version. |
| Existing utilities | Keep calculator, stopwatch, timer and Snake usable; fix broken actions and visible layout issues. Their existence does not require exhaustive hardware-service reconstruction. |
| Verification | Exercise each supported path, restart persistence, cancel/delete and long text. Use existing handset/code evidence first; perform new reverse engineering only to resolve a relevant discrepancy. |

## Outside current acceptance

Real call/SMS services, exact T9 internals, complex web compatibility, Alipay and Ximalaya remain excluded. Full advanced settings, file/gallery/video workflows and exhaustive native error/interrupt branches are not prerequisites. Unsupported entries must not pretend to work.

Music must support actual local audio playback. The owner's later decision replaces the import-copy proposal: use an in-process VFS, mounting storage volumes without an import UI. Directory volumes, runtime capability flags, MP3/WAV output and basic file operations are implemented. Disk-image/FAT backends, full native file-manager fidelity and playlist editing remain outside the current implementation. See [storage](storage.md).

## Evidence and decisions

Known native behavior takes priority. Unknown visible behavior is recorded explicitly; a conservative local replacement is acceptable only when identified as such in the technical evidence. Do not label a replacement as recovered firmware behavior. Do not add implementation notes throughout ordinary phone screens.

Ask the owner when uncertainty changes a visible interaction, supported capability or persistent-data behavior in a material way. Resolve routine code organization and verification details directly. This version is complete when the supported phone tasks work consistently, not when every firmware function has been explained.
