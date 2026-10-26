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
    assert_eq!(p.screen, Screen::Compose);
    assert!(p.mail_ui.pending.is_some());
    p.storage_saved();
    assert_eq!(p.screen, Screen::Mail(mail::MailPage::Result));
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
    assert_eq!(p.organizer.page, O::ContactsMenu);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, O::ContactForm);
    p.text("测试");
    p.organizer.draft[3] = "12345".into();
    press(&mut p, Key::Up); // Back is available on the picture row; text fields show Delete.
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
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    p.text("测试");
    p.organizer.draft[3] = "12345".into();
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        3,
    );
    assert_eq!(p.organizer.page, O::ContactFormMenu);
    press(&mut p, Key::Center);
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
    assert_eq!(p.local_data.reminders.alerts.len(), 1);
    press(&mut p, Key::Center);
    p.check_alarms(date, 420);
    assert!(p.local_data.reminders.alerts.is_empty());
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
        assert_eq!(
            p.screen,
            if crate::menu::phone_disabled(expected) {
                Screen::Menu
            } else {
                expected
            },
            "{}",
            entry.label
        );
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

#[test]
fn snake_native_menu_preview_cancel_pause_and_settings_commit() {
    use snake::SnakeView;
    let mut p = phone();
    p.launch(Screen::Snake);
    assert_eq!(
        p.snake_items(),
        ["新游戏", "关卡", "游戏类型", "最高得分", "声音"]
    );
    p.apply_snake_data(snake::SnakeData {
        level: 8,
        ..Default::default()
    });
    p.index = 1;
    press(&mut p, Key::Center);
    press(&mut p, Key::Left);
    assert_eq!(p.snake_level_preview, 7);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.snake.level, 8);
    assert_eq!(p.index, 1);
    assert!(!p.snake_dirty);
    press(&mut p, Key::Center);
    assert_eq!(p.snake_level_preview, 8);
    for _ in 0..12 {
        press(&mut p, Key::Left);
    }
    assert_eq!(p.snake_level_preview, 1);
    press(&mut p, Key::Center);
    assert_eq!(p.snake_data.level, 1);
    assert!(p.snake_dirty);
    assert_eq!(p.snake_view, SnakeView::Saved);
    press(&mut p, Key::Center);
    assert_eq!(p.snake_view, SnakeView::Menu);
    press(&mut p, Key::Center);
    assert!(p.snake.running);
    press(&mut p, Key::SoftRight);
    assert!(!p.snake.running);
    assert_eq!(p.snake_items()[0], "继续");
    let body = p.snake.body.clone();
    press(&mut p, Key::Center);
    assert_eq!(p.snake.body, body);
    press(&mut p, Key::SoftRight);
    p.index = 3;
    press(&mut p, Key::Center);
    assert_eq!(p.snake_view, SnakeView::Types);
    p.index = 4;
    press(&mut p, Key::Center);
    assert_eq!(p.snake.kind, 4);
    assert_eq!(p.snake.body, body);
    assert_eq!(p.snake_items()[0], "继续");
    assert_eq!(p.index, 3);
    p.index = 5;
    press(&mut p, Key::Center);
    assert_eq!(p.snake_items(), ["关", "开"]);
    press(&mut p, Key::Up);
    press(&mut p, Key::SoftRight);
    assert!(p.snake_data.sound);
    press(&mut p, Key::Center);
    press(&mut p, Key::Up);
    press(&mut p, Key::Center);
    assert!(!p.snake_data.sound);
}

#[test]
fn snake_scores_persist_per_map_and_invalid_settings_preserve_file() {
    let dir = std::env::temp_dir().join(format!("ta1278-snake-{}", std::process::id()));
    let mut data = snake::SnakeData::default();
    for score in [5, 42, 2, 42, 10, 1, 80] {
        data.record(2, score);
    }
    assert_eq!(data.scores[2], [80, 42, 42, 10, 5]);
    assert_eq!(data.scores[0], [0; 5]);
    data.kind = 2;
    data.level = 8;
    data.sound = false;
    crate::storage::save_json(&dir, "snake.json", &data, 4096).unwrap();
    let loaded = crate::storage::load_snake(&dir).unwrap();
    assert_eq!(loaded.scores, data.scores);
    assert_eq!((loaded.level, loaded.kind, loaded.sound), (8, 2, false));
    let mut p = phone();
    p.apply_snake_data(loaded);
    assert_eq!(p.snake_level_preview, 8);
    p.launch(Screen::Snake);
    p.index = 1;
    press(&mut p, Key::Center);
    assert_eq!(p.snake_level_preview, 8);
    press(&mut p, Key::Left);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.snake_data.level, 8);
    press(&mut p, Key::Center);
    press(&mut p, Key::Left);
    press(&mut p, Key::Center);
    crate::storage::save_json(&dir, "snake.json", &p.snake_data, 4096).unwrap();
    let mut reloaded = phone();
    reloaded.apply_snake_data(crate::storage::load_snake(&dir).unwrap());
    reloaded.launch(Screen::Snake);
    reloaded.index = 1;
    press(&mut reloaded, Key::Center);
    assert_eq!(reloaded.snake_level_preview, 7);
    press(&mut reloaded, Key::SoftRight);
    reloaded.index = 0;
    press(&mut reloaded, Key::Center);
    assert_eq!(reloaded.snake.level, 7);
    data.level = 0;
    crate::storage::save_json(&dir, "snake.json", &data, 4096).unwrap();
    let raw = std::fs::read(dir.join("snake.json")).unwrap();
    assert!(crate::storage::load_snake(&dir).is_err());
    assert_eq!(std::fs::read(dir.join("snake.json")).unwrap(), raw);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn alarm_native_form_root_toggle_edit_and_cancel_are_distinct() {
    use organizer::OrganizerPage::*;
    let mut p = phone();
    p.launch(Screen::Organizer(Alarms));
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, AlarmForm);
    assert_eq!(p.organizer.draft[0], "09:00");
    press(&mut p, Key::Digit(1));
    press(&mut p, Key::Digit(8));
    assert_eq!(p.organizer.draft[0], "18:00");
    press(&mut p, Key::Center); // Save, not an edit dialog.
    assert_eq!((p.organizer.page, p.index), (Alarms, 1));
    assert!(p.local_data.alarms[0].enabled);
    press(&mut p, Key::Center);
    assert!(!p.local_data.alarms[0].enabled);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center); // Edit from options.
    press(&mut p, Key::Digit(0));
    press(&mut p, Key::SoftRight);
    assert_eq!(p.local_data.alarms[0].fields[0], "18:00");
    assert!(!p.local_data.alarms[0].enabled);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center); // Single delete has no confirmation.
    assert!(p.local_data.alarms.is_empty());
    assert_eq!((p.organizer.page, p.index), (Alarms, 0));
}

