//! Floating input lists. Geometry and palette evidence: docs/native-ui.md.
use crate::{canvas::*, theme::rgb565};
use phone_core::editor::Editor;

fn column(
    c: &mut Canvas,
    items: &[String],
    selected: usize,
    focused: bool,
    x: i32,
    y: i32,
    width: i32,
) {
    let start = selected / 4 * 4;
    let rows = items.len().saturating_sub(start).min(4);
    c.rect(x, y, width, rows as i32 * 24, c.accent(5));
    for (i, label) in items.iter().enumerate().skip(start).take(4) {
        let yy = y + (i - start) as i32 * 24;
        if i == selected {
            c.rect(x, yy, width, 24, c.accent(if focused { 2 } else { 5 }));
        }
        let text_width = c.width(label, 16).min(width - 10);
        c.fit(
            x + (width - text_width) / 2,
            yy + 4,
            label,
            16,
            rgb565(c.theme.candidate_text_rgb565),
            width - 10,
        );
    }
}
fn column_width(c: &mut Canvas, labels: &[String]) -> i32 {
    (labels.iter().map(|s| c.width(s, 16)).max().unwrap_or(30) + 24).clamp(40, 116)
}
pub fn render(c: &mut Canvas, e: &Editor) {
    let p = &e.prediction;
    if !p.active() {
        return;
    }
    let Some((cx, cy, line_height)) = c.caret else {
        return;
    };
    let words: Vec<String> = p.candidates.iter().map(|v| v.text.clone()).collect();
    let left_width = if p.spellings.is_empty() {
        0
    } else {
        column_width(c, &p.spellings)
    };
    let right_width = column_width(c, &words);
    let height = words.len().min(4).max(p.spellings.len().min(4)) as i32 * 24;
    let width = left_width + right_width;
    // Native helper changes side at caret bottom 184. Clamp to this host editor's bounds.
    let y = if cy + line_height >= 184 {
        cy - height - 5
    } else {
        cy + line_height + 1
    };
    let y = y.clamp(24, 289 - height);
    let x = (cx - 3).min(237 - width).max(3);
    if left_width > 0 {
        column(
            c,
            &p.spellings,
            p.spelling_selected,
            p.spelling_focus,
            x,
            y,
            left_width,
        );
    }
    column(
        c,
        &words,
        p.selected,
        !p.spelling_focus,
        x + left_width,
        y,
        right_width,
    );
}
