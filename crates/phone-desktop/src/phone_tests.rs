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
    press(&mut p, Key::SoftRight);
    assert_eq!(p.screen, Screen::Messages);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    p.text("12345");
    press(&mut p, Key::Center);
    p.text("你好\nHello");
    option(&mut p, MenuKind::Sms, 4);
    assert_eq!(p.mailbox.drafts.len(), 1);
    assert_eq!(p.screen, Screen::Compose); // Until storage confirms the save.
    assert!(p.pending_compose_close);
    p.storage_saved();
    assert_eq!(p.screen, Screen::Messages);
    p.go(Screen::Drafts);
    press(&mut p, Key::Center);
    assert_eq!(p.compose.body.text(), "你好\nHello");
    p.text("!");
    press(&mut p, Key::Center);
    assert_eq!(p.mailbox.outbox.len(), 1);
    assert!(p.mailbox.drafts.is_empty());
    assert!(matches!(&p.overlay,Some(Overlay::Notice(s)) if s.contains("NOT sent")));
}
#[test]
fn browser_field_cancel_and_multi_select_back() {
    let mut p = phone();
    p.go(Screen::Browser);
    p.browser = Browser::new(web::demo());
    p.browser_view = BrowserView::Page;
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
    press(&mut p, Key::End);
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
    let (_, left) = crate::host_input::mouse(winit::event::MouseButton::Left).unwrap();
    press(&mut p, left);
    assert_eq!(p.screen, Screen::Browser);
    p.go(Screen::Home);
    let (_, right) = crate::host_input::mouse(winit::event::MouseButton::Right).unwrap();
    press(&mut p, right);
    assert_eq!(p.screen, Screen::Messages);
    p.go(Screen::Home);
    let (_, center) = crate::host_input::mouse(winit::event::MouseButton::Middle).unwrap();
    press(&mut p, center);
    assert_eq!(p.screen, Screen::Menu);
}
#[test]
fn navigation_cancels_seek_without_skipping() {
    let mut p = phone();
    p.go(Screen::Music);
    p.music_view = MusicView::Player;
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

#[test]
fn nine_key_confirmation_never_sends_or_closes_field() {
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.recipient = false;
    for d in [6, 4, 4, 2, 6] {
        press(&mut p, Key::Digit(d));
    }
    assert_eq!(p.compose.body.text(), "");
    press(&mut p, Key::Center);
    assert!(p.compose.body.text().is_empty());
    press(&mut p, Key::Center);
    assert_eq!(p.compose.body.text(), "你好");
    assert!(p.overlay.is_none());
    assert!(p.mailbox.outbox.is_empty());
    p.overlay = Some(Overlay::Text {
        target: TextTarget::Address,
        editor: Box::new(Editor::new("", 512)),
    });
    p.editor_mut()
        .unwrap()
        .prediction
        .set_language(Some(phone_core::predict::Language::English));
    for d in [4, 3, 5, 5, 6] {
        press(&mut p, Key::Digit(d));
    }
    press(&mut p, Key::Center);
    assert_eq!(p.editor().unwrap().text(), "hello");
    assert!(p.pending_url.is_none());
    assert!(p.overlay.is_some());
}

#[test]
fn sms_focus_menus_insert_and_save_failure_keep_editor() {
    let mut p = phone();
    p.go(Screen::Compose);
    press(&mut p, Key::SoftLeft);
    assert!(matches!(
        p.overlay,
        Some(Overlay::Menu {
            kind: MenuKind::SmsRecipient,
            ..
        })
    ));
    press(&mut p, Key::SoftRight);
    p.tab();
    press(&mut p, Key::SoftLeft);
    assert_eq!(
        p.menu_labels(MenuKind::Sms),
        vec!["插入选项", "附件", "移除附件", "清除文字", "另存为草稿"]
    );
    press(&mut p, Key::Center);
    option(&mut p, MenuKind::SmsInsert, 2);
    press(&mut p, Key::Center);
    assert_eq!(p.compose.body.text(), "现在无法通话。我将稍后呼叫。");
    option(&mut p, MenuKind::Sms, 4);
    p.storage_failed();
    p.notice("Save failed");
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.compose.body.text(), "现在无法通话。我将稍后呼叫。");
    p.text(" Updated");
    p.storage_saved();
    assert_eq!(p.screen, Screen::Compose);
    assert!(p.compose.body.text().ends_with(" Updated"));
    option(&mut p, MenuKind::Sms, 4);
    p.storage_saved();
    assert_eq!(p.screen, Screen::Messages);
}
#[test]
fn pinyin_column_switch_does_not_commit_or_move_text_cursor() {
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.recipient = false;
    for d in [6, 4] {
        press(&mut p, Key::Digit(d));
    }
    press(&mut p, Key::Left);
    assert!(p.compose.body.prediction.spelling_focus);
    press(&mut p, Key::Down);
    let prediction = &p.compose.body.prediction;
    let spelling = &prediction.spellings[prediction.spelling_selected];
    assert!(
        prediction
            .candidates
            .iter()
            .all(|c| &c.spelling == spelling)
    );
    press(&mut p, Key::Center);
    assert!(!p.compose.body.prediction.spelling_focus);
    assert!(p.compose.body.text().is_empty());
    press(&mut p, Key::Center);
    assert!(!p.compose.body.text().is_empty());
    assert!(p.mailbox.outbox.is_empty());
}

