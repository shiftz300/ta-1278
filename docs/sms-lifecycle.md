# Local SMS lifecycle

The outbox now has a scrollable reader, options, forward/edit-as-new, retry and confirmed deletion. A queued message stays **unsent**: there is no modem backend. Retry shows the same unavailable-service result and retains the same record. It does not create another message or simulate delivery.

## Storage fix

Previously, Send immediately removed the linked draft, added an outbox item and closed the editor before the file write succeeded. A failed write could therefore leave the editor closed and the old disk draft unchanged.

Now queueing is a transaction:

1. Validate the message and stage the outbox change, retaining only the removed draft and new record as undo data.
2. Keep the editor and block further edits/duplicate submission while `messages.json` is being written.
3. On successful atomic replacement, close the editor and show the unsent result.
4. On failure, remove only that staged item, restore the original draft at its position and retain the edited text. Existing unsaved changes retain their dirty flag.

The window-close path flushes this transaction before attempting to save a draft. A failed queue write keeps the window open. The reader's edit action explicitly creates a new message; it does not silently modify the original outbox item. Forward copies the body and clears the recipient. Delete requires confirmation; Cancel restores the prior row/page.

These are host storage and service policies, not a reconstruction of the phone's NV database.

## Code verification

`python3 scripts/recover-sms-lifecycle.py` rereads the immutable firmware, checks its SHA-256 and the previously recorded code-range hashes, creates a fresh Thumb listing and evaluates only the bounded integer comparisons at `0x883c5e68..0x883c5ecc`. All 65,536 inputs were checked; exactly 19 network reason values reach the allowed-return branch. No firmware code, store, call, mutable RAM or device operation is executed.

The Rust `sms_retry.rs` translation follows the reviewed error-handler ordering at `0x883c407e`:

- Event cause 0: ignore this error-handler call.
- Event cause 4 or 9: finish without retry, flag false.
- Other causes: an allowed network reason and nonzero retry byte select the continuation branch and decrement the byte.
- Otherwise: finish without retry, flag true.

A continuation is not an unconditional send or a 5-second timer. Its downstream context guards and the original initial retry budget remain unresolved. The pure rules are available for logic tests; the absent desktop SMS transport never invents a network error to activate them.

Seven operation labels were extracted separately after checking their literal pointers and adjustments. The empty-recipient notice uses native operation code 4, `电话号码为空`. Operation code 0 means progress (`正在发送…`), not success. UI operation codes and network reason codes are distinct. Log-only native branches have no invented label.

Evidence: [ranges, output hashes and limits](evidence/sms-lifecycle.json), [bounded listing](evidence/sms-lifecycle-thumb.txt).

## Validation and remaining scope

Tests force an actual temporary-file write failure, verify disk preservation and in-memory rollback, then retry a successful save. They also check blocked duplicate input, retry without duplicate records, reader scrolling, folder focus, forward/edit copies, delete cancellation, stale undo rejection, native label lookup and error priority across the full 16-bit reason domain.

No handset command or personal-message read occurred during this pass. Screens use synthetic `12345` data. The new folder/reader/result geometry is **replica policy**; native conversation grouping, received-message UI, multiple recipients, MMS, real send cancellation and full asynchronous storage races remain open. Existing mailbox JSON remains compatible; no new persisted fields were required. Older drafts longer than the default editor limit reopen at their stored length instead of losing the body.

Compiled conversation candidates for the next trace: `MMISMS_CHATBOX_CHILD_WIN_ID` at file `0x2112e0`, handler `0x88b4f3fb`; `MMISMS_SHOW_CHAT_SMS_WIN_ID` at `0x728004`, handler `0x883e4bc1`; options descriptor at `0x728128`, handler `0x883e6e51`. These come from the existing descriptor index. Their presence alone does not prove which layout is active on this handset.

## Recipient-focus layout correction

The final forward fixture exposed an existing renderer bug: a long inactive body could grow to y=24 and cover the recipient input. The reviewed handset image `sms-recipient-focus.png` has the inactive body limited to y=91..254. The renderer now enforces that 163-pixel bound, paints six visible body rows without an inactive caret, and preserves recipient cursor metrics. A pixel-region test verifies that long text cannot change the recipient header. The new comparison uses different synthetic bodies in the same focus state; it verifies a layout constraint, not the native forward command.

The empty recipient caret also starts at x=6 before its placeholder, matching the same capture. The header test checks this position.