#[test]
fn alarm_day_back_applies_to_draft_snooze_back_cancels_and_form_back_discards() {
    use organizer::OrganizerPage::*;
    let mut p = phone();
    p.launch(Screen::Organizer(Alarms));
    press(&mut p, Key::Center);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, AlarmDays);
    press(&mut p, Key::Center); // Monday
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center); // Wednesday
    press(&mut p, Key::SoftRight);
    assert_eq!(alarm::repeat_mask(&p.organizer.draft[1]), 2 | 8);
    for _ in 0..3 {
        press(&mut p, Key::Down);
    }
    press(&mut p, Key::Center);
    assert_eq!((p.organizer.page, p.index), (AlarmSnooze, 2));
    press(&mut p, Key::Down);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.organizer.draft[4], "10分钟");
    press(&mut p, Key::Center);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.draft[4], "15分钟");
    press(&mut p, Key::SoftRight);
    assert!(p.local_data.alarms.is_empty());
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.draft[1], "不必重复");
    assert_eq!(p.organizer.draft[4], "10分钟");
}

#[test]
fn alarm_inline_name_utf16_limit_and_save_from_options_persist() {
    use organizer::OrganizerPage::*;
    let mut p = phone();
    p.launch(Screen::Organizer(Alarms));
    press(&mut p, Key::Center);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    assert!(p.alarm_name_active());
    p.organizer.alarm.name.clear();
    p.text("012345678901234567😀");
    assert_eq!(p.organizer.alarm.name.units.len(), 20);
    p.text("x");
    assert_eq!(p.organizer.alarm.name.units.len(), 20);
    press(&mut p, Key::SoftRight); // Delete one scalar, not leave form.
    assert_eq!(p.organizer.alarm.name.text(), "012345678901234567");
    press(&mut p, Key::Down); // Form navigation, not multiline editor movement.
    assert_eq!(p.index, 3);
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.organizer.page, AlarmFormMenu);
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.alarms[0].fields[2], "012345678901234567");
    let json = serde_json::to_string(&p.local_data).unwrap();
    let restored: organizer::LocalData = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.alarms[0].fields, p.local_data.alarms[0].fields);
}

#[test]
fn alarm_weekdays_sunday_bit_and_multiple_same_minute_alerts() {
    let mut p = phone();
    for (name, repeat) in [("Monday", "星期一"), ("Sunday", "星期日"), ("Both", "周末")] {
        p.local_data.alarms.push(organizer::Alarm {
            enabled: true,
            fields: ["09:00", repeat, name, "Clock1", "5分钟"]
                .map(str::to_owned)
                .to_vec(),
        });
    }
    let sunday = phone_core::calendar::Date::new(2026, 10, 25).unwrap();
    p.check_alarms(sunday, 540);
    assert_eq!(
        p.local_data
            .reminders
            .alerts
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>(),
        ["Sunday", "Both"]
    );
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    p.check_alarms(sunday, 540);
    assert!(p.local_data.reminders.alerts.is_empty());
    p.check_alarms(sunday.next(1), 540);
    assert_eq!(
        p.local_data.reminders.alerts.front().unwrap().name,
        "Monday"
    );
    assert!(p.local_data.alarms.iter().all(|a| a.enabled));
    for mask in 0..=127 {
        assert_eq!(alarm::repeat_mask(&alarm::repeat_label(mask)), mask);
    }
}

