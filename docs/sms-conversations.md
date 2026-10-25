# SMS conversations

The main Messages screen now follows the observed [handset structure](sms-handset-loopback.md): New message, peer conversations, bubbles and a bottom composer. It replaces the earlier folder-first interface. Phone/call-history entries are grey and disabled. The dial screen is removed; Call/F3 and Home-screen digits do not open it. Calling from detected SMS numbers and assigning the call-history shortcut are also disabled.

## Supported local flow

- New message, recipient selection, text entry, counts, symbols, templates and draft save/cancel retain the existing editor behavior.
- Messages groups records by the trimmed peer string. It does not merge country codes or infer contact identity. Each row shows a preview, timestamp when known and unread styling. Root menu still provides Drafts and Unsent for older records.
- Opening a conversation marks its received records read. The bottom composer starts a reply to that peer, or resumes its latest draft. An empty reply never creates a recipient-only draft.
- Send waits for the mailbox write. Failure rolls back only the staged outgoing item and keeps the editor. Success retains the explicit **unsent** result because no transport is configured; View returns to the conversation. Normal input cannot mark delivery successful.
- A local received event retains the active editor and selected root conversation. Events for the visible, unblocked conversation are marked read; other peers remain unread. During a pending queue write, input is rejected for retry after the transaction completes.
- Up at the first editor line focuses the last bubble. Up/Down moves through bubbles and detected-number focus. Left/Right scrolls long bubble text. Back returns to the composer. These horizontal scroll controls are local policy.
- Bubble menu offers Delete and Forward. Deletion confirms first. Forwarding preserves a typed reply as a draft and opens a new recipient field. Detected-number actions can open a new message or a contact form; calling stays grey. Saving/canceling that contact form returns to the conversation.
- Normal window close saves a nonempty reply. Dirty storage or a failed queue transaction blocks close. Confirmed discard retains the last saved draft, as before.

## Data and testing

`messages.json` retains `drafts`, `outbox` and optional `inbox`. Message records gain optional `time` (Unix milliseconds, zero means unknown). Outgoing records also accept `sent`, default false. Queueing always clears this flag. An explicit synthetic fixture may use `sent: true` to exercise the sent appearance; the app never invents that event.

Conversation rendering projects these arrays rather than copying bodies into a second store. Records sort by timestamp, with stable outgoing-before-incoming fallback for untimed older records. Drafts remain separate. Old files still load without fabricated dates; invalid/out-of-display-range dates are left blank. Like the existing clock, display uses UTC+8.

Create a **new test profile**, without changing normal data:

```sh
python3 scripts/create-message-fixture.py --data-dir /tmp/ta1278-conversation-demo
cargo run --release --locked -p phone-desktop -- --data-dir /tmp/ta1278-conversation-demo
```

The generator refuses to overwrite `messages.json`. Fixtures use invented peers and bodies only. There is no media import UI, SMS polling loop or modem backend.

## Evidence boundary

The real self-send test established the conversation structure, number focus and two menus. The replica uses outgoing-left and incoming-right, following the observed left composer; the direction-to-tail assignment is still a **local policy**, not independently verified native code. Date rendering, old-record ordering, root maintenance menu, long-message scrolling, number detection and background receipt policy are also bounded local choices.

Only the first contiguous number of 3–40 characters is detected. Exact multi-link parsing, receive interrupts, modem PDUs, concatenation, delivery reports, multiple recipients and MMS are not restored. The completed local flow is not a claim of exhaustive firmware equivalence.

The main application grid has only its left menu and right back icons; the extra center “选择” label was removed after comparison with the native grid. The clock Home retains its center “功能表” action.

The shared composer now uses the [measured SMS geometry](sms-composer-layout.md), including mixed-script metrics and whole-pixel wrap. Conversation viewport measurement uses that same function.
