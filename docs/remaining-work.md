# Delivery scope

The owner no longer requires exhaustive firmware or application reconstruction. This remains a standalone phone interaction replica. Its public API, data model and UI describe phone behavior only; intended downstream use does not add fields or features.

The **1–2 day target** guides prioritization, not a guaranteed duration. The earlier 5–10 day estimate is withdrawn because it had no timed task breakdown.

## Required for the next usable version

| Area | Acceptance boundary |
|---|---|
| Messages | Complete local compose/read/reply/forward/draft/delete paths, received-message fixtures, visible unsent results, and persistence without text loss. No real modem. Conversation structure is sampled; exact bubble-direction mapping and interruption behavior remain gaps. |
| Basic browser | Simple HTTP(S) text, links, basic forms, history, bookmarks and Back/Forward. No full modern HTML/CSS/JS engine. |
| Input and navigation | Keyboard and mouse-button softkey mapping, text entry, focus/scroll, back/cancel and return to the calling app. No stuck or misleading active controls. |
| Common settings | Theme and left/right shortcuts persist and cancel correctly. Any additional setting must have a concrete local effect; unsupported settings remain explicit. |
| Local organizer | Contact create/edit/delete/recipient selection, dated calendar entries, alarm editing and local alerts. Full native recurrence, groups, wake service and ringtone behavior are not required for this version. |
| Existing utilities | Keep calculator, stopwatch, timer and Snake usable; fix broken actions and visible layout issues. Their existence does not require exhaustive hardware-service reconstruction. |
| Verification | Exercise each supported path, restart persistence, cancel/delete and long text. Use existing handset/code evidence first; perform new reverse engineering only to resolve a relevant discrepancy. |

## Later local completion pass

The owner subsequently asked to finish the deferred local flows. Contacts pictures/groups/ringtones, playlists, image/video browsing, copy/move, common settings and calendar reminders are now implemented and checked. Exact native behavior is claimed only where code or captures support it. [Implemented flows and hard limits](local-completion.md).

## Outside current acceptance

Real call/SMS backends, complex web compatibility, Alipay and Ximalaya remain excluded. The latest request adds T9 algorithm recovery and replaceable input/service interfaces; the original candidate engine is now integrated through bounded ARM interpretation, including spelling/page selection, associations and native dictionary persistence. Full semantic decompilation and exhaustive handset parity remain unclaimed. See [T9 recovery](t9-recovery.md) and [interfaces](input-engine.md). Full native advanced settings and exhaustive native error/interrupt branches remain outside this implementation. File/gallery/video now have working local workflows, with the codec and fidelity limits stated above. Unsupported entries must not pretend to work.

Music must support actual local audio playback. The latest storage decision removes VFS entirely: first launch selects one host folder, and Files, music, images and video access it directly through `std::fs`. Only 手机存储 is shown. There is no import UI or memory-card mount. Older VFS reports describe a superseded implementation. See [storage](storage.md).

## Evidence and decisions

Local received-message paths are now implemented and checked: unread state, reading, reply, forward, delete/cancel, persistence and synthetic fixtures. This meets the local mailbox path requirement above; it does not establish complete native parity. See [inbox validation](evidence/sms-inbox-validation.json).

The owner confirmed both send and receive on 2026-10-26. A [single real handset loopback](sms-handset-loopback.md) now supplies the basic native conversation UI evidence. Conversation grouping and the inline composer are implemented; direction-to-tail mapping and interruption cases remain evidence gaps. Real modem services are still outside the replica scope.

The subsequent [conversation implementation](sms-conversations.md) completes that local UI replacement and retains the storage-failure safeguards. The owner explicitly excludes telephone replication: call history and calling actions are grey and inert, with no dial screen. Remaining SMS parity limits are documented in the conversation page; real services remain excluded.

Known native behavior takes priority. Unknown visible behavior is recorded explicitly; a conservative local replacement is acceptable only when identified as such in the technical evidence. Do not label a replacement as recovered firmware behavior. Do not add implementation notes throughout ordinary phone screens.

Ask the owner when uncertainty changes a visible interaction, supported capability or persistent-data behavior in a material way. Resolve routine code organization and verification details directly. This version is complete when the supported phone tasks work consistently, not when every firmware function has been explained.

## Persistence follow-up

Pending alerts and snoozes, timer/stopwatch state and radio preferences now survive restart. JSON stores share bounded reads, atomic writes, one previous backup and an exclusive profile lock. [Checks and remaining limits](persistence.md). These are host data policies, not newly recovered handset behavior.

The normal-process follow-up also verified draft recovery, timer expiration, paused/running stopwatch recovery and save-failure protection through actual macOS windows. It fixed Cmd-Q bypassing the save path. These checks close the persistence acceptance gap for the exercised local flows; they do not extend the native-fidelity or backend scope above. [Restart evidence](evidence/restart-validation.json).

## Current closure — 2026-10-27

The previously outstanding original T9 runtime integration is now delivered: native candidate order, spelling groups, page selection, context associations and private native-format dictionary persistence are wired into all shared editors. Invalid keys retain composition; suggestions require explicit confirmation. # (Shift+3) switches the observed input modes through the shared phone key path; the F4/F5 shortcuts have been removed. System IME and the two excluded application entries remain removed.

The supported local scope has passed the final automated and window checks in [T9 runtime validation](evidence/t9-runtime-validation.json). Remaining work is the declared backend/hardware scope and unsampled native fidelity, not an unconnected predictor or missing local storage path. Exhaustive firmware reconstruction is still not claimed. Further screenshot discrepancies should be handled against their concrete references.

## 2026-10-27: new native-page audit

The owner requested a new audit of all retained applications and replacement of VFS with one directly mapped local folder. This supersedes the earlier storage design and broad closure wording above. Direct folder mapping, inline calendar editing, timer pages and confirmed multi-file deletion are implemented and tested; working local flows are not blanket native acceptance. The handset has been reconnected through its hub and firmware 17.00.17.02 has been freshly verified. Raw tap acknowledgment did not change the first screen; explicit 100 ms down/up did. The desktop has no real call/SMS backend, and calls and the two excluded services remain outside scope.


The current per-application evidence and remaining gaps are in [the native-page audit](native-page-audit.md). Since the owner's quiet-time request, handset key tests and all audio/vibration tests are suspended; offline inspection and CPU rendering continue. A separate muted desktop folder-chooser check was attempted; its process started but the automation tool could not attach, so visual acceptance remains open.