#[test]
fn alarm_snooze_crosses_midnight_without_mutating_interrupted_editor() {
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.recipient = false;
    p.compose.body.insert("unsaved text");
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::Sms,
        index: 4,
    });
    p.local_data.alarms.push(organizer::Alarm {
        enabled: true,
        fields: ["23:58", "一次", "Night", "Clock1", "5 分钟"]
            .map(str::to_owned)
            .to_vec(),
    });
    let date = phone_core::calendar::Date::new(2026, 12, 31).unwrap();
    p.check_alarms(date, 1438);
    p.text("must not leak");
    p.handle(
        Event {
            key: Key::Digit(7),
            phase: Phase::Long,
        },
        1000,
    );
    assert_eq!(p.compose.body.text(), "unsaved text");
    press(&mut p, Key::SoftLeft);
    assert!(p.local_data.reminders.alerts.is_empty());
    assert!(matches!(p.overlay, Some(Overlay::Menu { index: 4, .. })));
    p.check_alarms(date.next(1), 2);
    assert!(p.local_data.reminders.alerts.is_empty());
    p.check_alarms(date.next(1), 3);
    assert_eq!(p.local_data.reminders.alerts.front().unwrap().time, "00:03");
    press(&mut p, Key::End);
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.compose.body.text(), "unsaved text");
    assert!(!p.local_data.alarms[0].enabled);
}

#[test]
fn alarm_capacity_and_delete_all_cancel_preserve_records() {
    use organizer::OrganizerPage::*;
    let mut p = phone();
    for _ in 0..5 {
        p.local_data.alarms.push(organizer::Alarm {
            enabled: false,
            fields: ["09:00", "不必重复", "Test", "Clock1", "10分钟"]
                .map(str::to_owned)
                .to_vec(),
        });
    }
    p.launch(Screen::Organizer(Alarms));
    assert_eq!(p.index, 1);
    press(&mut p, Key::Up);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, Alarms);
    assert!(matches!(p.overlay, Some(Overlay::Notice(_))));
    press(&mut p, Key::SoftRight);
    press(&mut p, Key::Down);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, DeleteAllAlarms);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.local_data.alarms.len(), 5);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert!(p.local_data.alarms.is_empty());
    assert_eq!(p.index, 0);
}

#[test]
fn alarm_time_accepts_intermediate_digits_but_rejects_invalid_save() {
    use organizer::OrganizerPage::*;
    let mut p = phone();
    p.launch(Screen::Organizer(Alarms));
    press(&mut p, Key::Center);
    press(&mut p, Key::Digit(2)); // 09 -> 29, transiently invalid.
    press(&mut p, Key::Center);
    assert!(p.local_data.alarms.is_empty());
    assert_eq!(p.organizer.page, AlarmForm);
    assert!(matches!(p.overlay, Some(Overlay::Notice(_))));
    press(&mut p, Key::SoftRight);
    for d in [3, 5, 9] {
        press(&mut p, Key::Digit(d));
    }
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.alarms[0].fields[0], "23:59");
}

#[test]
fn sms_queue_disk_failure_rolls_back_and_success_waits_for_persistence() {
    let dir = std::env::temp_dir().join(format!("ta1278-queue-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut p = phone();
    let original = Message {
        to: "12345".into(),
        body: "old draft".into(),
        ..Default::default()
    };
    p.mailbox.save(None, original.clone());
    crate::storage::save(&dir, &p.mailbox).unwrap();
    p.go(Screen::Drafts);
    press(&mut p, Key::Center);
    p.text(" changed");
    let edited = p.message();
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    assert!(p.mail_ui.pending.is_some());
    for key in [Key::Center, Key::End, Key::SoftRight, Key::Digit(1)] {
        press(&mut p, key);
    }
    p.text("blocked while writing");
    assert_eq!(p.message(), edited);
    assert_eq!(p.mailbox.outbox.len(), 1);
    // Force a real filesystem error without permissions tricks or changing user data.
    std::fs::create_dir(dir.join("messages.json.bak")).unwrap();
    assert!(crate::storage::save(&dir, &p.mailbox).is_err());
    p.storage_failed();
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.message(), edited);
    assert_eq!(p.mailbox.drafts, std::slice::from_ref(&original));
    assert!(p.mailbox.outbox.is_empty());
    assert!(!p.dirty);
    assert_eq!(crate::storage::load(&dir).unwrap().drafts, [original]);
    std::fs::remove_dir(dir.join("messages.json.bak")).unwrap();
    press(&mut p, Key::Center);
    crate::storage::save(&dir, &p.mailbox).unwrap();
    p.storage_saved();
    assert_eq!(p.screen, Screen::Mail(mail::MailPage::Result));
    for _ in 0..5 {
        press(&mut p, Key::SoftLeft);
    }
    assert_eq!(p.mailbox.outbox, std::slice::from_ref(&edited));
    assert!(!p.dirty);
    let restored = crate::storage::load(&dir).unwrap();
    assert_eq!(restored.outbox, [edited]);
    assert!(restored.drafts.is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn sms_reader_retry_forward_edit_copy_and_delete_cancel() {
    use mail::MailPage::*;
    let mut p = phone();
    let message = Message {
        to: "12345".into(),
        body: "第一行\n第二行\n测试正文".into(),
        ..Default::default()
    };
    p.mailbox.outbox.push(message.clone());
    p.go(Screen::Outbox);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Mail(Read));
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Mail(Result));
    assert_eq!(p.mailbox.outbox, std::slice::from_ref(&message));
    press(&mut p, Key::Center);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.message(), message);
    assert!(p.compose.draft.is_none()); // Explicit edit-as-new never overwrites the queued record.
    p.go(Screen::Outbox);
    press(&mut p, Key::Center);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.compose.to.text(), "");
    assert_eq!(p.compose.body.text(), message.body);
    assert!(p.compose.recipient);
    p.go(Screen::Outbox);
    press(&mut p, Key::Delete);
    assert_eq!(p.screen, Screen::Mail(Delete));
    press(&mut p, Key::SoftRight);
    assert_eq!(p.screen, Screen::Outbox);
    assert_eq!(p.mailbox.outbox, [message]);
    press(&mut p, Key::Center);
    press(&mut p, Key::SoftLeft);
    for _ in 0..3 {
        press(&mut p, Key::Down);
    }
    press(&mut p, Key::Center);
    press(&mut p, Key::SoftRight);
    assert_eq!((p.screen, p.index), (Screen::Mail(Options), 3));
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert_eq!((p.screen, p.index), (Screen::Outbox, 0));
    assert!(p.mailbox.outbox.is_empty());
}

