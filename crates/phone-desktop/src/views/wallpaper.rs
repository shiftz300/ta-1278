//! Shared Home/Menu background; ordinary app pages keep their black canvas.
use crate::{
    canvas::Canvas,
    phone::{Phone, Screen},
};

pub(super) fn draw(c: &mut Canvas, p: &Phone) {
    if !matches!(p.screen, Screen::Home | Screen::Menu)
        || !p.features.pictures
        || !p.preferences.wallpaper_enabled
    {
        return;
    }
    let builtin = std::sync::Arc::clone(&c.default_wallpaper);
    let image = match &p.preferences.wallpaper {
        Some(path) => p.pictures.get(path),
        None => Some(builtin.as_ref()),
    };
    if let Some(image) = image {
        // Native cache is 232x316, centered at (4,2). Header and footer cover
        // it; do not stretch it to the full LCD or bake UI into the pixels.
        c.image_rect(image, [4, 2, 232, 316], [4, 24, 232, 258]);
        if p.screen == Screen::Menu {
            // Captures measure ~0.58 relative to decoded RGB565, including
            // JPEG/color rounding. 3/5 is a visual approximation, not code proof.
            c.shade(153);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallpaper_is_shared_clipped_and_respects_custom_and_disabled_choices() {
        let mut c = Canvas::new().unwrap();
        let mut p = Phone::new(Default::default());
        c.background();
        draw(&mut c, &p);
        let pixel = |c: &Canvas, x: usize, y: usize| c.pixels[(y * 240 + x) * 4..][..4].to_vec();
        assert_eq!(pixel(&c, 80, 200), c.default_wallpaper.get_pixel(76, 198).0);
        for (x, y) in [(3, 200), (236, 200), (80, 23), (80, 282)] {
            assert_eq!(pixel(&c, x, y), [0, 0, 0, 255]);
        }
        p.screen = Screen::Menu;
        c.background();
        draw(&mut c, &p);
        let source = c.default_wallpaper.get_pixel(76, 198).0;
        assert_eq!(pixel(&c, 80, 200)[0], (u16::from(source[0]) * 3 / 5) as u8);
        p.screen = Screen::Home;
        p.preferences.wallpaper = Some("/card/custom.png".into());
        p.pictures.insert(
            "/card/custom.png".into(),
            image::RgbaImage::from_pixel(1, 1, image::Rgba([123, 45, 67, 255])),
        );
        c.background();
        draw(&mut c, &p);
        assert_eq!(pixel(&c, 80, 200), [123, 45, 67, 255]);
        for screen in [Screen::Home, Screen::Menu, Screen::Compose] {
            p.screen = screen;
            p.features.pictures = screen == Screen::Compose;
            c.background();
            draw(&mut c, &p);
            assert_eq!(pixel(&c, 80, 200), [0, 0, 0, 255]);
        }
        p.screen = Screen::Home;
        p.features.pictures = true;
        p.preferences.wallpaper_enabled = false;
        c.background();
        draw(&mut c, &p);
        assert_eq!(pixel(&c, 80, 200), [0, 0, 0, 255]);
        // Disabled calling must not turn the surrounding wallpaper into a grey square.
        c.rect(0, 0, 240, 320, [20, 80, 40, 255]);
        let size = c.icons[3].width();
        assert_eq!(size, c.icons[3].height());
        c.icon_enabled(3, 0, 0, size, false);
        let mut transparent = 0;
        for (x, y, rgba) in c.icons[3].enumerate_pixels() {
            if rgba[3] == 0 {
                transparent += 1;
                assert_eq!(pixel(&c, x as usize, y as usize), [20, 80, 40, 255]);
            }
        }
        assert!(transparent > 0);
    }
}
