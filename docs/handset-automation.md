# Handset capture and replay

**Status: real handset screenshots, remote keys and guarded multi-step replay verified on 2026-10-25, firmware 17.00.17.02.** See [observations](handset-observations.md). Full application sampling remains in progress. No loader is launched and no firmware is flashed by these scripts.

## Modules

The key names are diagnostic aliases. Home softkeys follow the configurable lock-screen shortcuts. The owner explained this, and session 278 verified left=Internet/right=Messages under Settings > Personalization > Keyboard > Lock-screen shortcuts. This explains why key 8 opened the browser on the clock page. Keep replay wire values unchanged; do not hardcode this test handset configuration as the factory default. See [observations](handset-observations.md).

- `scripts/handset_capture.py`: commands and bounded connection waiting.
- `scripts/handset/usb.py`: descriptor inventory and explicit vendor bulk interface transport, using the installed libusb. No driver detachment, reset or configuration change. The reviewed profile uses a native UCOM open/close control pair.
- `scripts/handset/protocol.py`: framed packets, fixed version/screenshot/key request builders, bounded image chunk assembly.
- `scripts/handset/session.py`: version gate, JPEG checks, key ACKs, release handling and image-guarded replay.
- `scripts/handset/test_handset.py`: synthetic protocol and failure tests.
- `scripts/handset/build_catalog.py`: initial coverage ledger; refuses to overwrite existing work.
- `scripts/handset/archive_sessions.py`: hash-checked private archive outside the repository.
- `scripts/handset/export_samples.py`: explicit reviewed screenshot allowlist.

Python 3, Pillow and libusb are already available on this Mac. Nothing new was installed. Other operating systems and runtime USB layouts are untested.

## One-time handset preparation

Ordinary charging mode exposed no data interface. On this handset, the native `#*8378#8#` test-setting menu can enable USB Log. **Open also changes sound, call restrictions, lock/backlight and fills contacts. Close is not a full rollback.** The owner explicitly accepted these effects for this test device, enabled Open, and reconnected USB with the battery installed and the original OS running. This is not a general preparation recommendation for an everyday phone. Do not hold a boot key.

```sh
python3 scripts/handset_capture.py monitor --seconds 900 --out /private/tmp/ta1278-watch-UNIQUE
```

This polls cached device/configuration descriptors every 0.5 seconds, records only changes, and never opens a phone handle. It excludes serial strings and unrelated vendors. `latest.json` gives the last inventory and `usb.jsonl` the changes. Create `STOP` inside the output directory to end the watch, or use Ctrl+C. It ends after the chosen duration; this is not a recurring automation.

## Establish a runtime connection

Create a local profile **only from observed normal-runtime descriptors**, after identifying the diagnostic interface. Fields:

- `vid`, `pid`, `interface`, `in`, `out`: integer identifiers from that observation.
- `runtime_confirmed`: true only after confirming the original UI remains running.
- `expected_firmware`: `17.00.17.02`.

The [observed profile](handset-profiles/ta1278-17.00.17.02.json) is specific to this verified runtime. Known acquisition PID `4d00` remains blocked. Mass-storage, AT, ambiguous composite interfaces and driver-owned interfaces are not automatically repurposed. `native_ucom_open: true` is accepted only for the reviewed `2e04:0019`, interface 1, IN `83`, OUT `04` profile.

```sh
python3 scripts/handset_capture.py capture --profile /private/tmp/runtime-profile.json --wait-seconds 900 --out /private/tmp/ta1278-capture-UNIQUE
```

The reviewed transport first sends UCOM open (`21/22`, value `0401`, index 1, no data). Kernel file `0x177bc4..0x177cbc` identifies this as a RAM port-state/FIFO operation. A valid software-version response must match before any screenshot or key. Close uses value `0300`. Without open, the version request timed out; with open, the version and screenshot replied immediately.

The script requires a complete 240×320 image. Some native responses have damaged JPEG prefixes, observed through byte 55. Repair is bounded to known prefixes or the first DQT segment ending at byte 89; the surviving header must match: quantization bytes match the earlier intact response and kernel tables `0xbc6820` / `0xbc6960`; surviving header bytes are checked. Raw image bytes, reconstructed JPEG, decoded PNG and hashes stay local. Unknown damage stops the run. JPEG is lossy; PNG decoding does not turn this into a raw framebuffer or prove exact LCD pixels. Logs retain command metadata, not unrelated diagnostic payloads.