#[test]
fn sms_reader_scroll_clamps_and_returns_to_same_folder_row() {
    let mut p = phone();
    for i in 0..3 {
        p.mailbox.outbox.push(Message {
            to: format!("1234{i}"),
            body: (0..18).map(|i| format!("测试第{i}行\n")).collect(),
            ..Default::default()
        });
    }
    p.go(Screen::Outbox);
    p.index = 2;
    press(&mut p, Key::Center);
    let mut canvas = crate::canvas::Canvas::new().unwrap();
    crate::views::render(&mut canvas, &mut p);
    for _ in 0..100 {
        press(&mut p, Key::Down);
    }
    assert_eq!(p.mail_ui.scroll, p.mail_ui.lines - 7);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.mail_ui.scroll, p.mail_ui.lines - 7);
    press(&mut p, Key::SoftRight);
    assert_eq!((p.screen, p.index), (Screen::Outbox, 2));
    press(&mut p, Key::SoftRight);
    assert_eq!((p.screen, p.index), (Screen::Messages, 3));
}

#[test]
fn sms_missing_recipient_uses_verified_label_and_never_queues() {
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.recipient = false;
    p.text("test");
    press(&mut p, Key::Center);
    assert!(matches!(&p.overlay,Some(Overlay::Notice(s)) if s=="电话号码为空"));
    assert!(p.mailbox.outbox.is_empty());
    assert!(p.mail_ui.pending.is_none());
    assert_eq!(mail::operation_label(0), Some("正在发送…"));
    assert_eq!(mail::operation_label(5), None); // Log-only native branch is not invented UI.
}

#[test]
fn reopening_an_older_long_draft_does_not_silently_drop_the_body() {
    let mut p = phone();
    let message = Message {
        to: "1".repeat(41),
        body: "中".repeat(5000),
        ..Default::default()
    };
    p.mailbox.drafts.push(message.clone());
    p.go(Screen::Drafts);
    press(&mut p, Key::Center);
    assert_eq!(p.message(), message);
    assert_eq!(p.compose.draft, Some(0));
    p.save_draft();
    assert_eq!(p.mailbox.drafts[0], message);
}

#[test]
fn long_sms_recipient_focus_keeps_header_visible_and_only_recipient_caret() {
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.to.insert("12345");
    let mut c = crate::canvas::Canvas::new().unwrap();
    crate::views::render(&mut c, &mut p);
    let header = c.pixels[24 * 240 * 4..91 * 240 * 4].to_vec();
    let recipient_caret = c.caret;
    p.compose
        .body
        .insert(&"这是一段很长的转发正文。\n".repeat(20));
    crate::views::render(&mut c, &mut p);
    assert_eq!(&c.pixels[24 * 240 * 4..91 * 240 * 4], header);
    assert_eq!(c.caret, recipient_caret);
    assert_eq!(
        &c.pixels[(91 * 240) * 4..(91 * 240) * 4 + 4],
        &[48, 48, 48, 255]
    );
    assert!(
        p.editor_lines
            .iter()
            .flat_map(|r| &r.stops)
            .all(|(offset, _)| *offset <= 5)
    );
    p.compose.to.clear();
    crate::views::render(&mut c, &mut p);
    assert_eq!(c.caret.map(|v| v.0), Some(6));
}

