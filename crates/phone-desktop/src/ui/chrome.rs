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
