//! Native two-field conversion form; pair navigation uses the shared list.
use super::*;
pub(super) fn render(c: &mut Canvas, p: &Shell) {
    let labels = p.conversion_pair().labels;
    let state = &p.tools.conversion;
    for (i, label) in labels.iter().enumerate() {
        let lines = crate::ui::form::numeric(
            c,
            label,
            &state.fields[i],
            i == state.focus,
            24 + i as i32 * 74,
        );
        if i == state.focus {
            c.layout.editor = Some(lines);
        }
    }
    let nonempty = !state.fields[state.focus].units.is_empty();
    Softkeys::new(
        if nonempty {
            Button::Close
        } else {
            Button::None
        },
        Button::None,
        if nonempty {
            Button::Delete
        } else {
            Button::Back
        },
    )
    .draw(c);
}
