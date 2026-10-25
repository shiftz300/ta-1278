//! Soft-key geometry measured from native 240 x 320 captures.
use super::*;
pub(super) fn browser(c: &mut Canvas, menu: bool, center: &str, close: bool) {
    icons(c, menu, close);

    let x = (240 - c.width(center, 20)) / 2;
    c.text(x, 290, center, 20, WHITE);
}
pub(super) fn icons(c: &mut Canvas, menu: bool, close: bool) {
    if menu {
        for y in [292, 298, 304] {
            c.rect(8, y, 20, 2, WHITE);
        }
    }
    if close {
        for d in 0..14 {
            c.rect(215 + d, 291 + d, 2, 2, WHITE);
            c.rect(228 - d, 291 + d, 2, 2, WHITE);
        }
    } else {
        c.rect(212, 298, 20, 2, WHITE);
        for d in 0..10 {
            c.rect(212 + d, 298 - d, 2, 2, WHITE);
            c.rect(212 + d, 298 + d, 2, 2, WHITE);
        }
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
