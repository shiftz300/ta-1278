//! State variants for visual review. All values are synthetic; no host services.
use super::audit::frame;
use super::*;
use crate::phone::{organizer::OrganizerPage, settings::SettingsPage, tools::ToolPage};
use phone_core::input::{Event, Key, Phase};
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
fn system_state_variants() {
    let mut p = Phone::new(Default::default());
    p.settings.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Settings);
    for (page, name) in [
        (SettingsPage::Root, "settings-root"),
        (SettingsPage::Personalization, "settings-personalization"),
        (SettingsPage::Keyboard, "settings-keyboard"),
        (SettingsPage::Shortcuts, "settings-shortcuts"),
    ] {
        p.settings.page = page;
        p.index = 0;
        frame(&mut c, &mut p, name);
    }
    p.settings.page = SettingsPage::Shortcut { left: true };
    for (index, name) in [
        (0, "shortcut-first"),
        (9, "shortcut-media"),
        (12, "shortcut-browser"),
    ] {
        p.index = index;
        frame(&mut c, &mut p, name);
    }
    p.settings.first = 12;
    p.index = 12;
    frame(&mut c, &mut p, "shortcut-current-choice");
    p.settings.page = SettingsPage::Theme;
    for i in 0..6 {
        p.settings.preview = i;
        frame(&mut c, &mut p, &format!("theme-{i}"));
    }
    p.settings.page = SettingsPage::Personalization;
    p.index = 3;
    frame(&mut c, &mut p, "theme-cancel");
    p.launch(Screen::Tools(ToolPage::Counter));
    frame(&mut c, &mut p, "counter-empty");
    p.open_dialer("00000000000".into());
    for name in ["Alpha", "Bravo", "Charlie", "Delta"] {
        p.contacts.records.push(
            [
                "",
                name,
                "",
                "00000000000",
                "",
                "",
                "1900-01-01",
                "预设",
                "无",
            ]
            .map(str::to_owned)
            .into(),
        );
    }
    frame(&mut c, &mut p, "dial-synthetic-number");
    assert_eq!(c.caret.unwrap().1, 28);
    key(&mut p, Key::Down);
    frame(&mut c, &mut p, "dial-contact-selected");
    key(&mut p, Key::SoftLeft);
    frame(&mut c, &mut p, "dial-options");
    p.overlay = None;
    p.launch(Screen::Organizer(OrganizerPage::Calendar));
    p.organizer.date = phone_core::calendar::Date::new(2026, 1, 1).unwrap();
    frame(&mut c, &mut p, "calendar-root");
    p.organizer_form(OrganizerPage::EventForm, None);
    for (index, name) in [(4, "calendar-event-ringtone"), (5, "calendar-event-repeat")] {
        p.index = index;
        frame(&mut c, &mut p, name);
    }
    p.launch(Screen::Tools(ToolPage::Stopwatch));
    for (ms, running, name) in [
        (0, false, "stopwatch-root"),
        (1360, true, "stopwatch-running"),
        (15580, true, "stopwatch-lap"),
        (24660, false, "stopwatch-stopped"),
    ] {
        p.tools.stopwatch.elapsed_ms = ms;
        p.tools.stopwatch.running = running;
        if ms >= 15580 {
            p.tools.stopwatch.laps = vec![14980];
        }
        frame(&mut c, &mut p, name);
    }
    p.launch(Screen::Tools(ToolPage::Calculator));
    frame(&mut c, &mut p, "calculator-root");
    key(&mut p, Key::Digit(1));
    key(&mut p, Key::Up);
    key(&mut p, Key::Digit(2));
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "calculator-add");
    key(&mut p, Key::End);
    p.launch(Screen::Tools(ToolPage::Calculator));
    key(&mut p, Key::Digit(3));
    key(&mut p, Key::Right);
    key(&mut p, Key::Digit(4));
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "calculator-chain");
    p.launch(Screen::Tools(ToolPage::Timer));
    p.index = 1;
    p.tools.timer_edit.name.prediction.set_language(None);
    p.tools.timer_edit.name.titlecase = true;
    frame(&mut c, &mut p, "timer-abc");
    p.text("A");
    frame(&mut c, &mut p, "timer-named");
    p.launch(Screen::Tools(ToolPage::Other));
    frame(&mut c, &mut p, "other-root");
    p.index = 2;
    frame(&mut c, &mut p, "other-restored");
    p.launch(Screen::Snake);
    p.gaming.view = crate::phone::snake::SnakeView::Level;
    for level in [7, 8] {
        p.gaming.level_preview = level;
        frame(&mut c, &mut p, &format!("snake-level-{level}"));
    }
    p.gaming.view = crate::phone::snake::SnakeView::Menu;
    p.gaming.model.over = false;
    p.gaming.model.start();
    p.gaming.model.running = false;
    p.index = 0;
    frame(&mut c, &mut p, "snake-pause");
}

