use fontdue::{Font, FontSettings, Metrics};
use std::collections::HashMap;
pub const W: u32 = 240;
pub const H: u32 = 320;
pub type Color = [u8; 4];
pub const WHITE: Color = crate::theme::rgb565(0xef9d);
pub const MUTED: Color = crate::theme::rgb565(0x9cd3);

#[derive(Clone)]
pub struct Line {
    pub text: String,
    pub start: usize,
    pub end: usize,
}
pub struct Canvas {
    pub pixels: Vec<u8>,
    fonts: Vec<Font>,
    glyphs: HashMap<(char, u16), (Metrics, Vec<u8>)>,
    pub icons: Vec<image::RgbaImage>,
    sprites: Vec<image::RgbaImage>,
    softkey_icons: [image::RgbaImage; 4],
    pub theme: crate::theme::Theme,
    pub accent: usize,
    pub caret: Option<(i32, i32, i32)>,
}
impl Canvas {
    pub fn new() -> Result<Self, String> {
        let fonts = [
            crate::packed::LATIN,
            crate::packed::HANI1,
            crate::packed::HANI2,
            crate::packed::CJK,
            crate::packed::SYMBOLS,
            crate::packed::EMOJI,
        ]
        .into_iter()
        .map(|packed| {
            let bytes = phone_core::embedded::decode(packed)?;
            Font::from_bytes(bytes, FontSettings::default()).map_err(str::to_owned)
        })
        .collect::<Result<Vec<_>, _>>()?;
        let icons = [
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00436-icon-0006f368.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00439-icon-0006fe40.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00437-icon-0006f7fc.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00441-icon-00070b98.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00429-icon-0006d64c.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00444-icon-00071be0.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00420-icon-00056684.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00442-icon-0007117c.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00441-icon-00070b98.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00418-icon-00055de8.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00448-icon-000733fc.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00435-icon-0006f020.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00434-icon-0006ede0.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00449-icon-000737dc.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00443-icon-00071564.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00419-icon-000562e4.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00432-icon-0006e214.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00430-icon-0006d9f0.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/01340-icon-000ddcd0.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/01341-icon-000de0f4.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00421-icon-00056b70.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00447-icon-00072ddc.png"
            )
            .as_slice(),
            include_bytes!(
                "../../../nokia-ta1278-research/analysis/extracted/png/00292-icon-00049e5c.png"
            )
            .as_slice(),
        ]
        .iter()
        .map(|b| {
            image::load_from_memory(b)
                .map(|i| i.to_rgba8())
                .map_err(|e| e.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
        let theme = crate::theme::Theme::load();
        Ok(Self {
            accent: theme.startup_fallback_accent,
            theme,
            caret: None,
            pixels: vec![0; (W * H * 4) as usize],
            fonts,
            glyphs: HashMap::new(),
            icons,
            softkey_icons: [
                image::load_from_memory(include_bytes!(
                    "../../../nokia-ta1278-research/analysis/extracted/png/00397-icon-00052900.png"
                ))
                .map_err(|e| e.to_string())?
                .to_rgba8(),
                image::load_from_memory(include_bytes!(
                    "../../../nokia-ta1278-research/analysis/extracted/png/00398-icon-00052abc.png"
                ))
                .map_err(|e| e.to_string())?
                .to_rgba8(),
                image::load_from_memory(include_bytes!(
                    "../../../nokia-ta1278-research/analysis/extracted/png/01160-icon-000d0620.png"
                ))
                .map_err(|e| e.to_string())?
                .to_rgba8(),
                image::load_from_memory(include_bytes!(
                    "../../../nokia-ta1278-research/analysis/extracted/png/01151-icon-000cfe9c.png"
                ))
                .map_err(|e| e.to_string())?
                .to_rgba8(),
            ],
            sprites: crate::snake_art::load()?,
        })
    }
    fn glyph(&mut self, ch: char, size: u16) -> &(Metrics, Vec<u8>) {
        self.glyphs.entry((ch, size)).or_insert_with(|| {
            let font = self
                .fonts
                .iter()
                .find(|f| f.lookup_glyph_index(ch) != 0)
                .unwrap_or(&self.fonts[0]);
            font.rasterize(ch, size as f32)
        })
    }
    pub fn width(&mut self, text: &str, size: u16) -> i32 {
        text.chars()
            .map(|c| self.glyph(c, size).0.advance_width)
            .sum::<f32>()
            .ceil() as i32
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        for yy in y.max(0)..(y + h).min(H as i32) {
            for xx in x.max(0)..(x + w).min(W as i32) {
                let i = ((yy * W as i32 + xx) * 4) as usize;
                self.pixels[i..i + 4].copy_from_slice(&c);
            }
        }
    }
    fn blend_into(pixels: &mut [u8], x: i32, y: i32, c: Color, alpha: u8) {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = ((y * W as i32 + x) * 4) as usize;
        let a = alpha as u32 * c[3] as u32 / 255;
        for (j, channel) in c.iter().enumerate().take(3) {
            pixels[i + j] = ((pixels[i + j] as u32 * (255 - a) + *channel as u32 * a) / 255) as u8;
        }
        pixels[i + 3] = 255;
    }
    pub fn text(&mut self, x: i32, y: i32, text: &str, size: u16, c: Color) {
        let mut cursor = x as f32;
        for ch in text.chars() {
            let glyph_size = size;
            self.glyph(ch, glyph_size);
            let (m, bitmap) = &self.glyphs[&(ch, glyph_size)];
            let top = y + glyph_size as i32 - m.height as i32 - m.ymin;
            for yy in 0..m.height {
                for xx in 0..m.width {
                    Self::blend_into(
                        &mut self.pixels,
                        cursor.round() as i32 + m.xmin + xx as i32,
                        top + yy as i32,
                        c,
                        bitmap[yy * m.width + xx],
                    );
                }
            }
            cursor += m.advance_width;
            if cursor > W as f32 {
                break;
            }
        }
    }
    pub fn fit(&mut self, x: i32, y: i32, text: &str, size: u16, c: Color, max: i32) {
        let mut s = String::new();
        for ch in text.chars() {
            let next = format!("{s}{ch}");
            if self.width(&next, size) > max - 10 {
                s.push('…');
                break;
            }
            s.push(ch);
        }
        self.text(x, y, &s, size, c);
    }
    pub fn wrap(&mut self, text: &str, size: u16, width: i32) -> Vec<Line> {
        let mut out = vec![];
        let mut s = String::new();
        let mut start = 0;
        let mut offset = 0;
        for ch in text.chars() {
            if ch == '\n' {
                out.push(Line {
                    text: std::mem::take(&mut s),
                    start,
                    end: offset,
                });
                offset += 1;
                start = offset;
                continue;
            }
            if !s.is_empty() && self.width(&format!("{s}{ch}"), size) > width {
                out.push(Line {
                    text: std::mem::take(&mut s),
                    start,
                    end: offset,
                });
                start = offset;
            }
            s.push(ch);
            offset += ch.len_utf16();
        }
        out.push(Line {
            text: s,
            start,
            end: offset,
        });
        out
    }
    pub fn image(&mut self, image: &image::RgbaImage, x: i32, y: i32) {
        for (dx, dy, pixel) in image.enumerate_pixels() {
            Self::blend_into(&mut self.pixels, x + dx as i32, y + dy as i32, pixel.0, 255);
        }
    }
    /// Clipped nearest-neighbour blit without allocating a scaled image.
    pub fn image_rect(&mut self, image: &image::RgbaImage, dest: [i32; 4], clip: [i32; 4]) {
        let [x, y, w, h] = dest;
        let [cx, cy, cw, ch] = clip;
        if w <= 0 || h <= 0 || image.width() == 0 || image.height() == 0 {
            return;
        }
        for py in y.max(cy).max(0)..(y + h).min(cy + ch).min(H as i32) {
            let sy = ((py - y) as u64 * image.height() as u64 / h as u64) as u32;
            for px in x.max(cx).max(0)..(x + w).min(cx + cw).min(W as i32) {
                let sx = ((px - x) as u64 * image.width() as u64 / w as u64) as u32;
                Self::blend_into(&mut self.pixels, px, py, image.get_pixel(sx, sy).0, 255);
            }
        }
    }
    pub fn icon(&mut self, index: usize, x: i32, y: i32, size: u32) {
        if size == 0 {
            return;
        }
        let img = &self.icons[index];
        let x_ratio = img.width() as f32 / size as f32;
        let y_ratio = img.height() as f32 / size as f32;
        for oy in 0..size {
            for ox in 0..size {
                let sx = (((ox as f32 + 0.5) * x_ratio) as u32).min(img.width() - 1);
                let sy = (((oy as f32 + 0.5) * y_ratio) as u32).min(img.height() - 1);
                let p = img.get_pixel(sx, sy);
                Self::blend_into(&mut self.pixels, x + ox as i32, y + oy as i32, p.0, 255);
            }
        }
    }
    /// Copy a source rectangle at native pixels; no resize or allocation.
    /// The two observed Home actions use their original 26px resources.
    /// Other assignments reuse menu artwork until their Home glyphs are verified.
    pub fn home_shortcut(&mut self, shortcut: usize, x: i32, y: i32) {
        if shortcut == 0 {
            return;
        }
        if matches!(shortcut, 12 | 3) {
            self.softkey_image(usize::from(shortcut == 3), x, y);
        } else {
            let icons = [0, 4, 3, 0, 2, 7, 11, 9, 12, 21, 6, 15, 1, 14, 5, 16, 17, 18];
            if let Some(icon) = icons.get(shortcut) {
                self.icon(*icon, x, y, 26);
                if shortcut == 2 {
                    self.grey(x, y, 26, 26);
                }
            }
        }
    }
    pub fn softkey_symbol(&mut self, back: bool, x: i32, y: i32) {
        self.softkey_image(if back { 3 } else { 2 }, x, y);
    }
    fn softkey_image(&mut self, index: usize, x: i32, y: i32) {
        for (ox, oy, pixel) in self.softkey_icons[index].enumerate_pixels() {
            Self::blend_into(
                &mut self.pixels,
                x + ox as i32,
                y + oy as i32,
                pixel.0,
                pixel[3],
            );
        }
    }
    pub fn sprite(&mut self, kind: crate::snake_art::Sprite, x: i32, y: i32, src: [u32; 4]) {
        let img = &self.sprites[kind as usize];
        let [sx, sy, width, height] = src;
        for oy in 0..height.min(img.height().saturating_sub(sy)) {
            for ox in 0..width.min(img.width().saturating_sub(sx)) {
                let p = img.get_pixel(sx + ox, sy + oy);
                Self::blend_into(&mut self.pixels, x + ox as i32, y + oy as i32, p.0, 255);
            }
        }
    }
    pub fn accent(&self, shade: usize) -> Color {
        self.theme.color(self.accent, shade)
    }
    pub fn grey(&mut self, x: i32, y: i32, w: i32, h: i32) {
        for yy in y.max(0)..(y + h).min(H as i32) {
            for xx in x.max(0)..(x + w).min(W as i32) {
                let i = ((yy * W as i32 + xx) * 4) as usize;
                let v = ((self.pixels[i] as u16
                    + self.pixels[i + 1] as u16
                    + self.pixels[i + 2] as u16)
                    / 5) as u8;
                self.pixels[i..i + 3].fill(v);
            }
        }
    }
    pub fn dim(&mut self) {
        for px in self.pixels.chunks_exact_mut(4) {
            for v in &mut px[..3] {
                *v = (*v as u16 / 5) as u8;
            }
        }
    }
    pub fn background(&mut self) {
        // Handset owner confirmed black. Available wallpaper assets do not prove defaults.
        self.rect(0, 0, W as i32, H as i32, [0, 0, 0, 255]);
    }
    pub fn ring(&mut self, x: i32, y: i32, radius: i32, color: Color) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let d = dx * dx + dy * dy;
                if d <= radius * radius && d >= (radius - 2) * (radius - 2) {
                    self.rect(x + dx, y + dy, 1, 1, color);
                }
            }
        }
    }
    pub fn save(&self, path: &std::path::Path) -> Result<(), image::ImageError> {
        image::save_buffer(path, &self.pixels, W, H, image::ColorType::Rgba8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallback_has_outlines_for_expanded_scripts_and_emoji() {
        let mut c = Canvas::new().unwrap();
        for ch in "龘喆髙𠮷繁體かなカナ한글✓♫★→😀🙂🤔👍ΩЖ".chars() {
            assert!(
                c.fonts.iter().any(|f| f.lookup_glyph_index(ch) != 0),
                "Missing {ch}"
            );
            let (metrics, pixels) = c.glyph(ch, 17);
            assert!(
                metrics.width > 0 && pixels.iter().any(|p| *p > 0),
                "Blank {ch}"
            );
        }
    }
}

#[cfg(test)]
mod font_parity;
