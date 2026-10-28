use super::*;
use fontdue::{Font, FontSettings};
type Color = [u8; 3];
const WHITE: Color = [222, 227, 232];
pub(super) struct Painter {
    font: Font,
}
impl Painter {
    pub fn new() -> Result<Self, String> {
        let bytes = phone_core::embedded::decode(crate::packed::LATIN)?;
        Ok(Self {
            font: Font::from_bytes(bytes, FontSettings::default()).map_err(str::to_owned)?,
        })
    }
    pub fn paint(&self, pixels: &mut [u8], pressed: impl Fn(Key) -> bool) {
        let mut p = Paint {
            pixels,
            font: &self.font,
        };
        p.rect([0, 320, 240, 256], [23, 26, 29], 0);
        p.text_centered(120, 329, "TA-1278", 10., [131, 139, 145]);
        p.rect([85, 347, 70, 76], [115, 123, 126], 11);
        let [px, py, pw, ph] = PAD;
        for y in py..py + ph {
            for x in px..px + pw {
                if let Some(key) = pad_key(x as f32 + 0.5, y as f32 + 0.5) {
                    p.pixel(
                        x,
                        y,
                        if pressed(key) {
                            [91, 111, 123]
                        } else {
                            [39, 44, 49]
                        },
                        255,
                    );
                }
            }
        }
        p.rect([px + pw / 2 - 5, py + ph / 2 - 5, 10, 10], WHITE, 5);
        for b in BUTTONS {
            let [x, y, w, h] = b.rect;
            let cy = y + (h - 1) / 2;
            let down = pressed(b.key);
            let fill = if down { [91, 111, 123] } else { [39, 44, 49] };
            p.rect([x, y, w, h], [10, 12, 14], 6);
            p.rect([x + 1, y + 1, w - 2, h - 3], fill, 5);
            p.rect([x + 6, y + 1, w - 12, 1], [79, 85, 89], 0);
            match b.key {
                Key::SoftLeft | Key::SoftRight => p.rect([x + w / 2 - 10, y + 14, 20, 2], WHITE, 0),
                Key::Call | Key::End => {
                    let y = if b.key == Key::End { y - 3 } else { y };
                    let color = WHITE;
                    let cx = x + w / 2;
                    // Rasterize one half and mirror it: independently rounded
                    // line segments otherwise leave different tips on each side.
                    let points: [(i32, i32); 4] = [(-10, 18), (-9, 12), (-5, 10), (0, 10)];
                    for pair in points.windows(2) {
                        let (sx, sy) = pair[0];
                        let (ex, ey) = pair[1];
                        let n = (ex - sx).abs().max((ey - sy).abs());
                        for i in 0..=n {
                            let dx = sx + (ex - sx) * i / n;
                            let dy = sy + (ey - sy) * i / n;
                            for offset in 0..2 {
                                p.pixel(cx + dx, y + dy + offset, color, 255);
                                p.pixel(cx - dx, y + dy + offset, color, 255);
                            }
                        }
                    }
                    if b.key == Key::End {
                        // A closed circular ring with a separate vertical stroke inside.
                        for dy in -4..=4 {
                            for dx in -4..=4 {
                                if (10..=18).contains(&(dx * dx + dy * dy)) {
                                    p.pixel(cx + dx, y + 21 + dy, color, 255);
                                }
                            }
                        }
                        p.rect([cx, y + 19, 1, 5], color, 0);
                    }
                }
                Key::Digit(1) => {
                    p.text_centered(x + 18, cy, "1", 20., WHITE);
                    p.rect([x + 39, y + 8, 9, 2], WHITE, 0);
                    for xx in [x + 38, x + 44] {
                        p.rect([xx, y + 14, 6, 7], WHITE, 3);
                        p.rect([xx + 2, y + 16, 2, 3], fill, 1);
                    }
                }
                Key::Digit(0) => {
                    p.text_centered(x + 20, cy, "0", 20., WHITE);
                    p.rect([x + 40, y + 14, 10, 2], WHITE, 0);
                    p.rect([x + 40, y + 12, 2, 2], WHITE, 0);
                    p.rect([x + 48, y + 12, 2, 2], WHITE, 0);
                }
                Key::Hash => {
                    p.text_centered(x + 49, cy, "#", 20., WHITE);
                    let y = y - 5;
                    // Printed shift and Chinese-mode legends, not additional keys.
                    p.line(x + 10, y + 19, x + 14, y + 15, WHITE);
                    p.line(x + 14, y + 15, x + 18, y + 19, WHITE);
                    p.rect([x + 12, y + 19, 4, 5], WHITE, 0);
                    p.rect([x + 26, y + 16, 10, 7], WHITE, 0);
                    p.rect([x + 28, y + 18, 6, 3], fill, 0);
                    p.rect([x + 30, y + 13, 2, 13], WHITE, 0);
                }
                Key::Digit(3 | 6 | 9) => {
                    p.text_centered(x + 21, cy + 2, b.letters, 9., WHITE);
                    p.text_centered(x + 49, cy, b.label, 20., WHITE);
                }
                _ => {
                    if b.letters.is_empty() {
                        p.text_centered(x + w / 2, cy, b.label, 20., WHITE);
                    } else {
                        p.text_centered(x + 18, cy, b.label, 20., WHITE);
                        let letters_y = cy + if matches!(b.key, Key::Digit(_)) { 2 } else { 0 };
                        p.text_centered(x + 43, letters_y, b.letters, 9., WHITE);
                    }
                }
            }
        }
    }
}
struct Paint<'a> {
    pixels: &'a mut [u8],
    font: &'a Font,
}
impl Paint<'_> {
    fn pixel(&mut self, x: i32, y: i32, color: Color, alpha: u8) {
        if !(0..WIDTH as i32).contains(&x) || !(320..HEIGHT as i32).contains(&y) {
            return;
        }
        let i = (y as usize * WIDTH as usize + x as usize) * 4;
        for (j, c) in color.into_iter().enumerate() {
            self.pixels[i + j] = ((c as u16 * alpha as u16
                + self.pixels[i + j] as u16 * (255 - alpha as u16))
                / 255) as u8;
        }
        self.pixels[i + 3] = 255;
    }
    fn rect(&mut self, rect: [i32; 4], color: Color, r: i32) {
        let [x, y, w, h] = rect;
        for yy in 0..h {
            for xx in 0..w {
                let dx = (r - xx).max(xx - (w - r - 1)).max(0);
                let dy = (r - yy).max(yy - (h - r - 1)).max(0);
                if dx * dx + dy * dy <= r * r {
                    self.pixel(x + xx, y + yy, color, 255);
                }
            }
        }
    }
    fn line(&mut self, x: i32, y: i32, ex: i32, ey: i32, color: Color) {
        let n = (ex - x).abs().max((ey - y).abs()).max(1);
        for i in 0..=n {
            self.pixel(x + (ex - x) * i / n, y + (ey - y) * i / n, color, 255);
        }
    }
    fn text_centered(&mut self, cx: i32, cy: i32, value: &str, size: f32, color: Color) {
        // Center actual glyph ink, not the font's em box (which includes descent).
        let bounds = value
            .chars()
            .map(|ch| self.font.metrics(ch, size))
            .filter(|m| m.width > 0 && m.height > 0)
            .fold(None, |bounds: Option<(i32, i32)>, m| {
                let top = -m.ymin - m.height as i32;
                let bottom = -m.ymin - 1;
                Some(bounds.map_or((top, bottom), |(a, b)| (a.min(top), b.max(bottom))))
            });
        let Some((top, bottom)) = bounds else {
            return;
        };
        let baseline = cy - (top + bottom) / 2;
        let width: f32 = value
            .chars()
            .map(|c| self.font.metrics(c, size).advance_width)
            .sum();
        let mut x = cx as f32 - width / 2.;
        for ch in value.chars() {
            let (m, data) = self.font.rasterize(ch, size);
            for yy in 0..m.height {
                for xx in 0..m.width {
                    self.pixel(
                        x.round() as i32 + m.xmin + xx as i32,
                        baseline - m.height as i32 - m.ymin + yy as i32,
                        color,
                        data[yy * m.width + xx],
                    );
                }
            }
            x += m.advance_width;
        }
    }
}
