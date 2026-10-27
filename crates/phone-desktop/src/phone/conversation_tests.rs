use super::*;
use conversation::Popup;
fn press(p: &mut Phone, key: Key) {
    p.handle(
        Event {
            key,
            phase: Phase::Down,
        },
        1000,
    );
}
fn phone() -> Phone {
    Phone::new(Mailbox::default())
}

#[test]
fn conversation_migration_order_and_queue_never_invents_delivery() {
    let mut m:Mailbox=serde_json::from_str(r#"{"drafts":[],"outbox":[{"to":"123","body":"old"}],"inbox":[{"from":"123","body":"reply"}]}"#).unwrap();
    let c = m.thread("123").unwrap();
    assert_eq!((c.items.len(), c.unread, c.time), (2, 1, 0));
    assert!(m.content(c.items[0]).unwrap().unsent);
    m.queue(
        None,
        Message {
            to: "456".into(),
            body: "local".into(),
            time: 50,
            sent: true,
            submission: None,
        },
    )
    .unwrap();
    assert!(!m.outbox[1].sent);
    assert_eq!(m.conversations()[0].peer, "456");
    assert!(m.read_thread("123"));
    assert!(!m.read_thread("123"));
    m.delete_item(c.items[0]);
    assert_eq!(m.outbox[0].to, "456");
    assert_eq!(m.thread("123").unwrap().items.len(), 1);
}

#[test]
fn inline_reply_save_failure_receive_and_restart_preserve_text() {
    let mut p = phone();
    p.receive_message("12345".into(), "first".into()).unwrap();
    p.open_thread("12345".into());
    assert_eq!(p.mailbox.unread(), 0);
    p.text("unfinished reply");
    p.receive_message("67890".into(), "background".into())
        .unwrap();
    assert_eq!(p.compose.body.text(), "unfinished reply");
    assert_eq!(p.mailbox.unread(), 1);
    p.receive_message("12345".into(), "visible".into()).unwrap();
    assert_eq!(p.mailbox.unread(), 1);
    p.storage_saved();
    press(&mut p, Key::Center);
    assert!(p.mail_ui.pending.is_some());
    p.storage_failed();
    assert_eq!(p.screen, Screen::Conversation);
    assert_eq!(p.compose.body.text(), "unfinished reply");
    assert!(p.mailbox.outbox.is_empty());
    press(&mut p, Key::Center);
    p.storage_saved();
    assert_eq!(p.screen, Screen::Mail(mail::MailPage::Result));
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Conversation);
    assert!(p.compose.body.units.is_empty());
    assert!(!p.mailbox.outbox[0].sent);
    // Empty inline composer must not create a recipient-only draft on close.
    p.save_draft();
    assert!(p.mailbox.drafts.is_empty());
    p.text("saved draft");
    p.save_draft();
    let mut restored =
        Phone::new(serde_json::from_str(&serde_json::to_string(&p.mailbox).unwrap()).unwrap());
    restored.open_thread("12345".into());
    assert_eq!(restored.compose.body.text(), "saved draft");
    restored.compose.body.clear();
    restored.save_draft();
    assert!(restored.mailbox.drafts.is_empty());
}

#[test]
fn bubble_number_actions_disabled_calls_and_cancel_keep_reply() {
    let mut p = phone();
    p.receive_message("12345".into(), "call 67890 later".into())
        .unwrap();
    p.open_thread("12345".into());
    p.text("my draft");
    p.compose.body.cursor = 0;
    press(&mut p, Key::Up);
    assert_eq!(p.chat.focus, Some(0));
    press(&mut p, Key::Up);
    assert!(p.chat.number_focus);
    press(&mut p, Key::SoftLeft);
    assert!(p.chat.popup == Popup::Number);
    press(&mut p, Key::Center);
    assert!(p.chat.popup == Popup::Number);
    assert_eq!(p.screen, Screen::Conversation);
    press(&mut p, Key::SoftRight);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.draft[3], "67890");
    press(&mut p, Key::SoftRight);
    assert_eq!(p.screen, Screen::Conversation);
    assert_eq!(p.compose.body.text(), "my draft");
    p.chat.number_focus = false;
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert!(p.chat.popup == Popup::Delete);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.mailbox.inbox.len(), 1);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    assert!(p.compose.to.units.is_empty());
    assert_eq!(p.compose.body.text(), "call 67890 later");
    assert_eq!(p.mailbox.drafts[0].body, "my draft");
    p.open_thread("12345".into());
    p.chat.focus = Some(0);
    press(&mut p, Key::Delete);
    press(&mut p, Key::Center);
    assert!(p.mailbox.inbox.is_empty());
    assert_eq!(p.compose.body.text(), "my draft");
}

#[test]
fn calls_disabled_from_menu_home_shortcut_and_direct_route() {
    let mut p = phone();
    for key in [Key::Call, Key::Digit(1)] {
        press(&mut p, key);
        assert_eq!(p.screen, Screen::Home);
    }
    p.preferences.left_shortcut = 2;
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.screen, Screen::Home);
    p.launch(Screen::Tools(tools::ToolPage::Calls));
    assert_eq!(p.screen, Screen::Home);
    p.go(Screen::Menu);
    p.index = 7;
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Menu);
    assert!(p.overlay.is_none());
    p.go(Screen::Settings);
    p.settings_page = settings::SettingsPage::Shortcut { left: true };
    p.index = 2;
    p.preferences.left_shortcut = 3;
    press(&mut p, Key::Center);
    assert_eq!(p.preferences.left_shortcut, 3);
}

#[test]
fn long_conversation_scroll_and_clipping_preserve_fixed_composer() {
    let mut p = phone();
    p.receive_message("12345".into(), "long line\n".repeat(40))
        .unwrap();
    p.open_thread("12345".into());
    let mut c = crate::canvas::Canvas::new().unwrap();
    crate::views::render(&mut c, &mut p);
    let bottom = c.pixels[220 * 240 * 4..].to_vec();
    press(&mut p, Key::Up);
    crate::views::render(&mut c, &mut p);
    assert_eq!(p.chat.scroll, 0);
    assert!(p.chat.max_scroll > 0);
    press(&mut p, Key::Right);
    crate::views::render(&mut c, &mut p);
    assert_eq!(p.chat.scroll, 48);
    press(&mut p, Key::SoftRight);
    crate::views::render(&mut c, &mut p);
    assert_eq!(&c.pixels[220 * 240 * 4..], bottom.as_slice());
}
