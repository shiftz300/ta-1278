# Shared phone UI

Pages now declare controls and actions through `phone-desktop/src/ui/`. There is one softkey renderer, one listbox/popup renderer and one editor renderer. Changing shared geometry no longer requires editing individual apps.

| Module | Responsibility |
|---|---|
| `ui/softkeys.rs` | Left/center/right actions, enabled/disabled labels, menu/back/close/delete/contact icons, shortcuts and playback bar |
| `ui/list.rs` | Standard rows, popup rows, focus, disabled labels, radio buttons and checkboxes |
| `ui/editor.rs` | Edit rectangle, selection, caret, wrapping and UTF-16 cursor stops; shared message-body measurement |
| `ui/text.rs` | Standard and SMS metric profiles used by both measuring and drawing |
| `ui/chrome.rs` | Title and status controls |
| `ui/mod.rs` | Shared title/footer boundaries and center-label baseline |

For example, ordinary pages use `Softkeys::back("选择")`; a message editor supplies Menu, Send/disabled Send and Back/Delete as three typed actions. Apps do not draw their own softkey text or delete glyph. All labels share y=289 and the same slot alignment. The application grid supplies an empty center slot; full-screen Snake supplies no bar.

List and popup styles specify their actual row height, panel position and marker palette. The same list renderer handles plain rows, single selection, multiple selection and disabled actions. SMS keeps its measured mixed-script typography through a profile; browser/form editing uses the standard profile in the same `Edit` control. Conversation viewport and standalone SMS input measure their bubble using the same `MessageLayout` function.

App-specific content remains in `views/`: calendar cells, conversation bubbles, browser document flow, the music Home's larger first row and Snake's playfield. Their native layout differences are explicit content or style choices. App models own activation, navigation and persistence; drawing a button does not bypass those checks.

## Native evidence

[Shared UI evidence](evidence/shared-ui-framework.json) verifies all **751 indexed native window byte slices** against the original firmware hash. It records relevant Snake/chat descriptors and the correlated control macros:

- `WIN_SOFTKEY`: three resource-ID slots for left, middle and right.
- `CREATE_LISTBOX_CTRL` / `CREATE_POPMENU_CTRL`: shared list and popup control families.
- Edit/form/title/status descriptors: separate framework controls rather than per-app drawing implementations.

The public MMK reference is from another product/version and is used to interpret the captured descriptors. The Rust UI is a local implementation; reference C code is not compiled or executed. Verified descriptor bytes do not establish that every dynamic attribute or original control implementation has been recovered.

```sh
python3 scripts/check-ui-framework.py
python3 scripts/check-ui-framework.py --baseline BASELINE_FRAMES --frames CURRENT_FRAMES
```

The second command also checks that content above y=284 is unchanged outside the live Home clock and four explicitly recorded popup-inset corrections. Baseline frames for this pass are archived at `../ta-1278-source/replica-validation/20261026-sms-composer/ta1278-composer-layout-release/`.

## Maintenance

Add a shared action or control variant in `ui/` when it is used across pages. Configure genuine native differences through a style or a distinct content control. Keep colors and measured geometry out of app routing code. Do not reintroduce app-specific footer coordinates, Unicode substitutes for delete/back icons, or separate edit-measurement formulas.

Cross-app regression tests compare the center slot across settings, Snake, tools and browser, compare the delete control across message/browser editors, and ensure an empty menu slot cannot retain an editor label. The existing editor and app-flow tests remain in place.

`ui/form.rs` adds reusable Input/Date/Choice fields and avatars; `ui/art.rs` contains the verified native control artwork. The [phonebook/flashlight pass](contact-torch-layout.md) uses these controls and adds an accent-panel style to the same softkey renderer.
