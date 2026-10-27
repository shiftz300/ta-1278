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
    pub layout: crate::ui::layout::Layout,
    pub measuring: bool,
    pub pixels: Vec<u8>,
    clip: [i32; 4],
    fonts: Vec<Font>,
    glyphs: HashMap<(char, u16), (Metrics, Vec<u8>)>,
    pub icons: Vec<image::RgbaImage>,
    pub default_wallpaper: std::sync::Arc<image::RgbaImage>,
    sprites: Vec<image::RgbaImage>,
    symbols: [image::RgbaImage; 11],
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
        ]
        .into_iter()
        .map(|packed| {
            let bytes = phone_core::embedded::decode(packed)?;
            Font::from_bytes(bytes, FontSettings::default()).map_err(str::to_owned)
        })
        .collect::<Result<Vec<_>, _>>()?;
        let icons = [
            crate::assets::png!("00436-icon-0006f368.png"),
            crate::assets::png!("00439-icon-0006fe40.png"),
            crate::assets::png!("00437-icon-0006f7fc.png"),
            crate::assets::png!("00441-icon-00070b98.png"),
            crate::assets::png!("00429-icon-0006d64c.png"),
            crate::assets::png!("00444-icon-00071be0.png"),
            crate::assets::png!("00420-icon-00056684.png"),
            crate::assets::png!("00442-icon-0007117c.png"),
            crate::assets::png!("00441-icon-00070b98.png"),
            crate::assets::png!("00418-icon-00055de8.png"),
            crate::assets::png!("00448-icon-000733fc.png"),
            crate::assets::png!("00435-icon-0006f020.png"),
            crate::assets::png!("00434-icon-0006ede0.png"),
            crate::assets::png!("00449-icon-000737dc.png"),
            crate::assets::png!("00443-icon-00071564.png"),
            crate::assets::png!("00419-icon-000562e4.png"),
            crate::assets::png!("00432-icon-0006e214.png"),
            crate::assets::png!("00430-icon-0006d9f0.png"),
            crate::assets::png!("00421-icon-00056b70.png"),
            crate::assets::png!("00447-icon-00072ddc.png"),
            crate::assets::png!("00292-icon-00049e5c.png"),
        ]
        .into_iter()
        .map(crate::assets::decode)
        .collect::<Result<Vec<_>, _>>()?;
        let theme = crate::theme::Theme::load();
        Ok(Self {
            layout: Default::default(),
            measuring: false,
            accent: theme.startup_fallback_accent,
            theme,
            caret: None,
            clip: [0, 0, W as i32, H as i32],
            pixels: vec![0; (W * H * 4) as usize],
            fonts,
            glyphs: HashMap::new(),
            icons,
            default_wallpaper: image::load_from_memory(include_bytes!(
                "../../../assets/ui/owner-wallpaper.png"
            ))
            .map_err(|e| e.to_string())?
            .to_rgba8()
            .into(),
            symbols: [
                crate::assets::decode(crate::assets::png!("00397-icon-00052900.png"))?,
                crate::assets::decode(crate::assets::png!("00398-icon-00052abc.png"))?,
                crate::assets::decode(crate::assets::png!("01160-icon-000d0620.png"))?,
                crate::assets::decode(crate::assets::png!("01151-icon-000cfe9c.png"))?,
                crate::assets::decode(crate::assets::png!("01152-icon-000cff58.png"))?,
                crate::assets::decode(crate::assets::png!("00455-icon-000765e0.png"))?,
                crate::assets::decode(crate::assets::png!("00542-icon-000b3b3c.png"))?,
                crate::assets::decode(crate::assets::png!("01204-icon-000d44dc.png"))?,
                crate::assets::decode(crate::assets::png!("01294-icon-000da20c.png"))?,
                crate::assets::decode(crate::assets::png!("01295-icon-000da2d0.png"))?,
                crate::assets::decode(crate::assets::png!("01153-icon-000d0028.png"))?,
            ],
            sprites: crate::views::snake::art::load()?,
        })
    }
    fn glyph(&mut self, ch: char, size: u16) -> &(Metrics, Vec<u8>) {
        self.glyphs.entry((ch, size)).or_insert_with(|| {
            let found = self.fonts.iter().find(|f| f.lookup_glyph_index(ch) != 0);
            let shown = if found.is_some() { ch } else { '□' };
            let font = found
                .or_else(|| self.fonts.iter().find(|f| f.lookup_glyph_index('□') != 0))
                .unwrap_or(&self.fonts[0]);
            font.rasterize(shown, size as f32)
        })
    }
    pub fn width(&mut self, text: &str, size: u16) -> i32 {
        text.chars()
            .map(|c| self.glyph(c, size).0.advance_width)
            .sum::<f32>()
            .ceil() as i32
    }
    /// Intersect a temporary content viewport; restores the caller's clip.
    pub fn clipped(&mut self, [x, y, w, h]: [i32; 4], draw: impl FnOnce(&mut Self)) {
        let old = self.clip;
        self.clip = [
            x.max(old[0]),
            y.max(old[1]),
            (x + w).min(old[2]),
            (y + h).min(old[3]),
        ];
        draw(self);
        self.clip = old;
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        if self.measuring {
            return;
        }
        for yy in y.max(self.clip[1]).max(0)..(y + h).min(self.clip[3]).min(H as i32) {
            for xx in x.max(self.clip[0]).max(0)..(x + w).min(self.clip[2]).min(W as i32) {
                let i = ((yy * W as i32 + xx) * 4) as usize;
                self.pixels[i..i + 4].copy_from_slice(&c);
            }
        }
    }
    fn blend_into(pixels: &mut [u8], clip: [i32; 4], x: i32, y: i32, c: Color, alpha: u8) {
        if x < 0
            || y < 0
            || x >= W as i32
            || y >= H as i32
            || x < clip[0]
            || y < clip[1]
            || x >= clip[2]
            || y >= clip[3]
        {
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
        if self.measuring {
            return;
        }
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
                        self.clip,
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
        if max <= 0 {
            return;
        }
        if self.width(text, size) <= max {
            self.text(x, y, text, size, c);
            return;
        }
        let available = max - self.width("…", size);
        if available < 0 {
            return;
        }
        let mut s = String::new();
        for ch in text.chars() {
            let next = format!("{s}{ch}");
            if self.width(&next, size) > available {
                break;
            }
            s.push(ch);
        }
        s.push('…');
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
        if self.measuring {
            return;
        }
        for (dx, dy, pixel) in image.enumerate_pixels() {
            Self::blend_into(
                &mut self.pixels,
                self.clip,
                x + dx as i32,
                y + dy as i32,
                pixel.0,
                255,
            );
        }
    }
    /// Clipped nearest-neighbour blit without allocating a scaled image.
    pub fn image_rect(&mut self, image: &image::RgbaImage, dest: [i32; 4], clip: [i32; 4]) {
        if self.measuring {
            return;
        }
        let [x, y, w, h] = dest;
        let [cx, cy, cw, ch] = clip;
        if w <= 0 || h <= 0 || image.width() == 0 || image.height() == 0 {
            return;
        }
        for py in y.max(cy).max(0)..(y + h).min(cy + ch).min(H as i32) {
            let sy = ((py - y) as u64 * image.height() as u64 / h as u64) as u32;
            for px in x.max(cx).max(0)..(x + w).min(cx + cw).min(W as i32) {
                let sx = ((px - x) as u64 * image.width() as u64 / w as u64) as u32;
                Self::blend_into(
                    &mut self.pixels,
                    self.clip,
                    px,
                    py,
                    image.get_pixel(sx, sy).0,
                    255,
                );
            }
        }
    }
    pub fn icon(&mut self, index: usize, x: i32, y: i32, size: u32) {
        self.icon_enabled(index, x, y, size, true);
    }
    pub fn icon_enabled(&mut self, index: usize, x: i32, y: i32, size: u32, enabled: bool) {
        if self.measuring {
            return;
        }
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
                let mut pixel = img.get_pixel(sx, sy).0;
                if !enabled {
                    let grey = (pixel[..3].iter().map(|&v| u16::from(v)).sum::<u16>() / 5) as u8;
                    pixel[..3].fill(grey);
                }
                Self::blend_into(
                    &mut self.pixels,
                    self.clip,
                    x + ox as i32,
                    y + oy as i32,
                    pixel,
                    255,
                );
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
            self.symbol(
                if shortcut == 3 {
                    crate::assets::Symbol::Messages
                } else {
                    crate::assets::Symbol::Browser
                },
                x,
                y,
            );
        } else {
            let icons = [0, 4, 3, 0, 2, 7, 11, 9, 12, 19, 6, 15, 1, 14, 5, 16, 17];
            if let Some(icon) = icons.get(shortcut) {
                self.icon_enabled(*icon, x, y, 26, true);
            }
        }
    }
    pub fn softkey_symbol(&mut self, back: bool, x: i32, y: i32) {
        self.symbol(
            if back {
                crate::assets::Symbol::Back
            } else {
                crate::assets::Symbol::Menu
            },
            x,
            y,
        );
    }
    pub fn symbol(&mut self, symbol: crate::assets::Symbol, x: i32, y: i32) {
        self.paint_symbol(symbol, x, y, None);
    }
    pub fn tint_symbol(&mut self, symbol: crate::assets::Symbol, x: i32, y: i32, color: Color) {
        self.paint_symbol(symbol, x, y, Some(color));
    }
    fn paint_symbol(
        &mut self,
        symbol: crate::assets::Symbol,
        x: i32,
        y: i32,
        color: Option<Color>,
    ) {
        if self.measuring {
            return;
        }
        for (ox, oy, pixel) in self.symbols[symbol as usize].enumerate_pixels() {
            Self::blend_into(
                &mut self.pixels,
                self.clip,
                x + ox as i32,
                y + oy as i32,
                color.map_or(pixel.0, |mut c| {
                    c[3] = pixel[3];
                    c
                }),
                255,
            );
        }
    }
    pub fn sprite(
        &mut self,
        kind: crate::views::snake::art::Sprite,
        x: i32,
        y: i32,
        src: [u32; 4],
    ) {
        if self.measuring {
            return;
        }
        let img = &self.sprites[kind as usize];
        let [sx, sy, width, height] = src;
        for oy in 0..height.min(img.height().saturating_sub(sy)) {
            for ox in 0..width.min(img.width().saturating_sub(sx)) {
                let p = img.get_pixel(sx + ox, sy + oy);
                Self::blend_into(
                    &mut self.pixels,
                    self.clip,
                    x + ox as i32,
                    y + oy as i32,
                    p.0,
                    255,
                );
            }
        }
    }
    pub fn accent(&self, shade: usize) -> Color {
        self.theme.color(self.accent, shade)
    }
    pub fn dim(&mut self) {
        self.shade(51);
    }
    pub fn shade(&mut self, brightness: u8) {
        if self.measuring {
            return;
        }
        for px in self.pixels.as_chunks_mut::<4>().0 {
            for v in &mut px[..3] {
                *v = (u16::from(*v) * u16::from(brightness) / 255) as u8;
            }
        }
    }
    pub fn background(&mut self) {
        // Application pages stay black; Home/Menu add their shared wallpaper.
        self.rect(0, 0, W as i32, H as i32, [0, 0, 0, 255]);
    }
    pub fn rounded(&mut self, [x, y, w, h]: [i32; 4], radius: i32, color: Color) {
        for dy in 0..h {
            for dx in 0..w {
                let cx = dx.clamp(radius, w - radius - 1);
                let cy = dy.clamp(radius, h - radius - 1);
                if (dx - cx).pow(2) + (dy - cy).pow(2) <= radius.pow(2) {
                    self.rect(x + dx, y + dy, 1, 1, color);
                }
            }
        }
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
    #[cfg(any(feature = "desktop", test))]
    pub fn save(&self, path: &std::path::Path) -> Result<(), image::ImageError> {
        image::save_buffer(path, &self.pixels, W, H, image::ColorType::Rgba8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fitted_button_label_keeps_all_text_when_its_measured_width_fits() {
        let mut c = Canvas::new().unwrap();
        c.background();
        c.text(0, 0, "Submit", 12, WHITE);
        let full = c.pixels.clone();
        let width = c.width("Submit", 12);
        c.background();
        c.fit(0, 0, "Submit", 12, WHITE, width);
        assert_eq!(c.pixels, full);
        c.background();
        c.fit(0, 0, "Submit a much longer label", 12, WHITE, width);
        assert_ne!(c.pixels, full);
        assert!(
            c.pixels
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter(|(i, _)| i % 240 >= width as usize)
                .all(|(_, p)| *p == [0, 0, 0, 255])
        );
    }
    #[test]
    fn fallback_has_outlines_for_expanded_scripts_and_symbols() {
        let mut c = Canvas::new().unwrap();
        for ch in "龘喆髙𠮷繁體かなカナ한글✓♫★→ΩЖ".chars() {
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
    #[test]
    fn unsupported_emoji_and_missing_characters_use_the_square() {
        let mut c = Canvas::new().unwrap();
        for size in [12, 17, 20, 40] {
            let square = c.glyph('□', size).clone();
            assert!(square.1.iter().any(|v| *v > 0));
            for ch in ['😀', '🙂', '🤔', '\u{10ffff}'] {
                assert!(c.fonts.iter().all(|f| f.lookup_glyph_index(ch) == 0));
                assert_eq!(c.glyph(ch, size), &square);
            }
        }
    }
}

#[cfg(test)]
mod parity;
