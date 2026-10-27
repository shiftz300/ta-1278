# Service interface

API version 1. The shipped app uses `Offline`: no modem, real messages, calls or service connection. Existing call controls stay grey. The interface adds no dependency.

- `phone-core/src/services.rs`: public commands, events, capabilities and `Backend` trait.
- `phone-core/src/services/state.rs`: mailbox updates, delivery states, replay cursor and alerts.
- `phone-desktop/src/services.rs`: nonblocking app bridge; uses the existing message views and common notice renderer.

## Connect a backend later

Implement `phone_runtime::messaging::Backend` and construct `Runtime::with_backend(config, time, Box::new(your_adapter))`. The current desktop adapter still selects the offline backend. See [the runtime interface](runtime.md) for host-driven polling and persistence. Provide a stable backend ID and explicit capabilities. The app never discovers or connects a service automatically. No transport, credentials, server, protocol or CLI connection option is implemented now.

`submit`, `poll` and `acknowledge` must return promptly. Run network IO in your adapter's worker. `Busy` and `Disconnected` retain pending work. An accepted command belongs to the adapter until it emits a final result; it must handle its own connection recovery. The app does not repeatedly submit an accepted command within one process.

The current capabilities are message sending, message reception and alerts. `CallCommand` (dial/answer/hang up) and `CallEvent` (incoming/dialing/connected/ended) are reserved types. The current consumer rejects call events without acknowledging them. Do not advertise or emit calls yet.

## Outgoing messages

Only a new user-confirmed Send while an adapter enables sending creates a `Submission`. It contains the backend ID, an increasing request ID and state. Old/local outbox records have no submission and will never be uploaded automatically.

The app writes `messages.json` first. Only after that succeeds may the bridge submit `Command::Send`. A failed queue write restores the draft and keeps the editor open. Acceptance by `submit` does not mark a message sent.

States: `queued → accepted → sent → delivered`, with `failed` allowed before `sent`. Intermediate states may be skipped. Late progress/failure cannot undo a sent/delivered result. Here `failed` means submission failed, not a separate delivery-failure report. The current contract has no delivery-failure report, multi-recipient transaction or cancel command. A late receipt for a deleted record does not recreate it.

On process restart the app can resubmit pending work **with the same request ID**. The adapter must deduplicate `(local profile identity, backend ID, request ID)` durably. Do not send again merely because an ID is replayed. This is not an exactly-once network guarantee. Profile identity and restoring/cloning a profile must be coordinated with the future adapter; do not reuse a restored counter against an existing provider namespace without reconciliation.

There is no automatic retry scheduler. Existing UI Retry still retains the same record; use edit-as-new and confirm Send for a fresh request. Native retry rules remain a separate tested helper, not permission to retransmit.

## Incoming events

All event types derive Serde. For example, a future adapter can return:

```json
{"sequence":1,"event":{"kind":"message_received","from":"12345","body":"Hello","time_ms":1793000000000}}
```

Each backend/profile has one durable, ordered stream beginning at 1. Replays are allowed. Gaps, invalid data and unsupported events are rejected without acknowledgment; fix the adapter's stream rather than silently dropping an event. `poll` must retain unacknowledged events for replay after process failure. Do not reorder independently fetched topics; merge them in the adapter first.

The received message/status/alert and its stream cursor share **one atomic `messages.json` write** in the desktop host, or one successful `Runtime::save` transaction in an embedded host. The bridge acknowledges consumption only after that save. On a crash between save and acknowledgment, the saved cursor prevents duplicate incoming messages. Acknowledgment means durable local consumption, not that the user read the message. No outbound read-receipt command is implemented.

Incoming messages use the existing conversation list, unread marker and reader. An open composer keeps its text. Timestamp is UTC milliseconds from the backend; zero means unknown. UI labels are not modem/network cause codes.

## Alerts

```json
{"sequence":2,"event":{"kind":"alert_published","alert":{"id":"weather-1","title":"Weather warning","body":"Test notice","level":"warning","issued_ms":1793000000000,"expires_ms":null}}}
```

Publish the same alert ID with a new sequence to replace its content and clear its dismissed flag. `alert_withdrawn` with `id` removes it. Levels are `information`, `warning`, `emergency`; they are metadata, not a promise of native radio priority or a special alarm sound.

Alerts use the common title/body/scroll/softkey renderer. They wait until the active editor/modal and organizer alarm allow display. Confirm/back dismisses and saves that choice. Withdrawal or expiry closes an open alert and preserves the underlying screen. No forced interruption, location filtering, wake-from-sleep guarantee, ETWS/CMAS decoding or regulatory alert behavior is claimed.

Bounds: 8 backend streams, 32 retained alerts; ID 1–64 ASCII letters/digits or `._-:`; sender 64 UTF-16 units; body 4096; alert title 80; failure detail 1024 UTF-8 bytes. Expired alerts are removed locally. A full queue applies backpressure; the adapter must not discard blocked events. Existing `messages.json` has an 8 MiB file limit. The normal app retains its single-instance profile lock.

## Tests

`cargo test --workspace --locked` covers offline sends, save failure/rollback, submission after save, acknowledgment after save, disk reload/replay, preserved request IDs, late results, received messages during composition, alert update/dismiss/withdraw/expiry and shared rendering. The fake adapter records commands in memory; tests never send a real message or place a call.