#[test]
fn inbox_read_reply_forward_and_delete_are_local_and_persistent() {
    let mut p = phone();
    p.receive_message("12345".into(), "本地收件测试\n第二行".into())
        .unwrap();
    assert!(p.dirty);
    assert_eq!(p.mailbox.unread(), 1);
    p.go(Screen::Inbox);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Mail(mail::MailPage::Read));
    assert_eq!(p.mailbox.unread(), 0);
    press(&mut p, Key::SoftLeft);
    assert_eq!(
        p.mail_actions()
            .iter()
            .map(|a| a.label())
            .collect::<Vec<_>>(),
        ["回复", "转发", "删除"]
    );
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.compose.to.text(), "12345");
    assert!(p.compose.body.text().is_empty());
    assert!(!p.compose.recipient);
    p.compose.body.insert("本地回复");
    p.queue_message();
    assert!(p.mail_ui.pending.is_some());
    assert!(
        p.receive_message("67890".into(), "等待保存".into())
            .is_err()
    );
    assert_eq!(p.mailbox.inbox.len(), 1);
    p.storage_saved();
    assert_eq!(p.mailbox.outbox.len(), 1);
    assert_eq!(p.mailbox.outbox[0].to, "12345");
    p.go(Screen::Inbox);
    press(&mut p, Key::Center);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.compose.body.text(), "本地收件测试\n第二行");
    assert!(p.compose.to.text().is_empty());
    assert!(p.compose.recipient);
    let dir = std::env::temp_dir().join(format!("ta1278-inbox-{}", std::process::id()));
    crate::storage::save(&dir, &p.mailbox).unwrap();
    let stored = crate::storage::load(&dir).unwrap();
    assert_eq!(stored.inbox, p.mailbox.inbox);
    assert_eq!(stored.unread(), 0);
    p.go(Screen::Inbox);
    press(&mut p, Key::Delete);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.mailbox.inbox.len(), 1);
    press(&mut p, Key::Delete);
    press(&mut p, Key::Center);
    assert!(p.mailbox.inbox.is_empty());
    assert_eq!(p.mailbox.outbox.len(), 1);
    crate::storage::save(&dir, &p.mailbox).unwrap();
    assert!(crate::storage::load(&dir).unwrap().inbox.is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn inbox_compatibility_limits_and_background_receive_preserve_editor() {
    let old: Mailbox =
        serde_json::from_str(r#"{"drafts":[],"outbox":[{"to":"12345","body":"old"}]}"#).unwrap();
    assert!(old.inbox.is_empty());
    assert_eq!(old.outbox[0].body, "old");
    let mut p = phone();
    p.go(Screen::Compose);
    p.compose.to.insert("67890");
    p.compose.body.insert("未保存正文");
    p.receive_message("12345".into(), "新信息".into()).unwrap();
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.compose.body.text(), "未保存正文");
    assert_eq!(p.mailbox.unread(), 1);
    assert!(p.receive_message("".into(), "body".into()).is_err());
    assert!(p.receive_message("12345".into(), "x".repeat(4097)).is_err());
    assert_eq!(p.mailbox.inbox.len(), 1);
    p.go(Screen::Inbox);
    press(&mut p, Key::Delete);
    assert_eq!(p.mailbox.unread(), 1); // Delete confirmation does not mark read.
    press(&mut p, Key::SoftRight);
    press(&mut p, Key::SoftRight);
    assert_eq!((p.screen, p.index), (Screen::Messages, 1));
}

#[test]
fn snake_death_keeps_board_records_once_and_only_back_returns() {
    use snake::SnakeView;
    let mut p = phone();
    p.launch(Screen::Snake);
    press(&mut p, Key::Center);
    p.snake.kind = 3; // Mill: the row below the initial body is a wall.
    p.snake.key(Key::Down);
    p.snake.score = 15;
    p.tick(0, 0.0);
    for time in (250..=2000).step_by(250) {
        p.tick(time, 0.25);
    }
    assert_eq!(p.snake_view, SnakeView::GameOver);
    assert!(p.overlay.is_none());
    assert_eq!(p.snake_data.scores[3], [15, 0, 0, 0, 0]);
    let body = p.snake.body.clone();
    press(&mut p, Key::Center);
    p.tick(3000, 1.0);
    assert_eq!(p.snake_view, SnakeView::GameOver);
    assert_eq!(p.snake.body, body);
    assert_eq!(p.snake_data.scores[3], [15, 0, 0, 0, 0]);
    // An unread SMS must not paint its status icon over the full-screen board.
    let mut canvas = crate::canvas::Canvas::new().unwrap();
    crate::views::render(&mut canvas, &mut p);
    let expected = canvas.pixels.clone();
    p.receive_message("12345".into(), "test".into()).unwrap();
    crate::views::render(&mut canvas, &mut p);
    assert_eq!(canvas.pixels, expected);
    let pixel = (71 * 240 + 31) * 4;
    assert_eq!(
        &canvas.pixels[pixel..pixel + 4],
        &crate::theme::rgb565(0x0127)
    );
    press(&mut p, Key::SoftRight);
    assert_eq!(p.snake_view, SnakeView::Menu);
    assert_eq!(p.snake_items()[0], "新游戏");
    assert_eq!(p.index, 0);
}

#[test]
fn contact_native_routes_preserve_edits_and_disable_calling() {
    use organizer::OrganizerPage as P;
    let mut p = phone();
    p.launch(Screen::Contacts);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::Contacts);
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.organizer.page, P::ContactsMenu);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::ContactGroups);
    press(&mut p, Key::SoftRight);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!((p.organizer.page, p.index), (P::ContactForm, 1));
    p.text("测试");
    press(&mut p, Key::Down);
    p.text("人");
    press(&mut p, Key::Down);
    assert_eq!(p.organizer.draft[1], "测试");
    assert_eq!(p.organizer.draft[2], "人");
    p.text("12345");
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.organizer.page, P::ContactFormMenu);
    press(&mut p, Key::SoftRight);
    assert_eq!(
        (p.index, p.organizer.contact.input.text()),
        (3, "12345".into())
    );
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.contacts.len(), 1);
    assert_eq!(p.local_data.contacts[0][3], "12345");
    p.text("测试"); // Search input is separate from the saved record.
    assert_eq!(p.contact_ids(), [0]);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::ContactDetail);
    press(&mut p, Key::Center);
    press(&mut p, Key::Call);
    assert_eq!(p.organizer.page, P::ContactDetail);
    assert_eq!(p.screen, Screen::Contacts);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::ContactForm);
    p.text("discarded");
    let before = p.organizer.contact.input.text();
    press(&mut p, Key::SoftRight);
    assert_eq!(p.organizer.page, P::ContactForm);
    assert!(p.organizer.contact.input.text().len() < before.len());
    press(&mut p, Key::Up);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.local_data.contacts[0][1], "测试");
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Compose);
    assert_eq!(p.compose.to.text(), "12345");
    p.launch(Screen::Contacts);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::DeleteContact);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.local_data.contacts.len(), 1);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert!(p.local_data.contacts.is_empty());
}
#[test]
fn media_picker_preserves_contact_draft_and_cancel_restores_focus() {
    use media_browser::Purpose;
    use organizer::OrganizerPage as P;
    let mut p = phone();
    p.launch(Screen::Contacts);
    p.organizer_form(P::ContactForm, None);
    p.text("测试");
    press(&mut p, Key::Up);
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Media);
    assert_eq!(p.organizer.draft[1], "测试");
    assert_eq!(p.media_ui.purpose, Purpose::ContactPicture);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.index, 0);
    assert_eq!(p.organizer.page, P::ContactForm);
    assert_eq!(p.organizer.draft[0], "");
    press(&mut p, Key::Center);
    p.media_ui.selected = "/card/photo.png".into();
    p.media_ui.viewing = true;
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Media);
    p.media_ui.frame = Some(image::RgbaImage::new(1, 1));
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.draft[0], "/card/photo.png");
    assert_eq!(p.screen, Screen::Contacts);
    assert!(p.local_data.contacts.is_empty());
}
#[test]
fn groups_filter_saved_ids_and_assign_transactionally() {
    use organizer::OrganizerPage as P;
    let mut p = phone();
    p.launch(Screen::Contacts);
    p.organizer.page = P::ContactGroups;
    p.contact_group_name("家人".into());
    p.contact_group_name("家人".into());
    assert_eq!(p.local_data.groups, ["家人"]);
    p.overlay = None;
    p.organizer_form(P::ContactForm, None);
    p.text("甲");
    press(&mut p, Key::Down);
    p.contact_load_field(8);
    press(&mut p, Key::Center);
    assert_eq!(p.contact_choices(), ["无", "家人"]);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.draft[8], "家人");
    assert!(p.local_data.contacts.is_empty());
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.contacts[0][8], "家人");
    p.organizer.contact.group_filter = Some("其他".into());
    assert!(p.contact_ids().is_empty());
    p.organizer.contact.group_filter = Some("家人".into());
    assert_eq!(p.contact_ids(), [0]);
}
#[test]
fn local_settings_validate_time_and_reset_only_preferences() {
    use settings::SettingsPage as S;
    let mut p = phone();
    p.setting_text(0, "问候语".into());
    p.setting_text(1, "0".into());
    assert_eq!(p.preferences.volume, 0);
    p.setting_text(1, "11".into());
    assert_eq!(p.preferences.volume, 0);
    p.overlay = None;
    p.setting_text(2, "-13".into());
    assert_eq!(p.preferences.timezone, 8);
    p.overlay = None;
    p.setting_text(2, "9".into());
    assert_eq!(p.preferences.timezone, 9);
    p.setting_text(3, "17:41".into());
    assert_eq!(p.local_clock().1, 17 * 60 + 41);
    p.setting_text(4, "2028-02-29".into());
    assert_eq!(p.local_clock().0.iso(), "2028-02-29");
    p.setting_text(4, "2027-02-29".into());
    assert_eq!(p.local_clock().0.iso(), "2028-02-29");
    p.overlay = None;
    p.local_data.groups.push("test".into());
    p.launch(Screen::Settings);
    p.settings_open(S::Reset, 1);
    press(&mut p, Key::Center);
    assert_eq!(p.preferences.greeting, "");
    assert_eq!(p.local_data.groups, ["test"]);
    assert!(p.preferences_dirty);
}
#[test]
fn media_capabilities_block_entry_without_losing_caller() {
    let mut p = phone();
    p.features.pictures = false;
    p.open_media(media_browser::Purpose::Pictures);
    assert_eq!(p.screen, Screen::Home);
    assert!(p.overlay.is_some());
    p.features.video = false;
    p.overlay = None;
    p.open_media(media_browser::Purpose::Videos);
    assert_eq!(p.screen, Screen::Home);
    assert!(p.overlay.is_some());
}
#[test]
fn playlist_add_remove_and_restart_keep_virtual_paths() {
    let mut p = phone();
    p.launch(Screen::Music);
    p.set_music_paths(vec![
        "/card/a.wav".into(),
        "/card/b.wav".into(),
        "/card/c.wav".into(),
    ]);
    p.music_view = MusicView::AddPlaylist;
    p.music.index = 2;
    p.playlist_name("收藏".into());
    assert_eq!(p.local_data.playlists[0].paths, ["/card/c.wav"]);
    p.music_action(3);
    p.index = 1;
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.playlists[0].paths.len(), 1);
    p.music.index = 0;
    p.music_action(3);
    p.index = 1;
    press(&mut p, Key::Center);
    let json = serde_json::to_vec(&p.local_data).unwrap();
    p.local_data = serde_json::from_slice(&json).unwrap();
    p.music_view = MusicView::Playlists;
    p.index = 0;
    press(&mut p, Key::Center);
    p.index = 0;
    press(&mut p, Key::Center);
    assert_eq!(p.music.queue, [2, 0]);
    assert_eq!(p.music.index, 2);
    p.music.ended();
    assert_eq!(p.music.index, 0);
    p.music_view = MusicView::Playlist;
    p.index = 1;
    press(&mut p, Key::SoftLeft);
    p.index = 2;
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.playlists[0].paths, ["/card/c.wav"]);
}
#[test]
fn recurring_events_mark_days_and_remind_across_midnight_once() {
    use phone_core::calendar::Date;
    let mut p = phone();
    p.local_data.events.push(organizer::CalendarEvent {
        date: Date::new(2026, 10, 26).unwrap(),
        fields: ["提醒", "00:05", "15分钟前", "", "无", "每周", ""]
            .map(str::to_owned)
            .to_vec(),
    });
    let start = p.local_data.events[0].date;
    assert!(p.local_data.events[0].occurs(start.next(7)));
    assert!(!p.local_data.events[0].occurs(start.next(1)));
    p.check_events(start.next(-1), 23 * 60 + 50);
    assert_eq!(p.local_data.reminders.alerts.len(), 1);
    assert_eq!(p.local_data.reminders.alerts[0].tone, "无");
    p.check_events(start.next(-1), 23 * 60 + 50);
    assert_eq!(p.local_data.reminders.alerts.len(), 1);
    p.check_events(start.next(6), 23 * 60 + 50);
    assert_eq!(p.local_data.reminders.alerts.len(), 2);
    p.organizer.page = organizer::OrganizerPage::Events;
    p.organizer.date = start.next(7);
    assert_eq!(p.organizer_items(), ["提醒"]);
}
#[test]
fn snake_sound_tracks_normal_food_but_not_bonus_score_changes() {
    let mut p = phone();
    p.launch(Screen::Snake);
    p.snake.start();
    p.snake_view = snake::SnakeView::Game;
    p.snake.score = 5;
    p.snake.food = (9, 6);
    p.tick(0, 0.);
    p.tick(250, 0.25);
    assert_eq!(p.snake.score, 6);
    assert!(
        p.cues
            .iter()
            .any(|c| matches!(c, crate::effects::Cue::Food))
    );
    p.cues.clear();
    p.snake.food = (0, 5);
    p.snake.bonus.as_mut().unwrap().position = (10, 6);
    let len = p.snake.body.len();
    p.tick(500, 0.25);
    assert!(p.snake.score > 6);
    assert_eq!(p.snake.body.len(), len);
    assert!(
        !p.cues
            .iter()
            .any(|c| matches!(c, crate::effects::Cue::Food))
    );
}
#[test]
fn alarm_native_ringtone_choice_and_file_cancel_keep_the_form() {
    use organizer::OrganizerPage as P;
    let mut p = phone();
    p.launch(Screen::Organizer(P::Alarms));
    p.organizer_form(P::AlarmForm, None);
    let time = p.organizer.draft[0].clone();
    p.index = 3;
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::AlarmRingtone);
    assert_eq!(p.index, 2);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.draft[3], "Clock 2");
    assert_eq!(p.audio_preview.as_deref(), Some("Clock 2"));
    assert_eq!(p.index, 3);
    assert_eq!(p.organizer.draft[0], time);
    press(&mut p, Key::Center);
    p.index = crate::native_audio::file_index();
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Media);
    assert_eq!(p.media_ui.purpose, media_browser::Purpose::AlarmRingtone);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.organizer.page, P::AlarmForm);
    assert_eq!(p.index, 3);
    assert_eq!(p.organizer.draft[3], "Clock 2");
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.alarms[0].fields[3], "Clock 2");
}
#[test]
fn ringtone_controls_match_between_settings_and_contact_forms() {
    let mut p = phone();
    p.launch(Screen::Settings);
    p.settings_page = settings::SettingsPage::Ringtone;
    p.preferences.ringtone = "Clock1".into();
    p.index = 2;
    let mut c = crate::canvas::Canvas::new().unwrap();
    crate::views::render(&mut c, &mut p);
    let expected = c.pixels.clone();
    p.launch(Screen::Contacts);
    p.organizer_form(organizer::OrganizerPage::ContactForm, None);
    p.organizer.page = organizer::OrganizerPage::ContactRingtone;
    p.organizer.draft[7] = "Clock1".into();
    p.index = 2;
    crate::views::render(&mut c, &mut p);
    assert_eq!(c.pixels, expected);
}

