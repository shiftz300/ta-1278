# Shared phone UI

Pages now declare controls and actions through `phone-desktop/src/ui/`. There is one softkey renderer, one listbox/popup renderer and one editor renderer. Changing shared geometry no longer requires editing individual apps.

| Module | Responsibility |
|---|---|
| `ui/softkeys.rs` | Left/center/right actions, enabled/disabled labels, menu/back/close/delete/contact icons, shortcuts and playback bar |
| `ui/list.rs` | Standard rows, popup rows, focus, disabled labels, radio buttons and checkboxes |
| `ui/navigation.rs` | Shared vertical focus movement, explicit wrapping/clamping, empty/shrinking-list bounds |
| `ui/menu.rs` | Typed menu items; one source for labels, enabled state, markers and activation intent |
| `ui/popup.rs` | Parent dimming, popup panel/scrolling and footer composition |
| `ui/editor.rs` | Edit rectangle, selection, caret, wrapping and UTF-16 cursor stops; shared message-body measurement |
| `ui/text.rs` | Standard and SMS metric profiles used by both measuring and drawing |
| `ui/chrome.rs` | Title and status controls |
| `ui/mod.rs` | Shared title/footer boundaries and center-label baseline |

For example, ordinary pages use `Softkeys::back("选择")`; a message editor supplies Menu, Send/disabled Send and Back/Delete as three typed actions. Apps do not draw their own softkey text or delete glyph. All labels share y=289 and the same slot alignment. The application grid supplies an empty center slot; full-screen Snake supplies no bar.

List and popup styles specify their actual row height, panel position and marker palette. The same list renderer handles plain rows, single selection, multiple selection and disabled actions. SMS keeps its measured mixed-script typography through a profile; browser/form editing uses the standard profile in the same `Edit` control. Conversation viewport and standalone SMS input measure their bubble using the same `MessageLayout` function.

App-specific content remains in `views/`: calendar cells, conversation bubbles, browser document flow, the music Home's larger first row and Snake's playfield. Their native layout differences are explicit content or style choices. App models own activation, navigation and persistence; drawing a button does not bypass those checks.

## Maintenance

Add a shared action or control variant in `ui/` when it is used across pages. Configure genuine native differences through a style or a distinct content control. Keep colors and measured geometry out of app routing code. Do not reintroduce app-specific footer coordinates, Unicode substitutes for delete/back icons, or separate edit-measurement formulas.

Cross-app regression tests compare the center slot across settings, Snake, tools and browser, compare the delete control across message/browser editors, and ensure an empty menu slot cannot retain an editor label. The existing editor and app-flow tests remain in place.

`ui/form.rs` adds reusable Input/Date/Choice fields and avatars; `ui/art.rs` contains the verified native control artwork. The phonebook/flashlight pass (research archive) uses these controls and adds an accent-panel style to the same softkey renderer.

## Adding a menu

Declare application actions with an enum. Do not make row numbers the meaning of an action. The same items drive rendering and keyboard activation:

```rust
use crate::ui::{menu::{self, Event, Item}, popup::Popup};

#[derive(Clone, Copy)]
enum Action { Open, Delete }

let items = [
    Item::new(Action::Open, "打开"),
    Item { enabled: can_delete, ..Item::new(Action::Delete, "删除") },
];

// View: render the parent content first, then the popup exactly once.
Popup::default().draw(canvas, items.len(), selected, |i| items[i].row());

// Controller: execute effects here, never while rendering.
match menu::event(&items, &mut selected, key) {
    Event::Activate(Action::Open) => open_selected(),
    Event::Activate(Action::Delete) => request_delete_confirmation(),
    Event::Back => restore_parent(),
    Event::Focus | Event::Ignored => {}
}
```

The snippet illustrates the view and controller phases; they run separately. Actual examples are `shell/radio.rs` with `views/radio.rs`, and `shell/mail.rs` with `views/mail.rs`. Radio uses a fixed array; mail uses static slices. Neither requires an allocation to build its menu rows. `Item::marker` supplies a check/radio/switch marker when needed. Disabled entries remain focusable, but the shared event handler never activates them. Center activates and Right cancels; Left is not silently treated as confirmation.

## Choosing a page control

- **Ordinary list:** `list::draw` or `list::plain`, with `navigation::vertical(&mut index, count, key, Boundary::Wrap)` for cyclic focus. Use `Boundary::Clamp` where the existing UX stops at the ends, such as message folders. Navigation only handles Up/Down; the app owns selection effects and Back. Empty lists normalize to index zero without division, and a shrunken list cannot overflow focus.
- **Popup over a page:** render the parent, then `Popup::default().draw(...)`. It owns dimming, list panel, overflow scrollbar, bottom gap and default Select/Back softkeys. Do not also dim the parent or redraw its footer afterward.
- **Standalone/already shaded popup:** use `.unshaded()`. Use `.footer(...)` for a verified footer difference, for example an unavailable Format action with an empty center label.
- **Form/editor:** retain `form::Field`, `form::numeric`, `form::scroll_to`, Canvas clipping and `editor::Edit`. The app owns field values and commit/cancel; the control owns text/caret geometry.
- **Custom content:** retain the browser's measured child menus, timer's positioned popup, conversation bubbles and Snake board where their layout differs. Do not force them into a standard list merely to reduce code.

App models continue to own return destinations, selected identities, save failures and asynchronous work. There is no new global navigation stack, dynamic widget tree, callback registry or serialized UI schema. This pass introduces no dependency and does not change persisted application/config formats.

## 2026-10-27 framework follow-up

Thirteen standard popup call sites now use the common composition, across contacts, calendar, files, messages, radio and music. Eleven list routes use the common focus function. Radio and mail menus use typed items for both view and activation, replacing parallel row-index switches. Other apps can migrate when touched; this is not a claim that every existing route now uses typed items.


```sh
TA1278_AUDIT_FRAMES=/tmp/ta1278-before cargo test -p phone-desktop --locked views::audit
# Make the UI change, then render to a separate directory.
TA1278_AUDIT_FRAMES=/tmp/ta1278-after cargo test -p phone-desktop --locked views::audit
python3 scripts/compare-ui-frames.py /tmp/ta1278-before /tmp/ta1278-after --report /tmp/ui-comparison.json
```

The comparator requires identical frame names and 240×320 dimensions. Only the clock rectangle of `configured-home.png` is exempt; it records the actual difference bounds and hashes. These are CPU render regressions against the preceding working tree, not new handset measurements or winit/GPU acceptance. No phone access or sound output was used.

### Source ownership

Use one-word filenames under the owning feature. `ui/candidates.rs` and
`ui/tones.rs` are shared controls; `views/browser/chrome.rs` is browser-specific.
`shell/editor.rs` selects the active editor for both drawing and input, including
modal/reminder blocking. `ui/form.rs` shares the selected date/time digit renderer.
`assets.rs` centralizes embedded PNG paths and decoding; resource IDs and order
stay explicit at each call site. This avoids a separate resource registry or runtime
path lookup.

Embedded hosts call `Runtime::render`; the desktop host uses the same entry point. Controls remain private implementation details rather than a second public widget API. See [runtime](runtime.md).

The Shell follow-up also migrates common overlays and contact root/detail menus
to static typed Item declarations. Labels, enabled state, markers and activation
come from the same entry. Grid movement and verified SoftLeft differences are
retained; painting borrows overlay editors rather than cloning them. Radio/mail
and the remaining app-specific controls continue using the shared components.
