# Local received messages

This describes the earlier folder implementation. The current main route is [SMS conversations](sms-conversations.md); the mailbox schema and received-event boundary below remain in use.

The local mailbox now includes received messages. There is still **no modem or network SMS backend**. Normal startup never invents incoming messages. This completes the local read/reply/forward/delete path; the folder layout is a replica policy, not a claim about the native conversation screen.

## Behavior

- Messages has a received folder with unread/total counts. Existing Drafts and Outbox entries retain their positions.
- A received row shows the sender, a body preview and an unread marker. Opening it marks it read and schedules a local save. Delete confirmation alone does not mark it read.
- Reply starts a new, empty body addressed to the sender. Forward copies the body and leaves the recipient empty. Both preserve the received message.
- Reply uses the existing local outbox transaction. It reports no service and never reports delivery.
- Delete asks for confirmation, affects only the selected folder and saves through the existing mailbox store. Save failure retains dirty state and blocks normal close.
- A new local received event does not move focus or replace a draft being edited. While an outbox save transaction is pending, the adapter rejects a received event for retry after completion, avoiding a lost dirty flag on rollback.

## Data and adapter

`messages.json` gains an optional `inbox` array. Old stores containing only `drafts` and `outbox` still load. Each received record has `from`, `body` and `read`; omitted `read` defaults to false. The original received body is never used as the reply body.

`phone_core::sms::Mailbox::receive(from, body)` is the pure local input boundary. The desktop `Phone::receive_message` wrapper adds transaction guarding and persistence state. No transport polls or external calls are made. Sender length (64 UTF-16 units), body length (4096 units) and rejection of empty input are local implementation limits. Existing sender/recipient validation applies when attempting to queue a reply.

## Reproduce without private messages

Create a new test directory, then start the application against it:

```sh
python3 scripts/create-message-fixture.py --data-dir /tmp/ta1278-message-demo
cargo run --release --locked -p phone-desktop -- --data-dir /tmp/ta1278-message-demo
```

The script creates two synthetic messages and refuses to overwrite any existing `messages.json`. It is a developer fixture generator, not an import feature in the phone. Open Messages → 收件箱, read the first item, reply or forward, cancel deletion, restart and check its read state. No received messages or notifications are injected into the normal profile.

Tests cover old-store compatibility, persisted read state, reply/forward fields, local-only queueing, delete/cancel, pending-save rejection and retaining the active editor. The window smoke run adds four synthetic scenes: inbox, received reader, actions and reply editor. **Native thread grouping, native received-message layout, concatenated modem PDUs and delivery reports are not implemented.**

On 2026-10-26 the owner confirmed that both sending and receiving are required. A [real handset self-send test](sms-handset-loopback.md) now establishes a conversation list, bubble view and bottom composer. The local folder layout above has not yet been replaced with those observed interactions.