#[test]
fn grid_edges_and_app_return_preserve_focus() {
    let mut p = phone();
    p.go(Screen::Menu);
    assert_eq!(p.index, 4);
    p.index = 17;
    press(&mut p, Key::Right);
    assert_eq!(p.index, 0);
    press(&mut p, Key::Left);
    assert_eq!(p.index, 17);
    p.index = 6;
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Messages);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.screen, Screen::Menu);
    assert_eq!(p.index, 6);
    press(&mut p, Key::SoftRight);
    press(&mut p, Key::Center);
    assert_eq!(p.index, 4);
    p.go(Screen::Compose);
    option(&mut p, MenuKind::SmsInsert, 2);
    press(&mut p, Key::SoftRight);
    assert!(p.overlay.is_none());
}

#[test]
fn theme_preview_cancel_and_shortcut_configuration() {
    use settings::SettingsPage;
    let mut p = phone();
    p.launch(Screen::Settings);
    for key in [
        Key::Down,
        Key::Down,
        Key::Center,
        Key::Down,
        Key::Down,
        Key::Down,
        Key::Center,
    ] {
        press(&mut p, key);
    }
    assert_eq!(p.settings_page, SettingsPage::Theme);
    let initial = p.accent;
    for _ in 0..10 {
        press(&mut p, Key::Right);
    }
    assert_eq!(p.theme_preview, 5);
    assert_eq!(p.accent, initial);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.accent, initial);
    assert!(!p.preferences_dirty);
    press(&mut p, Key::Center);
    press(&mut p, Key::Right);
    press(&mut p, Key::Center);
    assert_eq!(p.accent, (initial + 1).min(5));
    assert!(p.preferences_dirty);
    p.settings_open(SettingsPage::Shortcuts, 1);
    press(&mut p, Key::Center);
    assert_eq!(p.index, 12);
    press(&mut p, Key::Down);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.preferences.left_shortcut, 12);
    press(&mut p, Key::Center);
    p.index = 3;
    press(&mut p, Key::Center);
    p.go(Screen::Home);
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.screen, Screen::Messages);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.screen, Screen::Home);
}

#[test]
fn browser_start_close_page_exit_and_select_focus_are_distinct() {
    let mut p = phone();
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.browser_view, BrowserView::Start);
    press(&mut p, Key::Center);
    assert_eq!(p.editor().unwrap().text(), "http://");
    p.editor_mut().unwrap().select_all();
    p.text("about:demo");
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.pending_url.take().as_deref(), Some("about:demo"));
    let id = p.browser.start();
    p.browser.complete(id, Ok(web::demo()));
    let i = p
        .browser
        .page
        .elements
        .iter()
        .position(|e| {
            matches!(
                e,
                Element::Select {
                    multiple: false,
                    ..
                }
            )
        })
        .unwrap();
    p.browser.focus = i;
    press(&mut p, Key::Center);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert!(matches!(&p.overlay,Some(Overlay::Select{index:0,selected,..}) if selected[1]));
    press(&mut p, Key::SoftRight);
    assert!(matches!(&p.browser.page.elements[i],Element::Select{selected,..} if selected[1]));
    press(&mut p, Key::SoftRight);
    assert!(matches!(p.overlay, Some(Overlay::ClosePage)));
    press(&mut p, Key::Center);
    assert_eq!(p.browser_view, BrowserView::Start);
    assert_eq!(p.screen, Screen::Browser);
    press(&mut p, Key::SoftRight);
    assert!(matches!(p.overlay, Some(Overlay::CloseBrowser)));
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Home);
}