Version, image and key traffic now validate `7e` framing and `7d` escaping without an added CRC on this handset. A timeout stops the attempt; no key is replayed automatically. Capture failures retain only the matching image transaction for local diagnosis. All session sequence numbers start at a random value to reject stale replies.

## Automatic keys and replay

The native handler supports tap, down and up. Request type 1, subtype 0/1/2; payload contains an unused four-byte legacy signal header followed by a little-endian 32-bit key value. Native code reads it at request offset 12. Key names are seeded from the extracted keymap. Directions, both softkeys, center, star, hash, 4 and 6 have visible handset results; held-key behavior remains untested. Call/power keys are deliberately unavailable.

Capture the current state before constructing a route. Generate a region fingerprint:

```sh
python3 scripts/handset_capture.py hash-region /private/tmp/capture/baseline.png --roi 0 24 240 296
```

A plan has `schema: 1` and a `steps` array. Each step requires a unique `name`, a `key`, and `before` containing the observed `roi` and `sha256`. Optional `after` contains a previously observed expected result in the same format. Optional `hold_ms` is 20–1000 (zero means tap); `settle_ms` is 100–5000. Supported keys: directions, LSOFT, RSOFT, CENTER, STAR, HASH and digits.

```sh
python3 scripts/handset_capture.py run --profile /private/tmp/runtime-profile.json --plan /private/tmp/observed-route.json --out /private/tmp/ta1278-route-UNIQUE
```

Every action gets a fresh pre-action screenshot. Its selected image region must match before a key can be sent. Every action then receives a fresh screenshot; a supplied postcondition must match. A route without postconditions is a guarded sampling route, not a successful behavior-equivalence test. A completed route does not mean all handset features have been sampled.

- An unexpected screen, invalid image, wrong version, timeout or disconnect stops the route. No key retries or automatic replay after reconnect.
- Held keys get a best-effort up event on failure or interruption using the same connection. After an actual disconnect, successful release cannot be guaranteed; the run stops for inspection.
- Create `STOP` inside the run directory to stop, or Ctrl+C. Device calls have finite timeouts.
- Screenshots may include the displayed screen's contents. Use synthetic messages and empty/test data. Output stays outside Git by default.
- Image guards are exact. Select meaningful stable regions that exclude the clock/cursor. Animations or JPEG differences can cause a conservative stop; no permissive matcher is silently substituted.
- Image matching is not semantic safety. Only reviewed routes may select actions. Do not build routes that make calls, send messages, pay, erase, reset or delete existing data. Removing the call key alone cannot prevent such actions through menus.

## Sampling scope

[Coverage ledger](handset-coverage.json) starts with all 30 compiled app registrations plus 48 cross-cutting cases. The ledger distinguishes partial sampling from completed cases. Runtime visibility is recorded only when observed; compiled registrations alone are not proof of a visible app. 喜马拉雅 and 支付宝 are owner exclusions; other entries stay included unless absence or a further exclusion is established.

For each present app, record entry/exit, layouts, focus/scrolling, both softkeys, context menus, dialogs/cancel, settings/persistence, empty/error states and background/resume. SMS, input, browser, music and home/lock have expanded cases. Capture every discovered subpage and transition, not just one screenshot per app. Link screenshots, action logs and firmware evidence, then compare the replica. A blocked dependency or unsafe final action must be recorded explicitly; it does not count as sampled.

## Tested handset route

The [numeric SMS regression](handset-plans/README.md) passed 172 steps and seven postcondition checks on the original handset. It requires a specific empty-editor/theme/language state and clears its own test content. Broad application coverage is still partial.

## Validation

```sh
python3 scripts/recover-lcd-diagnostic.py
python3 -m unittest discover -s scripts -p test_handset.py -t scripts
```

Tests use generated JPEGs and a simulated transport. They cover fragmented/escaped/coalesced frames, oversized and malformed data, chunk ordering, key layout, blocked keys, version gating, stale replies, guarded replay, state mismatch, invalid JPEG, STOP, key-release attempts after failure, preflight and no timeout retries. These tests never touch a phone. Live version, screenshot and guarded key replay have also passed on the reviewed runtime profile. Those observations are recorded separately from these host tests.