#[test]
fn message_state_variants() {
    let mut p = Phone::new(Default::default());
    p.settings.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.go(Screen::Compose);
    p.messaging.compose.recipient = false;
    for (name, text) in [
        ("sms-native-one", "你".to_string()),
        ("sms-numeric-160", "1234567890".repeat(16)),
        ("sms-numeric-161", format!("{}1", "1234567890".repeat(16))),
        (
            "sms-unicode-70",
            format!("呢{}", "1234567890".repeat(7)[..69].to_owned()),
        ),
        ("sms-unicode-71", format!("呢{}", "1234567890".repeat(7))),
        (
            "sms-template-inserted",
            "现在无法通话。我将稍后呼叫。".into(),
        ),
    ] {
        p.messaging.compose.body = Editor::new(&text, 4096);
        p.messaging.compose.body.numeric = true;
        frame(&mut c, &mut p, name);
    }
    for (index, name) in [
        (0, "sms-options"),
        (2, "sms-disabled-row-focus"),
        (4, "sms-save-selected"),
    ] {
        p.overlay = Some(Overlay::Menu {
            kind: MenuKind::Sms,
            index,
        });
        frame(&mut c, &mut p, name);
    }
    p.messaging.compose.body.clear();
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::Sms,
        index: 0,
    });
    frame(&mut c, &mut p, "sms-empty-options");
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::SmsInsert,
        index: 0,
    });
    frame(&mut c, &mut p, "sms-insert-menu");
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::Symbols,
        index: 0,
    });
    frame(&mut c, &mut p, "sms-symbol-picker");
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::Smileys,
        index: 0,
    });
    frame(&mut c, &mut p, "sms-smiley-picker");
    p.overlay = None;
    p.messaging.compose.body = Editor::new("", 4096);
    p.messaging
        .compose
        .body
        .prediction
        .set_language(Some(phone_core::predict::Language::Chinese));
    for name in ["pinyin", "stroke", "title", "lower", "upper", "number"] {
        frame(&mut c, &mut p, &format!("sms-mode-{name}"));
        p.messaging.compose.body.cycle_mode();
    }
    p.messaging
        .compose
        .body
        .prediction
        .set_language(Some(phone_core::predict::Language::Chinese));
    key(&mut p, Key::Digit(6));
    frame(&mut c, &mut p, "sms-pinyin-6");
    key(&mut p, Key::Digit(4));
    frame(&mut c, &mut p, "sms-pinyin-64");
    p.messaging.compose.body = Editor::new("", 4096);
    p.messaging.compose.body.titlecase = true;
    for _ in 0..3 {
        p.messaging.compose.body.digit(2, 0);
    }
    frame(&mut c, &mut p, "sms-multitap-pending");
    p.messaging.compose.body.tick(800);
    frame(&mut c, &mut p, "sms-multitap-committed");
    p.messaging.compose = Compose::default();
    frame(&mut c, &mut p, "sms-new-after-save");
    p.messaging.compose.body = Editor::new(&format!("呢{}", "1234567890".repeat(7)), 4096);
    frame(&mut c, &mut p, "sms-recipient-focus");
    p.messaging.compose.recipient = false;
    frame(&mut c, &mut p, "sms-body-restored");
}

