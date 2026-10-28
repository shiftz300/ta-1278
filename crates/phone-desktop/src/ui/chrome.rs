use crate::canvas::{Canvas, WHITE};
pub fn title(c: &mut Canvas, label: &str) {
    // Optical centers share the 24px header, accounting for glyph metrics and PNG padding.
    c.text(6, 4, label, 14, WHITE);
}
pub fn status(c: &mut Canvas) {
    c.text(184, 5, "离线", 12, WHITE);
    // Desktop placeholder: native full-battery artwork, not a hardware reading.
    c.symbol(crate::assets::Symbol::Battery, 210, 4);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_text_and_battery_share_the_same_visible_center() {
        let mut canvas = Canvas::new().unwrap();
        title(&mut canvas, "定时器");
        status(&mut canvas);
        let center = |left: usize, right: usize| {
            let rows: Vec<_> = (0..24)
                .filter(|&y| (left..right).any(|x| canvas.pixels[(y * 240 + x) * 4] > 100))
                .collect();
            (rows[0] + rows[rows.len() - 1]) as i32
        };
        let battery = center(210, 240);
        assert!((center(6, 100) - battery).abs() <= 2);
        assert!((center(184, 210) - battery).abs() <= 2);
    }
}