#[test]
fn flashlight_uses_original_power_art_and_colored_shared_bar() {
    let mut p = phone();
    p.launch(Screen::Tools(tools::ToolPage::Flashlight));
    let mut c = crate::canvas::Canvas::new().unwrap();
    crate::views::render(&mut c, &mut p);
    let off = c.pixels.clone();
    let at = |x: usize, y: usize| (y * 240 + x) * 4;
    assert_eq!(&off[at(0, 300)..at(0, 300) + 4], &c.accent(3));
    assert_eq!(&off[at(90, 165)..at(90, 165) + 4], &[0, 0, 0, 255]);
    assert_ne!(&off[at(118, 101)..at(118, 101) + 3], &[0, 0, 0]);
    press(&mut p, Key::Center);
    assert!(p.tools.torch);
    crate::views::render(&mut c, &mut p);
    assert_ne!(
        &off[100 * 240 * 4..155 * 240 * 4],
        &c.pixels[100 * 240 * 4..155 * 240 * 4]
    );
    press(&mut p, Key::Center);
    crate::views::render(&mut c, &mut p);
    assert_eq!(c.pixels, off);
}

#[test]
fn filtered_contact_delete_cannot_target_a_hidden_record() {
    use organizer::OrganizerPage as P;
    let mut p = phone();
    for name in ["Alice", "Bob"] {
        let mut row = vec![String::new(); 9];
        row[1] = name.into();
        row[3] = "12345".into();
        p.local_data.contacts.push(row);
    }
    p.launch(Screen::Contacts);
    p.text("missing");
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.page, P::ContactsMenu);
    assert_eq!(p.local_data.contacts.len(), 2);
    press(&mut p, Key::SoftRight);
    p.organizer.contact.query.clear();
    p.text("bob");
    assert_eq!(p.contact_ids(), [1]);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.selected, 1);
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.contacts.len(), 1);
    assert_eq!(p.local_data.contacts[0][1], "Alice");
}