#[test]
fn sms_grids_disabled_focus_and_direct_cancel() {
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.recipient = false;
    press(&mut p, Key::Star);
    for _ in 0..5 {
        press(&mut p, Key::Down);
    }
    for _ in 0..6 {
        press(&mut p, Key::Right);
    }
    press(&mut p, Key::Center);
    assert_eq!(p.compose.body.text(), "\n");
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert!(matches!(
        p.overlay,
        Some(Overlay::Menu {
            kind: MenuKind::Sms,
            index: 2
        })
    ));
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert!(p.compose.body.text().is_empty());
    assert!(p.overlay.is_none());
    option(&mut p, MenuKind::SmsInsert, 0);
    press(&mut p, Key::SoftRight);
    assert!(p.overlay.is_none());
}

#[test]
fn music_library_player_back_and_repeat_cancel() {
    let mut p = phone();
    p.launch(Screen::Music);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.music_view, MusicView::Songs);
    press(&mut p, Key::Center);
    assert!(p.music.playing);
    assert_eq!(p.music_view, MusicView::Player);
    press(&mut p, Key::Center);
    assert!(!p.music.playing);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.music_view, MusicView::Repeat);
    press(&mut p, Key::Down);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.music.repeat, phone_core::music::Repeat::Off);
    press(&mut p, Key::SoftRight);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.music_view, MusicView::Root);
    assert_eq!(p.music_items().len(), 4);
}

#[test]
fn url_cursor_wrap_and_readonly_focus_do_not_edit() {
    let mut p = Phone::new(Mailbox::default());
    p.launch(Screen::Browser);
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        0,
    );
    let end = p.editor().unwrap().units.len();
    p.handle(
        Event {
            key: Key::Right,
            phase: Phase::Down,
        },
        1,
    );
    assert_eq!(p.editor().unwrap().cursor, 0);
    p.handle(
        Event {
            key: Key::Left,
            phase: Phase::Down,
        },
        2,
    );
    assert_eq!(p.editor().unwrap().cursor, end);
    p.overlay = None;
    p.browser_view = BrowserView::Page;
    p.browser = Browser::new(crate::web::parse(
        "https://example.com",
        "<input value='fixed' readonly><input value='editable'>",
    ));
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        3,
    );
    assert!(p.overlay.is_none());
    p.browser.focus = 1;
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        4,
    );
    assert_eq!(p.editor().unwrap().text(), "editable");
}

#[test]
fn organizer_cancel_save_choose_recipient_and_persist() {
    use super::organizer::OrganizerPage as O;
    let mut p = Phone::new(Mailbox::default());
    p.launch(Screen::Contacts);
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        0,
    );
    assert_eq!(p.organizer.page, O::ContactForm);
    p.organizer.draft[1] = "测试".into();
    p.organizer.draft[3] = "12345".into();
    p.handle(
        Event {
            key: Key::SoftRight,
            phase: Phase::Down,
        },
        1,
    );
    assert!(p.local_data.contacts.is_empty());
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        2,
    );
    p.organizer.draft[1] = "测试".into();
    p.organizer.draft[3] = "12345".into();
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        3,
    );
    assert_eq!(p.local_data.contacts.len(), 1);
    assert!(p.local_dirty);
    p.go(Screen::Compose);
    p.compose.body.insert("内容");
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::SmsRecipient,
        index: 0,
    });
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        4,
    );
    assert_eq!(p.screen, Screen::Contacts);
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        5,
    );
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.compose.to.text(), "12345");
    assert_eq!(p.compose.body.text(), "内容");
    let dir = std::env::temp_dir().join(format!("ta1278-organizer-test-{}", std::process::id()));
    crate::storage::save_json(&dir, "organizer.json", &p.local_data, 8 * 1024 * 1024).unwrap();
    assert_eq!(
        crate::storage::load_organizer(&dir).unwrap().contacts[0][3],
        "12345"
    );
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn timer_continues_offscreen_and_browser_clear_cancel_preserves_data() {
    use super::tools::ToolPage as T;
    let mut p = Phone::new(Mailbox::default());
    p.tick(0, 0.);
    p.launch(Screen::Tools(T::Timer));
    p.tools.timer_seconds = 1;
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        0,
    );
    p.go(Screen::Home);
    p.tick(999, 0.);
    assert!(p.overlay.is_none());
    p.tick(1000, 0.);
    assert!(matches!(p.overlay, Some(Overlay::Notice(_))));
    p.launch(Screen::Browser);
    p.browser_data
        .history
        .push(("Test".into(), "https://example.com".into()));
    p.browser_view = BrowserView::Clear;
    p.index = 0;
    assert_eq!(p.browser_items(), vec!["历史", "缓存"]);
    press(&mut p, Key::SoftRight);
    assert_eq!((p.browser_view, p.index), (BrowserView::Settings, 6));
    assert_eq!(p.browser_data.history.len(), 1);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert!(matches!(p.overlay, Some(Overlay::Notice(_))));
    assert_eq!(p.browser_data.history.len(), 1);
}

