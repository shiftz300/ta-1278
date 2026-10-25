//! Soft-key geometry measured from native 240 x 320 captures.
use super::*;
/// Native session 283: icon ink x=6..29 / 209..232, y=288..311.
/// Original resources have one transparent pixel around their 24px ink.
pub(super) fn home(c: &mut Canvas, left: usize, right: usize) {
    c.rect(0, 284, 240, 36, [0, 0, 0, 255]);
    c.home_shortcut(left, 5, 287);
    c.home_shortcut(right, 208, 287);
    let x = (240 - c.width("功能表", 20)) / 2;
    c.text(x, 288, "功能表", 20, WHITE);
}

pub(super) fn browser(c: &mut Canvas, menu: bool, center: &str, close: bool) {
    icons(c, menu, close);

    let x = (240 - c.width(center, 20)) / 2;
    c.text(x, 290, center, 20, WHITE);
}
pub(super) fn icons(c: &mut Canvas, menu: bool, close: bool) {
    if menu {
        c.softkey_symbol(false, 5, 287);
    }
    if close {
        for d in 0..14 {
            c.rect(215 + d, 291 + d, 2, 2, WHITE);
            c.rect(228 - d, 291 + d, 2, 2, WHITE);
        }
    } else {
        c.softkey_symbol(true, 208, 287);
    }
}

// Recipient action silhouette, fitted to the native softkey; not a recovered resource.
pub(super) fn contacts(c: &mut Canvas) {
    for (x, y) in [(10, 294), (24, 294), (17, 292)] {
        for dy in -3..=3 {
            for dx in -3..=3 {
                if dx * dx + dy * dy <= 9 {
                    c.rect(x + dx, y + dy, 1, 1, WHITE);
                }
            }
        }
        for row in 0..6 {
            let half = (row + 2).min(5);
            c.rect(x - half, y + 5 + row, half * 2 + 1, 1, WHITE);
        }
    }
}

pub(super) fn center(c: &mut Canvas, label: &str) {
    let x = (240 - c.width(label, 20)) / 2;
    c.text(x, 288, label, 20, WHITE);
}