#[test]
fn contact_fields_use_measured_input_date_and_choice_geometry() {
    use organizer::OrganizerPage as P;
    let mut p = phone();
    p.launch(Screen::Contacts);
    p.organizer_form(P::ContactForm, None);
    let mut c = crate::canvas::Canvas::new().unwrap();
    for (field, x, y) in [(5, 10, 265), (6, 10, 269), (7, 0, 212), (8, 0, 212)] {
        p.contact_load_field(field);
        crate::views::render(&mut c, &mut p);
        let offset = (y * 240 + x) * 4;
        assert_eq!(
            &c.pixels[offset..offset + 4],
            &if field == 6 {
                crate::canvas::MUTED
            } else {
                c.accent(3)
            },
            "field {field}"
        );
    }
}

#[test]
fn birthday_uses_date_digits_without_text_mode_or_invalid_values() {
    let mut p = phone();
    p.launch(Screen::Contacts);
    p.organizer_form(organizer::OrganizerPage::ContactForm, None);
    p.contact_load_field(6);
    assert!(p.editor().is_none());
    press(&mut p, Key::Digit(2));
    assert_eq!(p.organizer.draft[6], "2900-01-01");
    p.organizer.contact.date_digit = 4;
    press(&mut p, Key::Digit(9));
    assert_eq!(p.organizer.draft[6], "2900-01-01");
    press(&mut p, Key::SoftRight);
    assert_eq!(p.organizer.page, organizer::OrganizerPage::Contacts);
    assert!(p.local_data.contacts.is_empty());
}

