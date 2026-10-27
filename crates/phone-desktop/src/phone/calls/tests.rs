use super::*;
fn key(p: &mut Phone, key: Key) {
    p.handle(
        Event {
            key,
            phase: Phase::Down,
        },
        0,
    );
}
#[test]
fn dial_editing_offline_failure_and_history_roundtrip() {
    let mut p = Phone::new(Default::default());
    p.wall_ms = Some(1234000);
    p.open_dialer(String::new());
    p.text("+");
    p.text("12025550123");
    p.text("abc");
    p.text("+");
    assert_eq!(p.calls.number.text(), "+12025550123");
    key(&mut p, Key::Left);
    key(&mut p, Key::Delete);
    key(&mut p, Key::Digit(4));
    assert_eq!(p.calls.number.text(), "+12025550143");
    key(&mut p, Key::Call);
    assert!(matches!(&p.overlay,Some(Overlay::Notice(s)) if s.contains("未连接通话服务")));
    assert_eq!(p.calls.records[0].time, 1234);
    assert!(p.records_dirty);
    let json = serde_json::to_vec(&p.records()).unwrap();
    let restored: organizer::LocalData = serde_json::from_slice(&json).unwrap();
    crate::storage::validate_organizer(&restored).unwrap();
    assert_eq!(restored.calls[0].number, "+12025550143");
    assert!(p.messaging.mailbox.outbox.is_empty());
}
#[test]
fn history_delete_cancel_return_and_capacity() {
    let mut p = Phone::new(Default::default());
    p.open_dialer("12345".into());
    for _ in 0..105 {
        p.overlay = None;
        key(&mut p, Key::Call);
    }
    assert_eq!(p.calls.records.len(), LIMIT);
    p.launch(Screen::Tools(tools::ToolPage::Calls));
    key(&mut p, Key::Delete);
    key(&mut p, Key::SoftRight);
    assert_eq!(p.calls.records.len(), LIMIT);
    key(&mut p, Key::Delete);
    key(&mut p, Key::Center);
    assert_eq!(p.calls.records.len(), LIMIT - 1);
    key(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Tools(tools::ToolPage::Dial));
    key(&mut p, Key::SoftLeft);
    key(&mut p, Key::Down);
    key(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Tools(tools::ToolPage::Calls));
    assert_eq!(p.tools.page, tools::ToolPage::Calls);
    p.calls.records[0].number = "invalid".into();
    assert!(crate::storage::validate_organizer(&p.records()).is_err());
}
#[test]
fn old_profile_and_hash_star_dialing_are_supported() {
    let old: organizer::LocalData =
        serde_json::from_str(r#"{"contacts":[],"events":[],"alarms":[]}"#).unwrap();
    assert!(old.calls.is_empty());
    let mut p = Phone::new(Default::default());
    p.open_dialer(String::new());
    key(&mut p, Key::Star);
    key(&mut p, Key::Digit(1));
    key(&mut p, Key::Hash);
    assert_eq!(p.calls.number.text(), "*1#");
    p.editor_mut().unwrap().select_all();
    p.text("9");
    assert_eq!(p.calls.number.text(), "9");
    key(&mut p, Key::SoftRight);
    key(&mut p, Key::Center);
    assert!(p.calls.records.is_empty());
}

#[test]
fn selecting_a_contact_never_falls_back_to_an_unrelated_typed_number() {
    let mut p = Phone::new(Default::default());
    p.contacts.records.push(
        ["", "Alpha", "", "", "", "", "1900-01-01", "预设", "无"]
            .map(str::to_owned)
            .into(),
    );
    p.open_dialer("12345".into());
    key(&mut p, Key::Down);
    key(&mut p, Key::Call);
    assert!(p.calls.records.is_empty());
    p.overlay = None;
    p.contacts.records[0][3] = "67890".into();
    key(&mut p, Key::Call);
    assert_eq!(p.calls.records[0].number, "67890");
    p.overlay = None;
    key(&mut p, Key::Digit(6));
    assert!(p.calls.selected.is_none());
    assert_eq!(p.dial_number(), "123456");
}
