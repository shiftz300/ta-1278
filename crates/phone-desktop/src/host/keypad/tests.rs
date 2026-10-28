use super::*;
use phone_core::input::{Input, Phase};

#[test]
fn every_physical_key_uses_the_drawn_bounds_at_all_window_sizes() {
    let pad = Keypad::new().unwrap();
    for (width, height) in [
        (240, 576),
        (480, 1152),
        (1080, 2100),
        (1920, 1080),
        (300, 900),
    ] {
        let viewport = Viewport::fit(width, height, true);
        for b in BUTTONS {
            let [x, y, w, h] = b.rect;
            let px = viewport.x + (x as f32 + w as f32 / 2.) / WIDTH as f32 * viewport.w;
            let py = viewport.y + (y as f32 + h as f32 / 2.) / HEIGHT as f32 * viewport.h;
            assert_eq!(pad.hit(viewport, px as f64, py as f64), Some(b.key));
        }
        assert_eq!(pad.hit(viewport, -1., -1.), None);
        assert_eq!(
            pad.hit(viewport, viewport.x as f64, viewport.y as f64),
            None
        );
    }
    let view = Viewport::fit(0, 0, true);
    assert!(view.point(0., 0.).is_none());
}
#[test]
fn multiple_fingers_long_press_and_cancel_use_existing_input_semantics() {
    let mut pad = Keypad::new().unwrap();
    let mut input = Input::default();
    let a = pad.press(Pointer::Touch(1), Key::Digit(2)).unwrap();
    let b = pad.press(Pointer::Touch(u64::MAX), Key::Digit(2)).unwrap();
    let mouse = pad.press(Pointer::Mouse, Key::Hash).unwrap();
    assert_ne!(b, mouse);
    assert!(pad.press(Pointer::Touch(1), Key::Digit(3)).is_none());
    assert_eq!(input.press(a, Key::Digit(2), 0).unwrap().phase, Phase::Down);
    assert!(input.press(b, Key::Digit(2), 0).is_none());
    assert_eq!(input.tick(500)[0].phase, Phase::Long);
    assert!(
        input
            .cancel_source(pad.release(Pointer::Touch(1)).unwrap())
            .is_empty()
    );
    assert_eq!(
        input.release(pad.release(Pointer::Touch(u64::MAX)).unwrap())[0].phase,
        Phase::Up
    );
    input.press(mouse, Key::Hash, 600);
    assert_eq!(
        input.cancel_source(pad.release(Pointer::Mouse).unwrap())[0].phase,
        Phase::Cancel
    );
    assert!(input.tick(2000).is_empty());
    assert!(pad.release(Pointer::Touch(1)).is_none());
}
#[test]
fn composed_controls_preserve_the_phone_frame_and_show_pressed_state() {
    let mut runtime = crate::Runtime::new(Default::default(), Default::default()).unwrap();
    let screen = runtime.render().pixels.to_vec();
    let mut pad = Keypad::new().unwrap();
    let frame = pad.compose(&screen).to_vec();
    assert_eq!(&frame[..screen.len()], &screen);
    assert_eq!(frame.len(), (WIDTH * HEIGHT * 4) as usize);
    assert!(frame.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
    pad.press(Pointer::Mouse, Key::Digit(5));
    assert_ne!(pad.compose(&screen), frame);
    pad.clear();
    assert_eq!(pad.compose(&screen), frame);
    if let Ok(path) = std::env::var("TA1278_KEYPAD_CAPTURE") {
        image::save_buffer(path, &frame, WIDTH, HEIGHT, image::ColorType::Rgba8).unwrap();
    }
}

#[test]
fn fullscreen_controls_fill_remaining_height_and_keep_hit_targets_aligned() {
    let pad = Keypad::new().unwrap();
    for (width, height) in [(1440, 3168), (1080, 2400), (720, 1280), (1920, 1080)] {
        let view = Viewport::fullscreen(width, height);
        let lcd = view.lcd_height.unwrap();
        assert_eq!(
            (view.x, view.y, view.w, view.h),
            (0., 0., width as f32, height as f32)
        );
        assert!(lcd * 240. / 320. <= view.w);
        assert!(view.h - lcd >= view.h * 0.29);
        for b in BUTTONS {
            let [x, y, w, h] = b.rect;
            let px = (x as f32 + w as f32 / 2.) / WIDTH as f32 * view.w;
            let py = lcd + (y as f32 + h as f32 / 2. - 320.) / 256. * (view.h - lcd);
            assert_eq!(pad.hit(view, px as f64, py as f64), Some(b.key));
        }
        assert!(
            pad.hit(view, (view.w / 2.) as f64, (lcd - 1.) as f64)
                .is_none()
        );
        assert!(view.point(view.w as f64, 0.).is_none());
        assert!(view.point(0., view.h as f64).is_none());
    }
    assert_eq!(Viewport::fullscreen(1440, 3168).lcd_height, Some(1920.));
    assert!(Viewport::fullscreen(0, 0).point(0., 0.).is_none());
}

#[test]
fn direction_sectors_have_equal_area_and_matching_pressed_feedback() {
    let mut pad = Keypad::new().unwrap();
    let screen = vec![0; 240 * 320 * 4];
    let mut counts = Vec::new();
    for key in [Key::Up, Key::Right, Key::Down, Key::Left] {
        pad.press(Pointer::Touch(1), key);
        let pixels = pad.compose(&screen);
        let mut count = 0;
        for y in 349..421 {
            for x in 87..153 {
                let selected = pad_key(x as f32 + 0.5, y as f32 + 0.5) == Some(key);
                count += usize::from(selected);
                let offset = (y * 240 + x) * 4;
                assert_eq!(pixels[offset..offset + 3] == [91, 111, 123], selected);
            }
        }
        counts.push(count);
        pad.clear();
    }
    // Rasterization of an equal-area continuous map may differ by a few pixels.
    assert!(
        counts.iter().max().unwrap() - counts.iter().min().unwrap() <= 8,
        "{counts:?}"
    );
    for (width, height) in [(1440, 3168), (1080, 2400), (720, 1280)] {
        let view = Viewport::fullscreen(width, height);
        for (x, y, key) in [
            (91., 360., Key::Left),
            (149., 360., Key::Right),
            (91., 410., Key::Left),
            (149., 410., Key::Right),
            (120., 360., Key::Up),
            (120., 410., Key::Down),
            (120., 385., Key::Center),
        ] {
            let px = x / 240. * view.w;
            let lcd = view.lcd_height.unwrap();
            let py = lcd + (y - 320.) / 256. * (view.h - lcd);
            assert_eq!(pad.hit(view, px as f64, py as f64), Some(key));
        }
    }
}