#[test]
fn browser_chrome_variants() {
    let mut p = Phone::new(Default::default());
    p.settings.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Browser);
    p.browsing.data.settings.home_url = "http://cn.bing.com".into();
    p.browsing.data.settings.download_images = false;
    p.browsing.view = BrowserView::Settings;
    for (i, name) in [
        (2, "browser-home-type-focus"),
        (4, "browser-images-off"),
        (5, "browser-font-size-focus"),
        (6, "browser-clear-focus"),
        (7, "browser-restore-focus"),
    ] {
        p.index = i;
        frame(&mut c, &mut p, name);
    }
    p.index = 4;
    p.browsing.data.settings.download_images = true;
    frame(&mut c, &mut p, "browser-images-on");
    p.browsing.data.settings.download_images = false;
    frame(&mut c, &mut p, "browser-images-restored-off");
    for (view, name) in [
        (BrowserView::HomeType, "browser-home-type-picker"),
        (BrowserView::FontSize, "browser-font-size-picker"),
        (BrowserView::Clear, "browser-clear-picker"),
    ] {
        p.browsing.view = view;
        p.index = 0;
        frame(&mut c, &mut p, name);
    }
    p.browsing.view = BrowserView::Page;
    for (kind, index, name) in [
        (MenuKind::BrowserOpen, 0, "browser-open-menu"),
        (MenuKind::BrowserNavigation, 0, "browser-navigation-menu"),
        (
            MenuKind::BrowserNavigation,
            1,
            "browser-disabled-stop-focus",
        ),
        (MenuKind::BrowserNavigation, 2, "browser-forward-focus"),
        (MenuKind::Browser, 6, "browser-page-settings-entry"),
    ] {
        p.overlay = Some(Overlay::Menu { kind, index });
        frame(&mut c, &mut p, name);
    }
    p.overlay = None;
    for (target, value, name) in [
        (TextTarget::Address, "", "browser-empty-url"),
        (
            TextTarget::Address,
            "http://example.com",
            "browser-public-url",
        ),
        (TextTarget::Field(0), "", "browser-tel-editor"),
        (TextTarget::Field(0), "test\na", "browser-textarea-newline"),
    ] {
        let mut editor = Editor::new(value, 512);
        if matches!(target, TextTarget::Field(_)) && value.is_empty() {
            editor
                .prediction
                .set_language(Some(phone_core::predict::Language::Chinese));
        }
        p.overlay = Some(Overlay::Text {
            target,
            editor: Box::new(editor),
        });
        frame(&mut c, &mut p, name);
    }
    p.overlay = None;
    p.browsing.view = BrowserView::Page;
    p.browsing.model.page.url = "http://example.com".into();
    p.browsing.model.page.title.clear();
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::Browser,
        index: 5,
    });
    key(&mut p, Key::Center);
    assert_eq!(p.browsing.view, BrowserView::Details);
    frame(&mut c, &mut p, "browser-page-details");
    key(&mut p, Key::SoftRight);
    assert_eq!(p.browsing.view, BrowserView::Page);
    p.overlay = Some(Overlay::ClosePage);
    frame(&mut c, &mut p, "browser-page-close");
    p.browsing.model.page.elements = vec![Element::Select {
        label: "".into(),
        name: "choice".into(),
        options: ["Open this select menu", "One", "Two", "Three"]
            .into_iter()
            .map(|s| (s.into(), s.into()))
            .collect(),
        selected: vec![true, false, false, false],
        initial: vec![true, false, false, false],
        multiple: false,
        form: 0,
    }];
    for (index, selected, name) in [
        (0, vec![true, false, false, false], "browser-select-open"),
        (
            0,
            vec![false, false, true, false],
            "browser-select-reopened",
        ),
        (
            1,
            vec![false, false, true, false],
            "browser-select-uncommitted-focus",
        ),
    ] {
        p.overlay = Some(Overlay::Select {
            element: 0,
            index,
            selected,
            multiple: false,
        });
        frame(&mut c, &mut p, name);
    }
}

#[test]
fn transfer_and_timer_feedback_variants() {
    use crate::phone::files::FileView;
    let mut p = Phone::new(Default::default());
    p.settings.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.capacity = Some((32 * 1024 * 1024, 30 * 1024 * 1024));
    p.files.writable = true;
    p.files.pending = None;
    p.files.entries = vec![crate::storage::files::Entry {
        name: "Alarm".into(),
        path: "/Alarm".into(),
        directory: true,
        len: 0,
    }];
    p.files.view = FileView::Options;
    p.index = 2;
    key(&mut p, Key::Center);
    assert!(p.files.transfer.is_some());
    assert!(p.files.pending.is_none());
    frame(&mut c, &mut p, "move-destination");
    p.launch(Screen::Tools(ToolPage::Timer));
    p.index = 1;
    p.text("A");
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "timer-started");
    assert_eq!(p.tools.timer_feedback.unwrap().0, "计时开始");
    key(&mut p, Key::Center);
    p.index = 1;
    p.tools.timer.expired = true;
    p.tools.timer.running = false;
    p.tools.timer.remaining_ms = 0;
    p.overlay = Some(Overlay::TimerElapsed);
    frame(&mut c, &mut p, "timer-expired");
    key(&mut p, Key::SoftRight);
    frame(&mut c, &mut p, "timer-alert-stopped");
    assert_eq!(p.tools.page, ToolPage::TimerMenu);
    assert_eq!(p.tools.timer_feedback.unwrap().0, "计时已停止");
    key(&mut p, Key::SoftRight);
    assert_eq!(p.tools.page, ToolPage::Other);
    key(&mut p, Key::Center);
    p.tools.timer.start(60);
    p.tools.page = ToolPage::TimerMenu;
    key(&mut p, Key::Down);
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "timer-stopped");
    assert!(!p.tools.timer.running);
    p.tick(0, 0.0);
    p.tick(2001, 0.0);
    assert!(p.tools.timer_feedback.is_none());
}

