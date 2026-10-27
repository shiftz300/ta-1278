use crate::canvas::{Canvas, WHITE};
pub fn title(c: &mut Canvas, label: &str) {
    c.text(6, 1, label, 14, WHITE);
}
pub fn status(c: &mut Canvas) {
    c.text(184, 1, "离线", 12, WHITE);
    c.rect(218, 4, 17, 9, WHITE);
    c.rect(220, 6, 12, 5, [30, 30, 30, 255]);
    c.rect(235, 7, 2, 3, WHITE);
}

/// Shared waiting caption. Dot animation is local; placement follows the library capture.
pub fn waiting(c: &mut Canvas, label: &str, now_ms: u64) {
    let phase = (now_ms / 180) as usize % 3;
    for (i, (x, y)) in [(105, 96), (102, 102), (102, 114)].iter().enumerate() {
        c.rect(*x, *y, 4, 4, if i == phase { WHITE } else { c.accent(5) });
    }
    let width = c.width(label, 16);
    c.fit(((240 - width) / 2).max(6), 134, label, 16, WHITE, 228);
}
