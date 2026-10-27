//! Synthetic frame fixtures shared by CPU tests and the desktop presentation check.
use crate::{
    phone::{self, Compose, Screen},
    runtime::Runtime,
    web,
};
use phone_core::input::{Event, Key, Phase};

pub(crate) fn name(stage: usize) -> Option<&'static str> {
    let name = match stage {
        0 => "home",
        1 => "menu",
        2 => "sms",
        3 => "browser",
        4 => "music",
        5 => "prediction-zh",
        6 => "prediction-en",
        7 => "font-coverage",
        8 => "sms-empty",
        9 => "sms-options",
        10 => "theme",
        11 => "sms-long",
        12 => "prediction-page",
        13 => "browser-start",
        14 => "browser-select",
        15 => "sms-symbols",
        16 => "sms-smileys",
        17 => "settings-shortcuts",
        18 => "music-root",
        19 => "music-songs",
        20 => "music-repeat",
        21 => "browser-form",
        22 => "calculator",
        23 => "calendar",
        24 => "contact-form",
        25 => "alarm-form",
        26 => "stopwatch",
        27 => "timer",
        28 => "radio",
        29 => "snake",
        30 => "converter",
        31 => "browser-settings",
        32 => "sms-recipient",
        33 => "browser-native-form",
        34 => "browser-native-links",
        35 => "browser-root-popup",
        36 => "browser-page-popup",
        37 => "browser-font-picker",
        38 => "browser-url-input",
        39 => "sms-empty-recipient",
        40 => "browser-exit",
        41 => "browser-clear-picker",
        42 => "browser-settings-tail",
        43 => "snake-menu",
        44 => "snake-level",
        45 => "snake-types",
        46 => "snake-scores",
        47 => "snake-sound",
        48 => "snake-native-initial",
        49 => "snake-map-box",
        50 => "snake-native-matched",
        51 => "alarm-days",
        52 => "alarm-days-selected",
        53 => "alarm-snooze-picker",
        54 => "alarm-snooze-preview",
        55 => "alarm-snooze-field",
        56 => "alarm-name",
        57 => "alarm-root",
        58 => "alarm-options",
        59 => "alarm-alert",
        60 => "sms-send-unavailable",
        61 => "sms-read",
        62 => "sms-read-scrolled",
        63 => "sms-read-options",
        64 => "sms-forward",
        65 => "sms-outbox",
        66 => "sms-delete-confirm",
        67 => "sms-inbox",
        68 => "sms-incoming-read",
        69 => "sms-incoming-options",
        70 => "sms-reply",
        71 => "sms-conversations",
        72 => "sms-thread",
        73 => "sms-thread-bubble-menu",
        74 => "sms-thread-number-menu",
        75 => "calls-entry",
        76 => "sms-thread-long",
        77 => "snake-game-over",
        78 => "snake-level-persisted",
        79 => "snake-level-saved",
        80 => "snake-type-return",
        81 => "sms-native-one",
        82 => "sms-native-newline",
        83 => "sms-native-wrap",
        84 => "sms-native-scrolled",
        85 => "torch-off",
        86 => "torch-on",
        87 => "contacts-empty",
        88 => "contacts-root-options",
        89 => "contact-new",
        90 => "contact-phone-fields",
        91 => "contact-birthday",
        92 => "contact-ringtone",
        93 => "contact-group",
        94 => "contact-details",
        95 => "contact-options",
        96 => "contact-delete",
        97 => "contacts-list",
        98 => "contacts-search",
        99 => "contact-form-filled",
        100 => "settings-sounds",
        101 => "settings-wallpaper",
        102 => "contact-groups",
        103 => "contact-picture",
        104 => "gallery-volumes",
        105 => "gallery-files",
        106 => "gallery-picture",
        107 => "gallery-zoom",
        108 => "video-preview",
        109 => "settings-time",
        110 => "settings-connections",
        111 => "settings-reset",
        112 => "music-playlists",
        113 => "music-playlist-tracks",
        114 => "settings-ringtones",
        115 => "alarm-ringtones",
        116 => "calendar-ringtones",
        117 => "contact-ringtones",
        118 => "t9-spelling-ni",
        119 => "t9-spelling-mi",
        120 => "t9-next-page",
        121 => "t9-association",
        122 => "t9-context-hao",
        123 => "t9-committed",
        124 => "t9-learning-selection",
        125 => "t9-learning-committed",
        _ => return None,
    };
    Some(name)
}
pub(crate) fn advance(runtime: &mut Runtime, stage: usize, now: u64) {
    match stage {
        0 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        1 => {
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.to.insert("12345");
            runtime
                .phone
                .messaging
                .compose
                .body
                .insert("Hello Nokia!\n短信编辑测试。\n240 × 320");
            runtime.phone.messaging.compose.recipient = false;
            runtime.phone.go(Screen::Compose);
        }
        2 => {
            runtime.phone.go(Screen::Browser);
            runtime.phone.browsing.view = phone::BrowserView::Page;
            runtime.phone.browsing.model = phone_core::browser::Browser::new(web::demo());
        }
        3 => {
            runtime.phone.go(Screen::Music);
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        4 => {
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.recipient = false;
            runtime.phone.messaging.compose.to.insert("12345");
            runtime.phone.messaging.compose.body.insert("明天见！\n");
            runtime.phone.go(Screen::Compose);
            for digit in [6, 4] {
                runtime.dispatch(Event {
                    key: Key::Digit(digit),
                    phase: Phase::Down,
                });
            }
        }
        5 => {
            runtime.phone.messaging.compose.body.clear();
            runtime
                .phone
                .messaging
                .compose
                .body
                .prediction
                .set_language(Some(phone_core::predict::Language::English));
            runtime.phone.messaging.compose.body.insert("Let's go ");
            for digit in [4, 6, 6, 3] {
                runtime.dispatch(Event {
                    key: Key::Digit(digit),
                    phase: Phase::Down,
                });
            }
        }
        6 => {
            runtime.phone.messaging.compose.body.clear();
            runtime
                .phone
                .messaging
                .compose
                .body
                .prediction
                .set_language(None);
            runtime.phone.messaging.compose.body.insert("简体／繁體：你好閱讀\n生僻：龘 喆 髙 𠮷\nかな カタカナ 한글\nSymbols: ✓ ♫ ★ →\nEmoji: 😀 🙂 🤔 👍\nGreek Ω · Cyrillic Ж");
        }
        7 => {
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.recipient = false;
            runtime.phone.go(Screen::Compose);
        }
        8 => runtime.dispatch(Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        }),
        9 => {
            runtime.phone.go(Screen::Settings);
            runtime.phone.settings.page = phone::settings::SettingsPage::Theme;
            runtime.phone.settings.preview = runtime.phone.settings.accent;
        }
        10 => {
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.recipient = false;
            runtime
                .phone
                .messaging
                .compose
                .body
                .insert(&"软换行与光标测试 Hello!\n".repeat(14));
            runtime.phone.go(Screen::Compose);
        }
        11 => {
            runtime.phone.messaging.compose.body.clear();
            for d in [6, 4] {
                runtime.dispatch(Event {
                    key: Key::Digit(d),
                    phase: Phase::Down,
                });
            }
            runtime.dispatch(Event {
                key: Key::Left,
                phase: Phase::Down,
            });
            for _ in 0..5 {
                runtime.dispatch(Event {
                    key: Key::Down,
                    phase: Phase::Down,
                });
            }
        }
        12 => {
            runtime.phone.go(Screen::Browser);
            runtime.phone.browsing.view = phone::BrowserView::Start;
        }
        13 => {
            runtime.phone.browsing.model = phone_core::browser::Browser::new(web::demo());
            runtime.phone.browsing.view = phone::BrowserView::Page;
            let i = runtime
                .phone
                .browsing
                .model
                .page
                .elements
                .iter()
                .position(|e| {
                    matches!(
                        e,
                        phone_core::browser::Element::Select {
                            multiple: false,
                            ..
                        }
                    )
                })
                .unwrap();
            if let phone_core::browser::Element::Select { selected, .. } =
                &mut runtime.phone.browsing.model.page.elements[i]
            {
                selected.fill(false);
                selected[1] = true;
            }
            runtime.phone.browsing.model.focus = i;
            runtime.phone.handle(
                Event {
                    key: Key::Center,
                    phase: Phase::Down,
                },
                now,
            );
        }
        14 => {
            runtime.phone.go(Screen::Compose);
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.recipient = false;
            runtime.phone.overlay = Some(phone::Overlay::Menu {
                kind: phone::MenuKind::Symbols,
                index: 41,
            });
        }
        15 => {
            runtime.phone.overlay = Some(phone::Overlay::Menu {
                kind: phone::MenuKind::Smileys,
                index: 0,
            })
        }
        16 => {
            runtime.phone.go(Screen::Settings);
            runtime.phone.settings.page = phone::settings::SettingsPage::Shortcuts;
        }
        17 => {
            runtime.phone.go(Screen::Music);
            runtime.phone.playback.view = phone::MusicView::Root;
        }
        18 => runtime.phone.playback.view = phone::MusicView::Songs,
        19 => runtime.phone.playback.view = phone::MusicView::Repeat,
        20 => {
            runtime.phone.go(Screen::Browser);
            runtime.phone.browsing.view = phone::BrowserView::Page;
            runtime.phone.browsing.model = phone_core::browser::Browser::new(web::parse(
                "about:fixture",
                r#"<title>Form test</title><form><label>Customer name<input name=name value=test></label><p>Pizza Size</p><input type=radio name=size aria-label=Small><input type=radio name=size aria-label=Medium checked><input type=checkbox name=topping aria-label=Bacon checked><input type=checkbox name=topping aria-label=Cheese checked></form>"#,
            ));
            runtime.phone.browsing.model.focus = 3;
        }
        21 => {
            runtime
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Calculator));
            for key in [
                Key::Digit(1),
                Key::Up,
                Key::Digit(2),
                Key::Center,
                Key::Right,
                Key::Digit(4),
                Key::Center,
            ] {
                runtime.phone.handle(
                    Event {
                        key,
                        phase: Phase::Down,
                    },
                    now,
                );
            }
        }
        22 => runtime
            .phone
            .launch(Screen::Organizer(phone::organizer::OrganizerPage::Calendar)),
        23 => {
            runtime.phone.launch(Screen::Contacts);
            runtime
                .phone
                .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
        }
        24 => {
            runtime
                .phone
                .launch(Screen::Organizer(phone::organizer::OrganizerPage::Alarms));
            runtime
                .phone
                .organizer_form(phone::organizer::OrganizerPage::AlarmForm, None);
        }
        25 => {
            runtime
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Stopwatch));
            runtime.phone.tools.stopwatch.elapsed_ms = 12345;
        }
        26 => runtime
            .phone
            .launch(Screen::Tools(phone::tools::ToolPage::Timer)),
        27 => runtime
            .phone
            .launch(Screen::Tools(phone::tools::ToolPage::Radio)),
        28 => {
            runtime.phone.launch(Screen::Snake);
            runtime.phone.gaming.model.start();
            runtime.phone.gaming.view = phone::snake::SnakeView::Game;
        }
        29 => {
            runtime
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Conversion(0)));
            runtime.phone.tools.conversion = Default::default();
            runtime.phone.text("32");
        }
        30 => {
            runtime.phone.launch(Screen::Browser);
            runtime.phone.browsing.view = phone::BrowserView::Settings;
        }
        31 => {
            runtime.phone.go(Screen::Compose);
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.to.insert("12345");
        }
        32 => {
            runtime.phone.launch(Screen::Browser);
            runtime.phone.browsing.view = phone::BrowserView::Page;
            runtime.phone.browsing.data.settings.font_size = 0;
            runtime.phone.browsing.model = phone_core::browser::Browser::new(web::parse(
                "http://httpbin.org/forms/post",
                include_str!("../../../scripts/fixtures/browser-form.html"),
            ));
            runtime.phone.browsing.model.focus = runtime
                .phone
                .browsing
                .model
                .page
                .elements
                .iter()
                .position(|e| matches!(e, phone_core::browser::Element::Field { .. }))
                .unwrap();
        }
        33 => {
            runtime.phone.browsing.model = phone_core::browser::Browser::new(web::parse(
                "http://info.cern.ch/",
                include_str!("../../../scripts/fixtures/browser-links.html"),
            ));
            runtime.phone.browsing.model.focus = runtime
                .phone
                .browsing
                .model
                .page
                .elements
                .iter()
                .position(|e| matches!(e, phone_core::browser::Element::Link { .. }))
                .unwrap();
        }
        34 => {
            runtime.phone.browsing.view = phone::BrowserView::Start;
            runtime.phone.index = 0;
            runtime.phone.overlay = Some(phone::Overlay::Menu {
                kind: phone::MenuKind::BrowserRoot,
                index: 0,
            });
        }
        35 => {
            runtime.phone.browsing.view = phone::BrowserView::Page;
            runtime.phone.overlay = Some(phone::Overlay::Menu {
                kind: phone::MenuKind::Browser,
                index: 0,
            });
        }
        36 => {
            runtime.phone.overlay = None;
            runtime.phone.browsing.view = phone::BrowserView::FontSize;
            runtime.phone.index = 0;
        }
        37 => {
            runtime.phone.browsing.view = phone::BrowserView::Start;
            runtime.phone.overlay = Some(phone::Overlay::Text {
                target: phone::TextTarget::Address,
                editor: Box::new(phone_core::editor::Editor::new("http://", 512)),
            });
        }
        38 => {
            runtime.phone.go(Screen::Compose);
            runtime.phone.messaging.compose = Compose::default();
        }
        39 => {
            runtime.phone.launch(Screen::Browser);
            runtime.phone.browsing.view = phone::BrowserView::Start;
            runtime.phone.overlay = Some(phone::Overlay::CloseBrowser);
        }
        40 => {
            runtime.phone.overlay = None;
            runtime.phone.browsing.view = phone::BrowserView::Clear;
            runtime.phone.index = 0;
        }
        41 => {
            runtime.phone.browsing.view = phone::BrowserView::Settings;
            runtime.phone.index = 7;
        }
        42 => {
            runtime.phone.launch(Screen::Snake);
            runtime.phone.gaming.model.discard();
            runtime.phone.gaming.view = phone::snake::SnakeView::Menu;
            runtime.phone.index = 0;
        }
        43 => {
            runtime.phone.gaming.view = phone::snake::SnakeView::Level;
            runtime.phone.gaming.level_preview = 7;
        }
        44 => {
            runtime.phone.gaming.view = phone::snake::SnakeView::Types;
            runtime.phone.index = 4;
        }
        45 => {
            runtime.phone.gaming.view = phone::snake::SnakeView::Scores;
            runtime.phone.gaming.data.scores[0] = [42, 30, 20, 10, 1];
            runtime.phone.gaming.model.kind = 0;
        }
        46 => {
            runtime.phone.gaming.view = phone::snake::SnakeView::Sound;
            runtime.phone.gaming.data.sound = false;
            runtime.phone.index = 0;
        }
        47 => {
            runtime.phone.gaming.model.start();
            runtime.phone.gaming.model.running = false;
            runtime.phone.gaming.model.food = (3, 8);
            runtime.phone.gaming.view = phone::snake::SnakeView::Game;
        }
        48 => {
            runtime.phone.gaming.model.kind = 1;
            runtime.phone.gaming.model.score = 123;
        }
        49 => {
            // Match native session149 positions for visual comparison only;
            // food RNG is deliberately not claimed to be the original.
            runtime.phone.gaming.model.kind = 0;
            runtime.phone.gaming.model.start();
            runtime.phone.gaming.model.food = (1, 3);
            runtime
                .phone
                .gaming
                .model
                .tick(runtime.phone.gaming.model.tick_ms());
            runtime.phone.gaming.model.running = false;
        }
        50 => {
            runtime
                .phone
                .launch(Screen::Organizer(phone::organizer::OrganizerPage::Alarms));
            runtime
                .phone
                .organizer_form(phone::organizer::OrganizerPage::AlarmForm, None);
            for key in [Key::Down, Key::Center] {
                runtime.dispatch(Event {
                    key,
                    phase: Phase::Down,
                });
            }
        }
        51 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        52 => {
            for key in [Key::SoftRight, Key::Down, Key::Down, Key::Down, Key::Center] {
                runtime.dispatch(Event {
                    key,
                    phase: Phase::Down,
                });
            }
        }
        53 => runtime.dispatch(Event {
            key: Key::Down,
            phase: Phase::Down,
        }),
        54 => runtime.dispatch(Event {
            key: Key::SoftRight,
            phase: Phase::Down,
        }),
        55 => {
            for key in [Key::Up, Key::Up] {
                runtime.dispatch(Event {
                    key,
                    phase: Phase::Down,
                });
            }
        }
        56 => {
            for key in [Key::Up, Key::Up, Key::Center] {
                runtime.dispatch(Event {
                    key,
                    phase: Phase::Down,
                });
            }
        }
        57 => runtime.dispatch(Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        }),
        58 => {
            // Local alert illustration, not a captured native interruption.
            runtime.phone.organizer.alarms[0].fields[1] = "不必重复".into();
            runtime
                .phone
                .check_alarms(phone_core::calendar::Date::default(), 540);
        }
        59 => {
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.to.insert("12345");
            runtime.phone.messaging.compose.body.insert("本地测试信息。\n这条信息不会发送。\n\n第一段正文\n第二段正文\n第三段正文\n第四段正文\n第五段正文\n第六段正文\n最后一行。");
            runtime.phone.messaging.compose.recipient = false;
            runtime.phone.go(Screen::Compose);
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        60 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        61 => {
            for _ in 0..6 {
                runtime.dispatch(Event {
                    key: Key::Down,
                    phase: Phase::Down,
                });
            }
        }
        62 => runtime.dispatch(Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        }),
        63 => {
            for key in [Key::Down, Key::Down, Key::Center] {
                runtime.dispatch(Event {
                    key,
                    phase: Phase::Down,
                });
            }
        }
        64 => runtime.phone.go(Screen::Outbox),
        65 => runtime.dispatch(Event {
            key: Key::Delete,
            phase: Phase::Down,
        }),
        66 => {
            runtime
                .phone
                .receive_message(
                    "12345".into(),
                    "这是一条本地测试信息。\n没有连接短信服务。\n回复、转发和删除均在本机完成。"
                        .into(),
                )
                .unwrap();
            runtime.phone.go(Screen::Inbox);
        }
        67 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        68 => runtime.dispatch(Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        }),
        69 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        70 => {
            runtime.phone.messaging.mailbox = Default::default();
            runtime
                .phone
                .messaging
                .mailbox
                .outbox
                .push(phone_core::sms::Message {
                    to: "12345".into(),
                    body: "明天见。".into(),
                    time: 1_792_944_000_000,
                    sent: true,
                    submission: None,
                });
            runtime
                .phone
                .receive_message("12345".into(), "好的，联系 67890。".into())
                .unwrap();
            runtime.phone.messaging.mailbox.inbox[0].time = 1_792_944_060_000;
            runtime.phone.messaging.chat = Default::default();
            runtime.phone.go(Screen::Messages);
        }
        71 => runtime.phone.open_thread("12345".into()),
        72 => {
            runtime.phone.messaging.chat.focus = Some(1);
            runtime.phone.messaging.chat.popup = phone::conversation::Popup::Bubble;
        }
        73 => {
            runtime.phone.messaging.chat.number_focus = true;
            runtime.phone.messaging.chat.popup = phone::conversation::Popup::Number;
        }
        74 => {
            runtime.phone.messaging.chat.popup = phone::conversation::Popup::None;
            runtime.phone.go(Screen::Menu);
            runtime.phone.index = 7;
        }
        75 => {
            runtime
                .phone
                .receive_message("12345".into(), "长信息滚动测试。\n".repeat(30))
                .unwrap();
            runtime.phone.open_thread("12345".into());
            runtime.phone.messaging.chat.focus = Some(2);
            runtime.phone.messaging.chat.reveal = true;
        }
        76 => {
            runtime.phone.launch(Screen::Snake);
            runtime.phone.gaming.model.kind = 3;
            runtime.phone.gaming.model.start();
            runtime.phone.gaming.model.food = (4, 14);
            runtime.phone.gaming.model.key(Key::Down);
            runtime.phone.gaming.model.tick(1000);
            runtime.phone.gaming.model.running = false;
            runtime.phone.gaming.view = phone::snake::SnakeView::GameOver;
        }
        77 => {
            runtime.phone.apply_snake_data(phone::snake::SnakeData {
                level: 8,
                ..Default::default()
            });
            runtime.phone.gaming.view = phone::snake::SnakeView::Menu;
            runtime.phone.index = 1;
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        78 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        79 => {
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            runtime.dispatch(Event {
                key: Key::SoftRight,
                phase: Phase::Down,
            });
            runtime.phone.index = 3;
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            runtime.phone.index = 3;
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        80 => {
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.recipient = false;
            runtime
                .phone
                .messaging
                .compose
                .body
                .prediction
                .set_language(None);
            runtime.phone.messaging.compose.body.titlecase = true;
            runtime.phone.messaging.compose.body.insert("你");
            runtime.phone.go(Screen::Compose);
        }
        81 => {
            runtime.phone.messaging.compose.body.insert("\n");
        }
        82 => {
            runtime.phone.messaging.compose.body.clear();
            runtime.phone.messaging.compose.body.titlecase = false;
            runtime.phone.messaging.compose.body.numeric = true;
            runtime
                .phone
                .messaging
                .compose
                .body
                .insert(&format!("呢{}123456789", "1234567890".repeat(6)));
        }
        83 => {
            runtime.phone.messaging.compose.body.clear();
            runtime
                .phone
                .messaging
                .compose
                .body
                .insert(&format!("{}1", "1234567890".repeat(16)));
        }
        84 => {
            runtime
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Flashlight));
            runtime.phone.tools.torch = false;
        }
        85 => runtime.dispatch(Event {
            key: Key::Center,
            phase: Phase::Down,
        }),
        86 => {
            runtime.phone.tools.torch = false;
            runtime.phone.contacts.records.clear();
            runtime.phone.contacts.ui = Default::default();
            runtime.phone.launch(Screen::Contacts);
        }
        87 => runtime.dispatch(Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        }),
        88 => {
            runtime.phone.index = 1;
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        89 => runtime.phone.contact_load_field(5),
        90 => runtime.phone.contact_load_field(6),
        91 => runtime.phone.contact_load_field(7),
        92 => runtime.phone.contact_load_field(8),
        93 => {
            // Verified test-mode fixture from the reviewed handset capture.
            runtime.phone.contacts.records.push(
                [
                    "",
                    "a0000",
                    "",
                    "1230000",
                    "",
                    "",
                    "1900-01-01",
                    "预设",
                    "无",
                ]
                .map(str::to_owned)
                .to_vec(),
            );
            runtime.phone.organizer.selected = 0;
            runtime.phone.organizer.page = phone::organizer::OrganizerPage::ContactDetail;
            runtime.phone.index = 0;
        }
        94 => runtime.dispatch(Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        }),
        95 => {
            runtime.phone.index = 1;
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        96 => runtime.phone.launch(Screen::Contacts),
        97 => {
            runtime.phone.contacts.ui.query.insert("no-match");
        }
        98 => {
            runtime
                .phone
                .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
            runtime.phone.text("测试联系人");
        }
        99 => {
            runtime.phone.launch(Screen::Settings);
            runtime.phone.settings.page = phone::settings::SettingsPage::Sounds;
            runtime.phone.index = 0;
        }
        100 => {
            runtime.phone.settings.page = phone::settings::SettingsPage::Wallpaper;
            runtime.phone.index = 0;
        }
        101 => {
            runtime.phone.launch(Screen::Contacts);
            runtime.phone.organizer.page = phone::organizer::OrganizerPage::ContactGroups;
            runtime.phone.contacts.groups = vec!["家人".into(), "朋友".into()];
            runtime.phone.index = 1;
        }
        102 => {
            let img = sample_picture();
            runtime
                .phone
                .media
                .pictures
                .insert("/card/sample.png".into(), img);
            runtime
                .phone
                .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
            runtime.phone.organizer.draft[0] = "/card/sample.png".into();
            runtime.phone.contact_load_field(0);
        }
        103 => {
            runtime.phone.open_media(phone::media::Purpose::Pictures);
            runtime.phone.media.ui.entries = vec![crate::storage::files::Entry {
                name: "手机存储".into(),
                path: "/phone".into(),
                directory: true,
                len: 0,
            }];
        }
        104 => {
            runtime.phone.media.ui.path = "/card".into();
            runtime.phone.media.ui.entries = vec![crate::storage::files::Entry {
                name: "sample.png".into(),
                path: "/card/sample.png".into(),
                directory: false,
                len: 100,
            }];
            runtime.phone.index = 0;
        }
        105 => {
            runtime.phone.media.ui.viewing = true;
            runtime.phone.media.ui.selected = "/card/sample.png".into();
            runtime.phone.media.ui.frame = Some(sample_picture());
        }
        106 => {
            runtime.phone.media.ui.zoom = 2;
        }
        107 => {
            runtime.phone.media.ui.purpose = phone::media::Purpose::Videos;
            runtime.phone.media.ui.zoom = 1;
            runtime.phone.media.ui.position = 3.;
            runtime.phone.media.ui.duration = 6.;
            runtime.phone.media.ui.playing = true;
        }
        108 => {
            runtime.phone.launch(Screen::Settings);
            runtime.phone.settings.page = phone::settings::SettingsPage::Time;
        }
        109 => {
            runtime.phone.settings.page = phone::settings::SettingsPage::Connections;
        }
        110 => {
            runtime.phone.settings.page = phone::settings::SettingsPage::Reset;
        }
        111 => {
            runtime.phone.launch(Screen::Music);
            runtime.phone.playback.view = phone::MusicView::Playlists;
            runtime.phone.playback.playlists = vec![phone::Playlist {
                name: "收藏".into(),
                paths: vec!["/card/test.wav".into()],
            }];
            runtime.phone.set_music_paths(vec!["/card/test.wav".into()]);
            runtime.phone.playback.model.tracks = vec![("test.wav".into(), 60.)];
        }
        112 => {
            runtime.phone.playback.view = phone::MusicView::Playlist;
            runtime.phone.playback.playlist = 0;
            runtime.phone.index = 0;
        }
        113 => {
            runtime.phone.launch(Screen::Settings);
            runtime.phone.settings.page = phone::settings::SettingsPage::Ringtone;
            runtime.phone.index = 2;
        }
        114 => {
            runtime.phone.launch(Screen::Organizer(
                phone::organizer::OrganizerPage::AlarmRingtone,
            ));
            runtime.phone.organizer.page = phone::organizer::OrganizerPage::AlarmRingtone;
            runtime.phone.index = 2;
        }
        115 => {
            runtime.phone.launch(Screen::Organizer(
                phone::organizer::OrganizerPage::EventRingtone,
            ));
            runtime.phone.organizer.page = phone::organizer::OrganizerPage::EventRingtone;
            runtime.phone.index = 20;
        }
        116 => {
            runtime.phone.launch(Screen::Organizer(
                phone::organizer::OrganizerPage::ContactRingtone,
            ));
            runtime.phone.organizer.draft = vec![String::new(); 10];
            runtime.phone.organizer.draft[7] = "Clock1".into();
            runtime.phone.index = 2;
        }
        117 => {
            runtime.phone.overlay = None;
            runtime.phone.messaging.compose = Compose::default();
            runtime.phone.messaging.compose.recipient = false;
            runtime.phone.go(Screen::Compose);
            for digit in [6, 4] {
                runtime.dispatch(Event {
                    key: Key::Digit(digit),
                    phase: Phase::Down,
                });
            }
        }
        118 => runtime.dispatch(Event {
            key: Key::Down,
            phase: Phase::Down,
        }),
        119 => {
            runtime.dispatch(Event {
                key: Key::Up,
                phase: Phase::Down,
            });
            runtime.dispatch(Event {
                key: Key::Right,
                phase: Phase::Down,
            });
            for _ in 0..4 {
                runtime.dispatch(Event {
                    key: Key::Down,
                    phase: Phase::Down,
                });
            }
        }
        120 => {
            runtime.phone.messaging.compose.body.prediction.selected = 0;
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
        }
        121 => {
            for digit in [4, 2, 6] {
                runtime.dispatch(Event {
                    key: Key::Digit(digit),
                    phase: Phase::Down,
                });
            }
        }
        122 => {
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            assert_eq!(runtime.phone.messaging.compose.body.text(), "你好");
            runtime.phone.messaging.compose.body.prediction.clear();
        }
        123 => {
            runtime.phone.messaging.compose.body.clear();
            for digit in [6, 4] {
                runtime.dispatch(Event {
                    key: Key::Digit(digit),
                    phase: Phase::Down,
                });
            }
            let p = &mut runtime.phone.messaging.compose.body.prediction;
            p.spelling_focus = false;
            p.selected = p
                .candidates
                .iter()
                .position(|c| c.text == "酿")
                .expect("Native learning fixture");
        }
        124 => {
            runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            });
            assert_eq!(runtime.phone.messaging.compose.body.text(), "酿");
        }
        _ => {}
    }
}
fn sample_picture() -> image::RgbaImage {
    image::RgbaImage::from_fn(240, 180, |x, y| {
        image::Rgba(if y < 60 {
            [40, 90, 180, 255]
        } else if y < 120 {
            if x < 120 {
                [255, 190, 40, 255]
            } else {
                [80, 180, 100, 255]
            }
        } else {
            [120, 60, 150, 255]
        })
    })
}
