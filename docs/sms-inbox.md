# Received messages

Messages use peer conversations, bubbles and an inline composer. See [conversation behavior](sms-conversations.md).

Incoming messages enter through the [service interface](services.md). No real transport is connected. Unread/read state, reply, forward, deletion and persistence work with local records.

For an isolated test profile:

```sh
python3 scripts/create-message-fixture.py --data-dir /tmp/ta1278-message-demo
cargo run --locked -p phone-desktop -- --data-dir /tmp/ta1278-message-demo
```

The fixture contains synthetic messages only.
