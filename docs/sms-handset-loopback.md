# Handset SMS loopback

On **2026-10-26 (Asia/Shanghai)** the owner clarified that both sending and receiving are in scope, and explicitly allowed test messages to their own specified handset number. This permits this bounded handset test; the desktop replica still has no real SMS transport.

## What ran

The existing runtime interface `2e04:0019`, interface 1, was present. The version gate verified `17.00.17.02`. No loader, reboot, USB reconfiguration or firmware write was used. Initial sandbox enumeration was empty; the same read-only check outside the sandbox found the device.

From the observed app grid, the test opened Messages and New message, entered the authorized recipient and the unique numeric body `127810260001`, and checked both on screen. The first send guard stopped **before any key** because the caret blinked. Pixel differences were confined to `(143,224)..(161,256)`. After inspecting the new image, the send guard used `(0,24)..(143,254)` to retain the recipient and body while excluding the caret. One CENTER send tap was then acknowledged. No retry was sent.

The message list initially returned without the new row, then showed a new thread for `177****2597` with the test body and a message status icon. Opening only this thread showed two copies of the body with opposing tails and timestamps. The status icon disappeared after reading. This is an observed self-send/receive UI round trip, not a decoded modem delivery report or PDU trace.

## Observed interaction

- The root is a conversation list with a full-width New message row. It is not the replica's Drafts/Outbox/Inbox folder menu.
- A conversation has the peer in the title, bubbles and timestamps above a persistent bottom input area. The empty input uses the same character counter and input-mode affordances as New message.
- In this short sample, bubble rules extend to x=200 from the left or x=40 from the right. One tail points down near x=30; the other near x=209. Timestamp alignment alternates. **Which tail belongs to which direction remains unverified independently**; do not infer the mapping solely from screen order.
- UP moves from the empty input into the latest bubble. Another UP focuses its detected numeric text; the center softkey becomes Save. The body is underlined because it is recognized as a number.
- A bubble's left menu showed Delete and Forward. With detected numeric text focused, the menu instead showed Call number and Send message. These actions were inspected but not activated.
- Back exits the menu, conversation and message list. The test finished at the app grid. Its messages remain on the handset; unrelated records were not opened or deleted.

## Evidence and limits

[Session evidence](evidence/sms-handset-loopback.json) records hashes, the guard stop and observations. Raw screenshots, key plans and logs are private local files under `../ta-1278-source/handset-sessions/20261026-sms-loopback/`. The root screen exposed existing conversation previews, so no raw frame is exported to this repository. The full recipient is absent from public reports and fixtures.

This closes the previous gap of having no real send/receive observation. It does **not** establish long-message layout, multiple recipients, unread behavior while editing, failure/retry UI, receive interrupts, direction-to-tail mapping or conversation database ordering. Existing recovered retry code remains separate evidence.

The later [conversation implementation](sms-conversations.md) replaces the folder-first route with peer grouping and an inline composer. Its local behavior, tests and remaining parity limits are documented separately from this handset observation.
