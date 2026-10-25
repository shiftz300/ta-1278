# Phonebook and flashlight layout

Firmware 17.00.17.02, 2026-10-26. This pass uses original system resources, bounded static code and existing reviewed handset captures. It sends no new handset commands and imports no personal contacts.

## Flashlight

The old screen incorrectly displayed the app-menu icon and a large text state. The native power page uses original image `0x40170027` when off, and theme-selected images `0x40170028..0x4017002d` when on. The six-theme order is verified at `user.bin+0x7a9b50`. The 50×62 image is drawn at (94,100), at native size. The footer is a theme-colored panel starting at y=282, with original phone icon `0x40170020`, center 打开/关闭 and Back. Center toggles the local state.

These are new variants of the shared art/softkey controls. The app does not operate a physical LED. Exact top-level quick-toggle behavior versus entry through Other remains outside this page correction.

[Off comparison](comparisons/torch-off-comparison.png), [on comparison](comparisons/torch-on-comparison.png).

## Phonebook

The old root was an ordinary list with a generic empty message and a direct Add softkey. It now has the observed search underline, avatar/name contact rows, menu icon and Open action for available records. Its menu uses the captured six entries; unavailable group/share/speed-dial/settings actions are grey. Case-insensitive substring search is a local policy. The native empty-store caption has not been observed, so the empty local list has no invented generic caption or active Open action.

New contact uses an avatar row and inline surname, given name and mobile/home/work fields. Birthday, ringtone and group use their measured date/choice geometry. The common `Edit`, form fields, input-mode marker, softkeys and popup renderer own their drawing. Whole-application storage remains the existing nine-string contact schema; there is no device data import.

The observed row tops are 24, 86, 154, 222, 290, 358, 426, 486 and 550. Numeric fields scroll to an underline at y=265, birthday to y=269, and focused choice panels start at y=212. These are measured control profiles, not a claim to have recovered the original layout engine.

The left menu provides local Save. Moving between inline fields commits the current editor into the draft. The right softkey deletes while a text field is nonempty; Back on an empty or non-text field cancels unsaved changes. Saving updates the local record. SMS recipient selection preserves the existing message body. Filtered deletion confirms and targets the saved record ID rather than its filtered row number.

Contact details now show the avatar/name and per-number Call/Send message rows starting at y=76. Calling and speed dial remain grey/inert by owner instruction. Send message opens the local SMS composer. The captured detail menu and delete confirmation replace the earlier generic field list. No actual call or message is sent by the replica.

- [New contact](comparisons/contact-new-comparison.png)
- [Phone fields](comparisons/contact-phone-fields-comparison.png)
- [Birthday](comparisons/contact-birthday-comparison.png)
- [Ringtone](comparisons/contact-ringtone-comparison.png)
- [Group](comparisons/contact-group-comparison.png)
- [Details](comparisons/contact-details-comparison.png)
- [Options](comparisons/contact-options-comparison.png)
- [Delete confirmation](comparisons/contact-delete-comparison.png)

## Evidence and limits

Run `python3 scripts/check-contact-torch.py` to check both original `user.bin` and `mmi.bin` hashes, eight original image resources, the torch table/operands and four phonebook/torch window descriptors. [The manifest](evidence/contact-torch-layout.json) stores addresses, hashes, raw code ranges and the source distinctions.

Public contact previews use only the already verified test-mode fixture `a0000 / 1230000` and new invented local text. Existing private root captures were used only for control chrome/geometry; no contact text from them is published. The runtime starts with no generated contacts.

Picture assignment, sharing, group management and ringtone selection are unavailable. Picture is shown as disabled; birthday digit editing and the Save popup are local controls, not verified native picker/menu behavior. Exact search matching, long-field horizontal scrolling, discard prompts, root empty-state text and all SIM/group operations remain unverified. Fonts and some glyph edges differ. Disabled call/share controls deliberately differ from the active native references.

Image and text resource IDs have separate namespaces. The torch image IDs were checked against the image catalog and native drawing calls, rather than treating the softkey operands as string IDs. The birthday control uses 16px date digits and no text-input mode/Delete action; its digit stepping and invalid-date handling remain local policies.
