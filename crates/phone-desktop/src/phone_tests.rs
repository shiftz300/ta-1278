use super::*;
use phone_core::input::{Event, Key, Phase};
fn press(p: &mut Phone, key: Key) {
    p.handle(
        Event {
            key,
            phase: Phase::Down,
        },
        1000,
    );
    p.handle(
        Event {
            key,
            phase: Phase::Up,
        },
        1100,
    );
}
fn phone() -> Phone {
    Phone::new(Mailbox::default())
}
fn option(p: &mut Phone, kind: MenuKind, index: usize) {
    p.overlay = Some(Overlay::Menu { kind, index });
    press(p, Key::Center);
}
#[test]
fn compose_save_reopen_and_queue_is_local() {
    let mut p = phone();
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.screen, Screen::Menu);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    p.text("12345");
    press(&mut p, Key::Center);
    p.text("你好\nHello");
    option(&mut p, MenuKind::Sms, 1);
    assert_eq!(p.mailbox.drafts.len(), 1);
    press(&mut p, Key::Center);
    press(&mut p, Key::End);
    option(&mut p, MenuKind::ExitSms, 0);
    p.go(Screen::Drafts);
    press(&mut p, Key::Center);
    assert_eq!(p.compose.body.text(), "你好\nHello");
    p.text("!");
    option(&mut p, MenuKind::Sms, 0);
    assert_eq!(p.mailbox.outbox.len(), 1);
    assert!(p.mailbox.drafts.is_empty());
    assert!(matches!(&p.overlay,Some(Overlay::Notice(s)) if s.contains("NOT sent")));
}
#[test]
fn browser_field_cancel_and_multi_select_back() {
    let mut p = phone();
    p.go(Screen::Browser);
    let i = p
        .browser
        .page
        .elements
        .iter()
        .position(|e| matches!(e, Element::Field { .. }))
        .unwrap();
    p.browser.focus = i;
    press(&mut p, Key::Center);
    p.editor_mut().unwrap().select_all();
    p.text("changed");
    press(&mut p, Key::SoftRight);
    assert!(matches!(&p.browser.page.elements[i],Element::Field{value,..} if value=="Hello"));
    press(&mut p, Key::Center);
    p.editor_mut().unwrap().select_all();
    p.text("changed");
    press(&mut p, Key::SoftLeft);
    assert!(matches!(&p.browser.page.elements[i],Element::Field{value,..} if value=="changed"));
    let i = p
        .browser
        .page
        .elements
        .iter()
        .position(|e| matches!(e, Element::Select { multiple: true, .. }))
        .unwrap();
    p.browser.focus = i;
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert!(p.overlay.is_some());
    press(&mut p, Key::SoftRight);
    assert!(p.overlay.is_none());
    assert!(matches!(&p.browser.page.elements[i],Element::Select{selected,..} if selected[0]));
    p.browser.reset(0);
    assert_eq!(
        p.browser.form_values(0, None),
        vec![
            ("message".into(), "Hello".into()),
            ("color".into(), "blue".into())
        ]
    );
}
#[test]
fn mouse_routed_softkeys_work_without_region_hit_tests() {
    let mut p = phone();
    for (button, expected) in [
        (winit::event::MouseButton::Left, Screen::Menu),
        (winit::event::MouseButton::Right, Screen::Home),
        (winit::event::MouseButton::Middle, Screen::Menu),
    ] {
        let (_, key) = crate::host_input::mouse(button).unwrap();
        press(&mut p, key);
        assert_eq!(p.screen, expected);
    }
}
#[test]
fn navigation_cancels_seek_without_skipping() {
    let mut p = phone();
    p.go(Screen::Music);
    p.handle(
        Event {
            key: Key::Right,
            phase: Phase::Down,
        },
        0,
    );
    p.handle(
        Event {
            key: Key::Right,
            phase: Phase::Long,
        },
        500,
    );
    p.go(Screen::Home);
    p.go(Screen::Music);
    press(&mut p, Key::Center);
    p.tick(1200, 1.);
    assert_eq!(p.music.index, 0);
    assert_eq!(p.music.position, 6.);
}