#[test]
fn local_alarm_once_disables_and_does_not_repeat_same_minute() {
    let mut p = Phone::new(Mailbox::default());
    p.local_data.alarms.push(super::organizer::Alarm {
        enabled: true,
        fields: ["07:00", "一次", "测试闹钟", "默认", "10 分钟"]
            .map(str::to_owned)
            .to_vec(),
    });
    let date = phone_core::calendar::Date::new(2026, 10, 25).unwrap();
    p.check_alarms(date, 419);
    assert!(p.overlay.is_none());
    p.check_alarms(date, 420);
    assert!(!p.local_data.alarms[0].enabled);
    assert!(matches!(p.overlay, Some(Overlay::Notice(_))));
    p.overlay = None;
    p.check_alarms(date, 420);
    assert!(p.overlay.is_none());
}

#[test]
fn all_enabled_menu_entries_have_routes_and_cancel_without_backend_actions() {
    let mut p = Phone::new(Mailbox::default());
    for (i, entry) in crate::menu::ENTRIES.iter().enumerate().take(16) {
        let expected = entry.screen.expect("non-excluded entry needs a route");
        p.go(Screen::Menu);
        p.index = i;
        p.handle(
            Event {
                key: Key::Center,
                phase: Phase::Down,
            },
            0,
        );
        assert_eq!(p.screen, expected, "{}", entry.label);
        p.handle(
            Event {
                key: Key::End,
                phase: Phase::Down,
            },
            1,
        );
        assert_eq!(p.screen, Screen::Home, "{}", entry.label);
    }
    assert!(p.mailbox.outbox.is_empty());
}

#[test]
fn native_browser_settings_toggle_in_place_and_cancel_keeps_choice() {
    let mut p = phone();
    p.launch(Screen::Browser);
    p.browser_view = BrowserView::Settings;
    p.index = 4;
    let original = p.browser_data.settings.download_images;
    press(&mut p, Key::Center);
    assert_eq!(p.browser_view, BrowserView::Settings);
    assert_eq!(p.index, 4);
    assert_eq!(p.browser_data.settings.download_images, !original);
    press(&mut p, Key::Center);
    assert_eq!(p.browser_data.settings.download_images, original);
    p.index = 5;
    p.browser_data.settings.font_size = 0;
    press(&mut p, Key::Center);
    assert_eq!(p.browser_items(), vec!["小号字体", "普通字体"]);
    press(&mut p, Key::Down);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.browser_data.settings.font_size, 0);
    assert_eq!((p.browser_view, p.index), (BrowserView::Settings, 5));
    p.index = 2;
    press(&mut p, Key::Center);
    assert_eq!(p.browser_items(), vec!["自定义"]);
    press(&mut p, Key::SoftRight);
    assert_eq!((p.browser_view, p.index), (BrowserView::Settings, 2));
}

#[test]
fn native_browser_exit_uses_center_not_left_softkey() {
    let mut p = phone();
    p.launch(Screen::Browser);
    p.overlay = Some(Overlay::CloseBrowser);
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.screen, Screen::Browser);
    assert!(matches!(p.overlay, Some(Overlay::CloseBrowser)));
    press(&mut p, Key::Center);
    assert_ne!(p.screen, Screen::Browser);
    assert!(p.overlay.is_none());
}