#[test]
fn browser_document_variants() {
    // Equal visible text/values for review; these are local HTML fixtures, not
    // captured handset HTTP payloads. No request is sent by this test.
    let mut p = Phone::new(Default::default());
    p.settings.accent = 4;
    p.launch(Screen::Browser);
    p.browsing.data.settings.font_size = 0;
    p.browsing.view = BrowserView::Page;
    let mut c = Canvas::new().unwrap();
    p.browsing.model =
        phone_core::browser::Browser::new(crate::web::parse("http://example.com", ""));
    assert!(p.browsing.model.page.elements.is_empty());
    frame(&mut c, &mut p, "browser-blank-page");
    for (name, url, html, title) in [
        (
            "browser-live-text",
            "http://example.com",
            "<p>This domain is for use in documentation examples without needing permission. This is not a service; avoid use in monitoring environments.</p>",
            "Example Domain",
        ),
        (
            "browser-link-target",
            "http://info.cern.ch/hypertext/WWW/TheProject.html",
            "<h1>World Wide Web</h1><p>The WorldWideWeb (W3) is a wide-area<a href='about:next'>hypermedia</a> information retrieval initiative aiming to give universal access to a large universe of documents.</p><p>Everything there is online about W3 is linked directly or indirectly to this document, including an <a href='about:next'>executive summary</a> of the project, <a href='about:next'>Mailing lists</a>, <a href='about:next'>Policy</a>, November's <a href='about:next'>W3 news</a>, <a href='about:next'>Frequently Asked Questions</a>.</p><a href='about:next'>What's out there?</a>",
            "World Wide Web",
        ),
    ] {
        p.browsing.data.settings.font_size = usize::from(name == "browser-live-text");
        p.browsing.model = phone_core::browser::Browser::new(crate::web::parse(url, html));
        p.browsing.model.page.title = title.into();
        if let Some(i) = p
            .browsing
            .model
            .page
            .elements
            .iter()
            .position(|e| matches!(e, Element::Link { .. }))
        {
            p.browsing.model.focus = i;
        }
        frame(&mut c, &mut p, name);
    }
    p.browsing.data.settings.font_size = 0;
    p.browsing.model = phone_core::browser::Browser::new(crate::web::parse(
        "http://httpbin.org/forms/post",
        include_str!("../../../../scripts/fixtures/browser-form.html"),
    ));
    frame(&mut c, &mut p, "browser-live-form");
    for e in &mut p.browsing.model.page.elements {
        if let Element::Field { name, value, .. } = e
            && name == "custname"
        {
            *value = "test".into();
        }
    }
    frame(&mut c, &mut p, "browser-field-committed");
    p.browsing.model.focus = p
        .browsing
        .model
        .page
        .elements
        .iter()
        .position(|e| matches!(e, Element::Check { value, .. } if value == "medium"))
        .unwrap();
    frame(&mut c, &mut p, "browser-radio-focus");
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "browser-radio-selected");
    p.browsing.model = phone_core::browser::Browser::new(crate::web::parse(
        "https://www.selenium.dev/selenium/web/web-form.html",
        "<h1>Web form</h1>Text input<input name=text>Password<input name=password type=password>Textarea<textarea name=textarea></textarea>Disabled input<input disabled>Readonly input<input readonly value='Readonly input'><br><a href='about:next'>Return to index</a><br>Dropdown (select)<br><select name=dropdown><option>Open this select menu</option><option>One</option><option>Two</option><option>Three</option></select>Dropdown",
    ));
    frame(&mut c, &mut p, "browser-https-form");
    p.browsing.model.focus = p
        .browsing
        .model
        .page
        .elements
        .iter()
        .position(|e| matches!(e, Element::Select { .. }))
        .unwrap();
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "browser-select-open");
    key(&mut p, Key::Down);
    key(&mut p, Key::Down);
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "browser-select-committed");
    key(&mut p, Key::Center);
    frame(&mut c, &mut p, "browser-select-reopened");
    key(&mut p, Key::Down);
    frame(&mut c, &mut p, "browser-select-uncommitted-focus");
    key(&mut p, Key::SoftRight);
    frame(&mut c, &mut p, "browser-select-cancelled");
    assert!(
        matches!(&p.browsing.model.page.elements[p.browsing.model.focus], Element::Select { selected, .. } if selected[2])
    );
}