#[test]
fn browser_initial_field_value_obeys_native_utf16_cap_without_changing_page_on_cancel() {
    for (value, limit, expected) in [
        ("ABCDE", 3, "ABC"),
        ("中文内容", 2, "中文"),
        ("A😀B", 3, "A😀"),
        ("A😀B", 2, "A"),
    ] {
        let mut p = phone();
        p.go(Screen::Browser);
        p.browser_view = BrowserView::Page;
        p.browser = Browser::new(web::parse(
            "https://example.com/",
            &format!("<input value='{value}' maxlength='{limit}'>"),
        ));
        press(&mut p, Key::Center);
        let editor = p.editor().unwrap();
        assert_eq!(editor.text(), expected);
        assert_eq!(editor.cursor, expected.encode_utf16().count());
        p.text("Z");
        assert_eq!(p.editor().unwrap().units.len(), limit);
        press(&mut p, Key::End);
        assert!(
            matches!(&p.browser.page.elements[0], Element::Field { value: saved, .. } if saved == value)
        );
        press(&mut p, Key::Center);
        press(&mut p, Key::Center);
        assert!(
            matches!(&p.browser.page.elements[0], Element::Field { value: saved, .. } if saved == expected)
        );
    }
}

#[test]
fn browser_native_save_actions_use_home_bookmark_page_order_and_persist() {
    let mut p = phone();
    p.go(Screen::Browser);
    p.browser_view = BrowserView::Page;
    p.browser = Browser::new(web::demo());
    assert_eq!(
        p.menu_labels(MenuKind::BrowserSave),
        ["作为主页", "书签", "网页", "图像"]
    );
    assert!(!p.menu_enabled(MenuKind::BrowserSave, 3));
    option(&mut p, MenuKind::BrowserSave, 0);
    assert!(p.browser_dirty && p.browser_data.settings.custom_home);
    assert_eq!(p.browser_data.settings.home_url, "about:demo");
    option(&mut p, MenuKind::BrowserSave, 1);
    option(&mut p, MenuKind::BrowserSave, 1);
    assert_eq!(p.browser_data.bookmarks.len(), 1);
    option(&mut p, MenuKind::BrowserSave, 2);
    let json = serde_json::to_vec(&p.browser_data).unwrap();
    p.browser_data = serde_json::from_slice(&json).unwrap();
    assert_eq!(p.browser_data.offline.len(), 1);
    option(&mut p, MenuKind::BrowserOpen, 0);
    assert_eq!(p.pending_url.as_deref(), Some("about:demo"));
    assert_eq!(p.browser_view, BrowserView::Page);
}
