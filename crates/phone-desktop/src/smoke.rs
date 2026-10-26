//! Bounded synthetic screenshot routes for a real winit/wgpu window.
use super::*;
impl App {
    /// Synthetic route checks on an actual winit/wgpu surface. Not physical-device QA.
    pub(super) fn smoke_frame(&mut self, event_loop: &ActiveEventLoop) {
        use phone::{Compose, Screen};
        use phone_core::input::{Key, Phase};
        let dir = self.smoke.clone().unwrap();
        if self.gpu.as_ref().is_none_or(|g| g.presented == 0) {
            return;
        }
        let name = match self.smoke_stage {
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
            75 => "phone-disabled",
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
            _ => {
                eprintln!(
                    "SMOKE PASS: {} presented frames",
                    self.gpu.as_ref().unwrap().presented
                );
                self.smoke = None;
                event_loop.exit();
                return;
            }
        };
        if let Err(e) = self.canvas.save(&dir.join(format!("{name}.png"))) {
            self.fail(event_loop, e.to_string());
            return;
        }
        match self.smoke_stage {
            0 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            1 => {
                self.phone.compose = Compose::default();
                self.phone.compose.to.insert("12345");
                self.phone
                    .compose
                    .body
                    .insert("Hello Nokia!\n短信编辑测试。\n240 × 320");
                self.phone.compose.recipient = false;
                self.phone.go(Screen::Compose);
            }
            2 => {
                self.phone.go(Screen::Browser);
                self.phone.browser_view = phone::BrowserView::Page;
                self.phone.browser = phone_core::browser::Browser::new(web::demo());
            }
            3 => {
                self.phone.go(Screen::Music);
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            4 => {
                self.phone.compose = Compose::default();
                self.phone.compose.recipient = false;
                self.phone.compose.to.insert("12345");
                self.phone.compose.body.insert("明天见！\n");
                self.phone.go(Screen::Compose);
                for digit in [6, 4, 4, 2, 6] {
                    self.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
            }
            5 => {
                self.phone.compose.body.clear();
                self.phone
                    .compose
                    .body
                    .prediction
                    .set_language(Some(phone_core::predict::Language::English));
                self.phone.compose.body.insert("Let's go ");
                for digit in [4, 6, 6, 3] {
                    self.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
            }
            6 => {
                self.phone.compose.body.clear();
                self.phone.compose.body.prediction.set_language(None);
                self.phone.compose.body.insert("简体／繁體：你好閱讀\n生僻：龘 喆 髙 𠮷\nかな カタカナ 한글\nSymbols: ✓ ♫ ★ →\nEmoji: 😀 🙂 🤔 👍\nGreek Ω · Cyrillic Ж");
            }
            7 => {
                self.phone.compose = Compose::default();
                self.phone.compose.recipient = false;
                self.phone.go(Screen::Compose);
            }
            8 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            9 => {
                self.phone.go(Screen::Settings);
                self.phone.settings_page = phone::settings::SettingsPage::Theme;
                self.phone.theme_preview = self.phone.accent;
            }
            10 => {
                self.phone.compose = Compose::default();
                self.phone.compose.recipient = false;
                self.phone
                    .compose
                    .body
                    .insert(&"软换行与光标测试 Hello!\n".repeat(14));
                self.phone.go(Screen::Compose);
            }
            11 => {
                self.phone.compose.body.clear();
                for d in [6, 4] {
                    self.dispatch(Event {
                        key: Key::Digit(d),
                        phase: Phase::Down,
                    });
                }
                self.dispatch(Event {
                    key: Key::Left,
                    phase: Phase::Down,
                });
                for _ in 0..5 {
                    self.dispatch(Event {
                        key: Key::Down,
                        phase: Phase::Down,
                    });
                }
            }
            12 => {
                self.phone.go(Screen::Browser);
                self.phone.browser_view = phone::BrowserView::Start;
            }
            13 => {
                self.phone.browser = phone_core::browser::Browser::new(web::demo());
                self.phone.browser_view = phone::BrowserView::Page;
                let i = self
                    .phone
                    .browser
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
                    &mut self.phone.browser.page.elements[i]
                {
                    selected.fill(false);
                    selected[1] = true;
                }
                self.phone.browser.focus = i;
                self.phone.handle(
                    Event {
                        key: Key::Center,
                        phase: Phase::Down,
                    },
                    self.now(),
                );
            }
            14 => {
                self.phone.go(Screen::Compose);
                self.phone.compose = Compose::default();
                self.phone.compose.recipient = false;
                self.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::Symbols,
                    index: 41,
                });
            }
            15 => {
                self.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::Smileys,
                    index: 0,
                })
            }
            16 => {
                self.phone.go(Screen::Settings);
                self.phone.settings_page = phone::settings::SettingsPage::Shortcuts;
            }
            17 => {
                self.phone.go(Screen::Music);
                self.phone.music_view = phone::MusicView::Root;
            }
            18 => self.phone.music_view = phone::MusicView::Songs,
            19 => self.phone.music_view = phone::MusicView::Repeat,
            20 => {
                self.phone.go(Screen::Browser);
                self.phone.browser_view = phone::BrowserView::Page;
                self.phone.browser = phone_core::browser::Browser::new(web::parse(
                    "about:fixture",
                    r#"<title>Form test</title><form><label>Customer name<input name=name value=test></label><p>Pizza Size</p><input type=radio name=size aria-label=Small><input type=radio name=size aria-label=Medium checked><input type=checkbox name=topping aria-label=Bacon checked><input type=checkbox name=topping aria-label=Cheese checked></form>"#,
                ));
                self.phone.browser.focus = 3;
            }
            21 => {
                self.phone
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
                    self.phone.handle(
                        Event {
                            key,
                            phase: Phase::Down,
                        },
                        self.now(),
                    );
                }
            }
            22 => self
                .phone
                .launch(Screen::Organizer(phone::organizer::OrganizerPage::Calendar)),
            23 => {
                self.phone.launch(Screen::Contacts);
                self.phone
                    .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
            }
            24 => {
                self.phone
                    .launch(Screen::Organizer(phone::organizer::OrganizerPage::Alarms));
                self.phone
                    .organizer_form(phone::organizer::OrganizerPage::AlarmForm, None);
            }
            25 => {
                self.phone
                    .launch(Screen::Tools(phone::tools::ToolPage::Stopwatch));
                self.phone.tools.stopwatch.elapsed_ms = 12345;
            }
            26 => self
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Timer)),
            27 => self
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Radio)),
            28 => {
                self.phone.launch(Screen::Snake);
                self.phone.snake.start();
                self.phone.snake_view = phone::snake::SnakeView::Game;
            }
            29 => {
                self.phone
                    .launch(Screen::Tools(phone::tools::ToolPage::Conversion(0)));
                self.phone.tools.conversion_input = "32".into();
            }
            30 => {
                self.phone.launch(Screen::Browser);
                self.phone.browser_view = phone::BrowserView::Settings;
            }
            31 => {
                self.phone.go(Screen::Compose);
                self.phone.compose = Compose::default();
                self.phone.compose.to.insert("12345");
            }
            32 => {
                self.phone.launch(Screen::Browser);
                self.phone.browser_view = phone::BrowserView::Page;
                self.phone.browser_data.settings.font_size = 0;
                self.phone.browser = phone_core::browser::Browser::new(web::parse(
                    "http://httpbin.org/forms/post",
                    include_str!("../../../scripts/fixtures/browser-form.html"),
                ));
                self.phone.browser.focus = self
                    .phone
                    .browser
                    .page
                    .elements
                    .iter()
                    .position(|e| matches!(e, phone_core::browser::Element::Field { .. }))
                    .unwrap();
            }
            33 => {
                self.phone.browser = phone_core::browser::Browser::new(web::parse(
                    "http://info.cern.ch/",
                    include_str!("../../../scripts/fixtures/browser-links.html"),
                ));
                self.phone.browser.focus = self
                    .phone
                    .browser
                    .page
                    .elements
                    .iter()
                    .position(|e| matches!(e, phone_core::browser::Element::Link { .. }))
                    .unwrap();
            }
            34 => {
                self.phone.browser_view = phone::BrowserView::Start;
                self.phone.index = 0;
                self.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::BrowserRoot,
                    index: 0,
                });
            }
            35 => {
                self.phone.browser_view = phone::BrowserView::Page;
                self.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::Browser,
                    index: 0,
                });
            }
            36 => {
                self.phone.overlay = None;
                self.phone.browser_view = phone::BrowserView::FontSize;
                self.phone.index = 0;
            }
            37 => {
                self.phone.browser_view = phone::BrowserView::Start;
                self.phone.overlay = Some(phone::Overlay::Text {
                    target: phone::TextTarget::Address,
                    editor: Box::new(phone_core::editor::Editor::new("http://", 512)),
                });
            }
            38 => {
                self.phone.go(Screen::Compose);
                self.phone.compose = Compose::default();
            }
            39 => {
                self.phone.launch(Screen::Browser);
                self.phone.browser_view = phone::BrowserView::Start;
                self.phone.overlay = Some(phone::Overlay::CloseBrowser);
            }
            40 => {
                self.phone.overlay = None;
                self.phone.browser_view = phone::BrowserView::Clear;
                self.phone.index = 0;
            }
            41 => {
                self.phone.browser_view = phone::BrowserView::Settings;
                self.phone.index = 7;
            }
            42 => {
                self.phone.launch(Screen::Snake);
                self.phone.snake.discard();
                self.phone.snake_view = phone::snake::SnakeView::Menu;
                self.phone.index = 0;
            }
            43 => {
                self.phone.snake_view = phone::snake::SnakeView::Level;
                self.phone.snake_level_preview = 7;
            }
            44 => {
                self.phone.snake_view = phone::snake::SnakeView::Types;
                self.phone.index = 4;
            }
            45 => {
                self.phone.snake_view = phone::snake::SnakeView::Scores;
                self.phone.snake_data.scores[0] = [42, 30, 20, 10, 1];
                self.phone.snake.kind = 0;
            }
            46 => {
                self.phone.snake_view = phone::snake::SnakeView::Sound;
                self.phone.snake_data.sound = false;
                self.phone.index = 0;
            }
            47 => {
                self.phone.snake.start();
                self.phone.snake.running = false;
                self.phone.snake.food = (3, 8);
                self.phone.snake_view = phone::snake::SnakeView::Game;
            }
            48 => {
                self.phone.snake.kind = 1;
                self.phone.snake.score = 123;
            }
            49 => {
                // Match native session149 positions for visual comparison only;
                // food RNG is deliberately not claimed to be the original.
                self.phone.snake.kind = 0;
                self.phone.snake.start();
                self.phone.snake.food = (1, 3);
                self.phone.snake.tick(self.phone.snake.tick_ms());
                self.phone.snake.running = false;
            }
            50 => {
                self.phone
                    .launch(Screen::Organizer(phone::organizer::OrganizerPage::Alarms));
                self.phone
                    .organizer_form(phone::organizer::OrganizerPage::AlarmForm, None);
                for key in [Key::Down, Key::Center] {
                    self.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            51 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            52 => {
                for key in [Key::SoftRight, Key::Down, Key::Down, Key::Down, Key::Center] {
                    self.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            53 => self.dispatch(Event {
                key: Key::Down,
                phase: Phase::Down,
            }),
            54 => self.dispatch(Event {
                key: Key::SoftRight,
                phase: Phase::Down,
            }),
            55 => {
                for key in [Key::Up, Key::Up] {
                    self.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            56 => {
                for key in [Key::Up, Key::Up, Key::Center] {
                    self.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            57 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            58 => {
                // Local alert illustration, not a captured native interruption.
                self.phone.local_data.alarms[0].fields[1] = "不必重复".into();
                self.phone
                    .check_alarms(phone_core::calendar::Date::default(), 540);
            }
            59 => {
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.phone.compose = Compose::default();
                self.phone.compose.to.insert("12345");
                self.phone.compose.body.insert("本地测试信息。\n这条信息不会发送。\n\n第一段正文\n第二段正文\n第三段正文\n第四段正文\n第五段正文\n第六段正文\n最后一行。");
                self.phone.compose.recipient = false;
                self.phone.go(Screen::Compose);
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.flush();
            }
            60 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            61 => {
                for _ in 0..6 {
                    self.dispatch(Event {
                        key: Key::Down,
                        phase: Phase::Down,
                    });
                }
            }
            62 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            63 => {
                for key in [Key::Down, Key::Down, Key::Center] {
                    self.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            64 => self.phone.go(Screen::Outbox),
            65 => self.dispatch(Event {
                key: Key::Delete,
                phase: Phase::Down,
            }),
            66 => {
                self.phone.receive_message("12345".into(), "这是一条本地测试信息。\n没有连接短信服务。\n回复、转发和删除均在本机完成。".into()).unwrap();
                self.phone.go(Screen::Inbox);
            }
            67 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            68 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            69 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            70 => {
                self.phone.mailbox = Default::default();
                self.phone.mailbox.outbox.push(phone_core::sms::Message {
                    to: "12345".into(),
                    body: "明天见。".into(),
                    time: 1_792_944_000_000,
                    sent: true,
                });
                self.phone
                    .receive_message("12345".into(), "好的，联系 67890。".into())
                    .unwrap();
                self.phone.mailbox.inbox[0].time = 1_792_944_060_000;
                self.phone.chat = Default::default();
                self.phone.go(Screen::Messages);
            }
            71 => self.phone.open_thread("12345".into()),
            72 => {
                self.phone.chat.focus = Some(1);
                self.phone.chat.popup = phone::conversation::Popup::Bubble;
            }
            73 => {
                self.phone.chat.number_focus = true;
                self.phone.chat.popup = phone::conversation::Popup::Number;
            }
            74 => {
                self.phone.chat.popup = phone::conversation::Popup::None;
                self.phone.go(Screen::Menu);
                self.phone.index = 7;
            }
            75 => {
                self.phone
                    .receive_message("12345".into(), "长信息滚动测试。\n".repeat(30))
                    .unwrap();
                self.phone.open_thread("12345".into());
                self.phone.chat.focus = Some(2);
                self.phone.chat.reveal = true;
            }
            76 => {
                self.phone.launch(Screen::Snake);
                self.phone.snake.kind = 3;
                self.phone.snake.start();
                self.phone.snake.food = (4, 14);
                self.phone.snake.key(Key::Down);
                self.phone.snake.tick(1000);
                self.phone.snake.running = false;
                self.phone.snake_view = phone::snake::SnakeView::GameOver;
            }
            77 => {
                self.phone.apply_snake_data(phone::snake::SnakeData {
                    level: 8,
                    ..Default::default()
                });
                self.phone.snake_view = phone::snake::SnakeView::Menu;
                self.phone.index = 1;
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            78 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            79 => {
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.dispatch(Event {
                    key: Key::SoftRight,
                    phase: Phase::Down,
                });
                self.phone.index = 3;
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.phone.index = 3;
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            80 => {
                self.phone.compose = Compose::default();
                self.phone.compose.recipient = false;
                self.phone.compose.body.prediction.set_language(None);
                self.phone.compose.body.titlecase = true;
                self.phone.compose.body.insert("你");
                self.phone.go(Screen::Compose);
            }
            81 => {
                self.phone.compose.body.insert("\n");
            }
            82 => {
                self.phone.compose.body.clear();
                self.phone.compose.body.titlecase = false;
                self.phone.compose.body.numeric = true;
                self.phone
                    .compose
                    .body
                    .insert(&format!("呢{}123456789", "1234567890".repeat(6)));
            }
            83 => {
                self.phone.compose.body.clear();
                self.phone
                    .compose
                    .body
                    .insert(&format!("{}1", "1234567890".repeat(16)));
            }
            84 => {
                self.phone
                    .launch(Screen::Tools(phone::tools::ToolPage::Flashlight));
                self.phone.tools.torch = false;
            }
            85 => self.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            86 => {
                self.phone.tools.torch = false;
                self.phone.local_data.contacts.clear();
                self.phone.organizer.contact = Default::default();
                self.phone.launch(Screen::Contacts);
            }
            87 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            88 => {
                self.phone.index = 1;
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            89 => self.phone.contact_load_field(5),
            90 => self.phone.contact_load_field(6),
            91 => self.phone.contact_load_field(7),
            92 => self.phone.contact_load_field(8),
            93 => {
                // Verified test-mode fixture from the reviewed handset capture.
                self.phone.local_data.contacts.push(
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
                self.phone.organizer.selected = 0;
                self.phone.organizer.page = phone::organizer::OrganizerPage::ContactDetail;
                self.phone.index = 0;
            }
            94 => self.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            95 => {
                self.phone.index = 1;
                self.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            96 => self.phone.launch(Screen::Contacts),
            97 => {
                self.phone.organizer.contact.query.insert("no-match");
            }
            98 => {
                self.phone
                    .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
                self.phone.text("测试联系人");
            }
            99 => {
                self.phone.launch(Screen::Settings);
                self.phone.settings_page = phone::settings::SettingsPage::Sounds;
                self.phone.index = 0;
            }
            100 => {
                self.phone.settings_page = phone::settings::SettingsPage::Wallpaper;
                self.phone.index = 0;
            }
            101 => {
                self.phone.launch(Screen::Contacts);
                self.phone.organizer.page = phone::organizer::OrganizerPage::ContactGroups;
                self.phone.local_data.groups = vec!["家人".into(), "朋友".into()];
                self.phone.index = 1;
            }
            102 => {
                let img = sample_picture();
                self.phone.pictures.insert("/card/sample.png".into(), img);
                self.phone
                    .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
                self.phone.organizer.draft[0] = "/card/sample.png".into();
                self.phone.contact_load_field(0);
            }
            103 => {
                self.phone
                    .open_media(phone::media_browser::Purpose::Pictures);
                self.phone.media_ui.entries = vec![
                    phone_vfs::Entry {
                        name: "手机存储".into(),
                        path: "/phone".into(),
                        directory: true,
                        len: 0,
                    },
                    phone_vfs::Entry {
                        name: "存储卡".into(),
                        path: "/card".into(),
                        directory: true,
                        len: 0,
                    },
                ];
            }
            104 => {
                self.phone.media_ui.path = "/card".into();
                self.phone.media_ui.entries = vec![phone_vfs::Entry {
                    name: "sample.png".into(),
                    path: "/card/sample.png".into(),
                    directory: false,
                    len: 100,
                }];
                self.phone.index = 0;
            }
            105 => {
                self.phone.media_ui.viewing = true;
                self.phone.media_ui.selected = "/card/sample.png".into();
                self.phone.media_ui.frame = Some(sample_picture());
            }
            106 => {
                self.phone.media_ui.zoom = 2;
            }
            107 => {
                self.phone.media_ui.purpose = phone::media_browser::Purpose::Videos;
                self.phone.media_ui.zoom = 1;
                self.phone.media_ui.position = 3.;
                self.phone.media_ui.duration = 6.;
                self.phone.media_ui.playing = true;
            }
            108 => {
                self.phone.launch(Screen::Settings);
                self.phone.settings_page = phone::settings::SettingsPage::Time;
            }
            109 => {
                self.phone.settings_page = phone::settings::SettingsPage::Connections;
            }
            110 => {
                self.phone.settings_page = phone::settings::SettingsPage::Reset;
            }
            111 => {
                self.phone.launch(Screen::Music);
                self.phone.music_view = phone::MusicView::Playlists;
                self.phone.local_data.playlists = vec![phone::Playlist {
                    name: "收藏".into(),
                    paths: vec!["/card/test.wav".into()],
                }];
                self.phone.set_music_paths(vec!["/card/test.wav".into()]);
                self.phone.music.tracks = vec![("test.wav".into(), 60.)];
            }
            112 => {
                self.phone.music_view = phone::MusicView::Playlist;
                self.phone.playlist = 0;
                self.phone.index = 0;
            }
            113 => {
                self.phone.launch(Screen::Settings);
                self.phone.settings_page = phone::settings::SettingsPage::Ringtone;
                self.phone.index = 2;
            }
            114 => {
                self.phone.launch(Screen::Organizer(
                    phone::organizer::OrganizerPage::AlarmRingtone,
                ));
                self.phone.organizer.page = phone::organizer::OrganizerPage::AlarmRingtone;
                self.phone.index = 2;
            }
            115 => {
                self.phone.launch(Screen::Organizer(
                    phone::organizer::OrganizerPage::EventRingtone,
                ));
                self.phone.organizer.page = phone::organizer::OrganizerPage::EventRingtone;
                self.phone.index = 20;
            }
            116 => {
                self.phone.launch(Screen::Organizer(
                    phone::organizer::OrganizerPage::ContactRingtone,
                ));
                self.phone.organizer.draft = vec![String::new(); 10];
                self.phone.organizer.draft[7] = "Clock1".into();
                self.phone.index = 2;
            }
            _ => {}
        }
        self.smoke_stage += 1;
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
