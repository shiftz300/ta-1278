//! Bounded synthetic screenshot routes for a real winit/wgpu window.
use super::*;
use crate::web;
use phone_core::input::Event;
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
            118 => "t9-spelling-ni",
            119 => "t9-spelling-mi",
            120 => "t9-next-page",
            121 => "t9-association",
            122 => "t9-context-hao",
            123 => "t9-committed",
            124 => "t9-learning-selection",
            125 => "t9-learning-committed",
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
        if let Err(e) = self.runtime.canvas.save(&dir.join(format!("{name}.png"))) {
            self.fail(event_loop, e.to_string());
            return;
        }
        match self.smoke_stage {
            0 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            1 => {
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.to.insert("12345");
                self.runtime
                    .phone
                    .compose
                    .body
                    .insert("Hello Nokia!\n短信编辑测试。\n240 × 320");
                self.runtime.phone.compose.recipient = false;
                self.runtime.phone.go(Screen::Compose);
            }
            2 => {
                self.runtime.phone.go(Screen::Browser);
                self.runtime.phone.browser_view = phone::BrowserView::Page;
                self.runtime.phone.browser = phone_core::browser::Browser::new(web::demo());
            }
            3 => {
                self.runtime.phone.go(Screen::Music);
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            4 => {
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.recipient = false;
                self.runtime.phone.compose.to.insert("12345");
                self.runtime.phone.compose.body.insert("明天见！\n");
                self.runtime.phone.go(Screen::Compose);
                for digit in [6, 4] {
                    self.runtime.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
            }
            5 => {
                self.runtime.phone.compose.body.clear();
                self.runtime
                    .phone
                    .compose
                    .body
                    .prediction
                    .set_language(Some(phone_core::predict::Language::English));
                self.runtime.phone.compose.body.insert("Let's go ");
                for digit in [4, 6, 6, 3] {
                    self.runtime.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
            }
            6 => {
                self.runtime.phone.compose.body.clear();
                self.runtime
                    .phone
                    .compose
                    .body
                    .prediction
                    .set_language(None);
                self.runtime.phone.compose.body.insert("简体／繁體：你好閱讀\n生僻：龘 喆 髙 𠮷\nかな カタカナ 한글\nSymbols: ✓ ♫ ★ →\nEmoji: 😀 🙂 🤔 👍\nGreek Ω · Cyrillic Ж");
            }
            7 => {
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.recipient = false;
                self.runtime.phone.go(Screen::Compose);
            }
            8 => self.runtime.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            9 => {
                self.runtime.phone.go(Screen::Settings);
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Theme;
                self.runtime.phone.theme_preview = self.runtime.phone.accent;
            }
            10 => {
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.recipient = false;
                self.runtime
                    .phone
                    .compose
                    .body
                    .insert(&"软换行与光标测试 Hello!\n".repeat(14));
                self.runtime.phone.go(Screen::Compose);
            }
            11 => {
                self.runtime.phone.compose.body.clear();
                for d in [6, 4] {
                    self.runtime.dispatch(Event {
                        key: Key::Digit(d),
                        phase: Phase::Down,
                    });
                }
                self.runtime.dispatch(Event {
                    key: Key::Left,
                    phase: Phase::Down,
                });
                for _ in 0..5 {
                    self.runtime.dispatch(Event {
                        key: Key::Down,
                        phase: Phase::Down,
                    });
                }
            }
            12 => {
                self.runtime.phone.go(Screen::Browser);
                self.runtime.phone.browser_view = phone::BrowserView::Start;
            }
            13 => {
                self.runtime.phone.browser = phone_core::browser::Browser::new(web::demo());
                self.runtime.phone.browser_view = phone::BrowserView::Page;
                let i = self
                    .runtime
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
                    &mut self.runtime.phone.browser.page.elements[i]
                {
                    selected.fill(false);
                    selected[1] = true;
                }
                self.runtime.phone.browser.focus = i;
                self.runtime.phone.handle(
                    Event {
                        key: Key::Center,
                        phase: Phase::Down,
                    },
                    self.now(),
                );
            }
            14 => {
                self.runtime.phone.go(Screen::Compose);
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.recipient = false;
                self.runtime.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::Symbols,
                    index: 41,
                });
            }
            15 => {
                self.runtime.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::Smileys,
                    index: 0,
                })
            }
            16 => {
                self.runtime.phone.go(Screen::Settings);
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Shortcuts;
            }
            17 => {
                self.runtime.phone.go(Screen::Music);
                self.runtime.phone.music_view = phone::MusicView::Root;
            }
            18 => self.runtime.phone.music_view = phone::MusicView::Songs,
            19 => self.runtime.phone.music_view = phone::MusicView::Repeat,
            20 => {
                self.runtime.phone.go(Screen::Browser);
                self.runtime.phone.browser_view = phone::BrowserView::Page;
                self.runtime.phone.browser = phone_core::browser::Browser::new(web::parse(
                    "about:fixture",
                    r#"<title>Form test</title><form><label>Customer name<input name=name value=test></label><p>Pizza Size</p><input type=radio name=size aria-label=Small><input type=radio name=size aria-label=Medium checked><input type=checkbox name=topping aria-label=Bacon checked><input type=checkbox name=topping aria-label=Cheese checked></form>"#,
                ));
                self.runtime.phone.browser.focus = 3;
            }
            21 => {
                self.runtime
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
                    self.runtime.phone.handle(
                        Event {
                            key,
                            phase: Phase::Down,
                        },
                        self.now(),
                    );
                }
            }
            22 => self
                .runtime
                .phone
                .launch(Screen::Organizer(phone::organizer::OrganizerPage::Calendar)),
            23 => {
                self.runtime.phone.launch(Screen::Contacts);
                self.runtime
                    .phone
                    .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
            }
            24 => {
                self.runtime
                    .phone
                    .launch(Screen::Organizer(phone::organizer::OrganizerPage::Alarms));
                self.runtime
                    .phone
                    .organizer_form(phone::organizer::OrganizerPage::AlarmForm, None);
            }
            25 => {
                self.runtime
                    .phone
                    .launch(Screen::Tools(phone::tools::ToolPage::Stopwatch));
                self.runtime.phone.tools.stopwatch.elapsed_ms = 12345;
            }
            26 => self
                .runtime
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Timer)),
            27 => self
                .runtime
                .phone
                .launch(Screen::Tools(phone::tools::ToolPage::Radio)),
            28 => {
                self.runtime.phone.launch(Screen::Snake);
                self.runtime.phone.snake.start();
                self.runtime.phone.snake_view = phone::snake::SnakeView::Game;
            }
            29 => {
                self.runtime
                    .phone
                    .launch(Screen::Tools(phone::tools::ToolPage::Conversion(0)));
                self.runtime.phone.tools.conversion = Default::default();
                self.runtime.phone.text("32");
            }
            30 => {
                self.runtime.phone.launch(Screen::Browser);
                self.runtime.phone.browser_view = phone::BrowserView::Settings;
            }
            31 => {
                self.runtime.phone.go(Screen::Compose);
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.to.insert("12345");
            }
            32 => {
                self.runtime.phone.launch(Screen::Browser);
                self.runtime.phone.browser_view = phone::BrowserView::Page;
                self.runtime.phone.browser_data.settings.font_size = 0;
                self.runtime.phone.browser = phone_core::browser::Browser::new(web::parse(
                    "http://httpbin.org/forms/post",
                    include_str!("../../../../scripts/fixtures/browser-form.html"),
                ));
                self.runtime.phone.browser.focus = self
                    .runtime
                    .phone
                    .browser
                    .page
                    .elements
                    .iter()
                    .position(|e| matches!(e, phone_core::browser::Element::Field { .. }))
                    .unwrap();
            }
            33 => {
                self.runtime.phone.browser = phone_core::browser::Browser::new(web::parse(
                    "http://info.cern.ch/",
                    include_str!("../../../../scripts/fixtures/browser-links.html"),
                ));
                self.runtime.phone.browser.focus = self
                    .runtime
                    .phone
                    .browser
                    .page
                    .elements
                    .iter()
                    .position(|e| matches!(e, phone_core::browser::Element::Link { .. }))
                    .unwrap();
            }
            34 => {
                self.runtime.phone.browser_view = phone::BrowserView::Start;
                self.runtime.phone.index = 0;
                self.runtime.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::BrowserRoot,
                    index: 0,
                });
            }
            35 => {
                self.runtime.phone.browser_view = phone::BrowserView::Page;
                self.runtime.phone.overlay = Some(phone::Overlay::Menu {
                    kind: phone::MenuKind::Browser,
                    index: 0,
                });
            }
            36 => {
                self.runtime.phone.overlay = None;
                self.runtime.phone.browser_view = phone::BrowserView::FontSize;
                self.runtime.phone.index = 0;
            }
            37 => {
                self.runtime.phone.browser_view = phone::BrowserView::Start;
                self.runtime.phone.overlay = Some(phone::Overlay::Text {
                    target: phone::TextTarget::Address,
                    editor: Box::new(phone_core::editor::Editor::new("http://", 512)),
                });
            }
            38 => {
                self.runtime.phone.go(Screen::Compose);
                self.runtime.phone.compose = Compose::default();
            }
            39 => {
                self.runtime.phone.launch(Screen::Browser);
                self.runtime.phone.browser_view = phone::BrowserView::Start;
                self.runtime.phone.overlay = Some(phone::Overlay::CloseBrowser);
            }
            40 => {
                self.runtime.phone.overlay = None;
                self.runtime.phone.browser_view = phone::BrowserView::Clear;
                self.runtime.phone.index = 0;
            }
            41 => {
                self.runtime.phone.browser_view = phone::BrowserView::Settings;
                self.runtime.phone.index = 7;
            }
            42 => {
                self.runtime.phone.launch(Screen::Snake);
                self.runtime.phone.snake.discard();
                self.runtime.phone.snake_view = phone::snake::SnakeView::Menu;
                self.runtime.phone.index = 0;
            }
            43 => {
                self.runtime.phone.snake_view = phone::snake::SnakeView::Level;
                self.runtime.phone.snake_level_preview = 7;
            }
            44 => {
                self.runtime.phone.snake_view = phone::snake::SnakeView::Types;
                self.runtime.phone.index = 4;
            }
            45 => {
                self.runtime.phone.snake_view = phone::snake::SnakeView::Scores;
                self.runtime.phone.snake_data.scores[0] = [42, 30, 20, 10, 1];
                self.runtime.phone.snake.kind = 0;
            }
            46 => {
                self.runtime.phone.snake_view = phone::snake::SnakeView::Sound;
                self.runtime.phone.snake_data.sound = false;
                self.runtime.phone.index = 0;
            }
            47 => {
                self.runtime.phone.snake.start();
                self.runtime.phone.snake.running = false;
                self.runtime.phone.snake.food = (3, 8);
                self.runtime.phone.snake_view = phone::snake::SnakeView::Game;
            }
            48 => {
                self.runtime.phone.snake.kind = 1;
                self.runtime.phone.snake.score = 123;
            }
            49 => {
                // Match native session149 positions for visual comparison only;
                // food RNG is deliberately not claimed to be the original.
                self.runtime.phone.snake.kind = 0;
                self.runtime.phone.snake.start();
                self.runtime.phone.snake.food = (1, 3);
                self.runtime
                    .phone
                    .snake
                    .tick(self.runtime.phone.snake.tick_ms());
                self.runtime.phone.snake.running = false;
            }
            50 => {
                self.runtime
                    .phone
                    .launch(Screen::Organizer(phone::organizer::OrganizerPage::Alarms));
                self.runtime
                    .phone
                    .organizer_form(phone::organizer::OrganizerPage::AlarmForm, None);
                for key in [Key::Down, Key::Center] {
                    self.runtime.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            51 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            52 => {
                for key in [Key::SoftRight, Key::Down, Key::Down, Key::Down, Key::Center] {
                    self.runtime.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            53 => self.runtime.dispatch(Event {
                key: Key::Down,
                phase: Phase::Down,
            }),
            54 => self.runtime.dispatch(Event {
                key: Key::SoftRight,
                phase: Phase::Down,
            }),
            55 => {
                for key in [Key::Up, Key::Up] {
                    self.runtime.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            56 => {
                for key in [Key::Up, Key::Up, Key::Center] {
                    self.runtime.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            57 => self.runtime.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            58 => {
                // Local alert illustration, not a captured native interruption.
                self.runtime.phone.local_data.alarms[0].fields[1] = "不必重复".into();
                self.runtime
                    .phone
                    .check_alarms(phone_core::calendar::Date::default(), 540);
            }
            59 => {
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.to.insert("12345");
                self.runtime.phone.compose.body.insert("本地测试信息。\n这条信息不会发送。\n\n第一段正文\n第二段正文\n第三段正文\n第四段正文\n第五段正文\n第六段正文\n最后一行。");
                self.runtime.phone.compose.recipient = false;
                self.runtime.phone.go(Screen::Compose);
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.flush();
            }
            60 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            61 => {
                for _ in 0..6 {
                    self.runtime.dispatch(Event {
                        key: Key::Down,
                        phase: Phase::Down,
                    });
                }
            }
            62 => self.runtime.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            63 => {
                for key in [Key::Down, Key::Down, Key::Center] {
                    self.runtime.dispatch(Event {
                        key,
                        phase: Phase::Down,
                    });
                }
            }
            64 => self.runtime.phone.go(Screen::Outbox),
            65 => self.runtime.dispatch(Event {
                key: Key::Delete,
                phase: Phase::Down,
            }),
            66 => {
                self.runtime.phone.receive_message("12345".into(), "这是一条本地测试信息。\n没有连接短信服务。\n回复、转发和删除均在本机完成。".into()).unwrap();
                self.runtime.phone.go(Screen::Inbox);
            }
            67 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            68 => self.runtime.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            69 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            70 => {
                self.runtime.phone.mailbox = Default::default();
                self.runtime
                    .phone
                    .mailbox
                    .outbox
                    .push(phone_core::sms::Message {
                        to: "12345".into(),
                        body: "明天见。".into(),
                        time: 1_792_944_000_000,
                        sent: true,
                        submission: None,
                    });
                self.runtime
                    .phone
                    .receive_message("12345".into(), "好的，联系 67890。".into())
                    .unwrap();
                self.runtime.phone.mailbox.inbox[0].time = 1_792_944_060_000;
                self.runtime.phone.chat = Default::default();
                self.runtime.phone.go(Screen::Messages);
            }
            71 => self.runtime.phone.open_thread("12345".into()),
            72 => {
                self.runtime.phone.chat.focus = Some(1);
                self.runtime.phone.chat.popup = phone::conversation::Popup::Bubble;
            }
            73 => {
                self.runtime.phone.chat.number_focus = true;
                self.runtime.phone.chat.popup = phone::conversation::Popup::Number;
            }
            74 => {
                self.runtime.phone.chat.popup = phone::conversation::Popup::None;
                self.runtime.phone.go(Screen::Menu);
                self.runtime.phone.index = 7;
            }
            75 => {
                self.runtime
                    .phone
                    .receive_message("12345".into(), "长信息滚动测试。\n".repeat(30))
                    .unwrap();
                self.runtime.phone.open_thread("12345".into());
                self.runtime.phone.chat.focus = Some(2);
                self.runtime.phone.chat.reveal = true;
            }
            76 => {
                self.runtime.phone.launch(Screen::Snake);
                self.runtime.phone.snake.kind = 3;
                self.runtime.phone.snake.start();
                self.runtime.phone.snake.food = (4, 14);
                self.runtime.phone.snake.key(Key::Down);
                self.runtime.phone.snake.tick(1000);
                self.runtime.phone.snake.running = false;
                self.runtime.phone.snake_view = phone::snake::SnakeView::GameOver;
            }
            77 => {
                self.runtime
                    .phone
                    .apply_snake_data(phone::snake::SnakeData {
                        level: 8,
                        ..Default::default()
                    });
                self.runtime.phone.snake_view = phone::snake::SnakeView::Menu;
                self.runtime.phone.index = 1;
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            78 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            79 => {
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.runtime.dispatch(Event {
                    key: Key::SoftRight,
                    phase: Phase::Down,
                });
                self.runtime.phone.index = 3;
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.runtime.phone.index = 3;
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            80 => {
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.recipient = false;
                self.runtime
                    .phone
                    .compose
                    .body
                    .prediction
                    .set_language(None);
                self.runtime.phone.compose.body.titlecase = true;
                self.runtime.phone.compose.body.insert("你");
                self.runtime.phone.go(Screen::Compose);
            }
            81 => {
                self.runtime.phone.compose.body.insert("\n");
            }
            82 => {
                self.runtime.phone.compose.body.clear();
                self.runtime.phone.compose.body.titlecase = false;
                self.runtime.phone.compose.body.numeric = true;
                self.runtime
                    .phone
                    .compose
                    .body
                    .insert(&format!("呢{}123456789", "1234567890".repeat(6)));
            }
            83 => {
                self.runtime.phone.compose.body.clear();
                self.runtime
                    .phone
                    .compose
                    .body
                    .insert(&format!("{}1", "1234567890".repeat(16)));
            }
            84 => {
                self.runtime
                    .phone
                    .launch(Screen::Tools(phone::tools::ToolPage::Flashlight));
                self.runtime.phone.tools.torch = false;
            }
            85 => self.runtime.dispatch(Event {
                key: Key::Center,
                phase: Phase::Down,
            }),
            86 => {
                self.runtime.phone.tools.torch = false;
                self.runtime.phone.local_data.contacts.clear();
                self.runtime.phone.organizer.contact = Default::default();
                self.runtime.phone.launch(Screen::Contacts);
            }
            87 => self.runtime.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            88 => {
                self.runtime.phone.index = 1;
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            89 => self.runtime.phone.contact_load_field(5),
            90 => self.runtime.phone.contact_load_field(6),
            91 => self.runtime.phone.contact_load_field(7),
            92 => self.runtime.phone.contact_load_field(8),
            93 => {
                // Verified test-mode fixture from the reviewed handset capture.
                self.runtime.phone.local_data.contacts.push(
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
                self.runtime.phone.organizer.selected = 0;
                self.runtime.phone.organizer.page = phone::organizer::OrganizerPage::ContactDetail;
                self.runtime.phone.index = 0;
            }
            94 => self.runtime.dispatch(Event {
                key: Key::SoftLeft,
                phase: Phase::Down,
            }),
            95 => {
                self.runtime.phone.index = 1;
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            96 => self.runtime.phone.launch(Screen::Contacts),
            97 => {
                self.runtime
                    .phone
                    .organizer
                    .contact
                    .query
                    .insert("no-match");
            }
            98 => {
                self.runtime
                    .phone
                    .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
                self.runtime.phone.text("测试联系人");
            }
            99 => {
                self.runtime.phone.launch(Screen::Settings);
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Sounds;
                self.runtime.phone.index = 0;
            }
            100 => {
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Wallpaper;
                self.runtime.phone.index = 0;
            }
            101 => {
                self.runtime.phone.launch(Screen::Contacts);
                self.runtime.phone.organizer.page = phone::organizer::OrganizerPage::ContactGroups;
                self.runtime.phone.local_data.groups = vec!["家人".into(), "朋友".into()];
                self.runtime.phone.index = 1;
            }
            102 => {
                let img = sample_picture();
                self.runtime
                    .phone
                    .pictures
                    .insert("/card/sample.png".into(), img);
                self.runtime
                    .phone
                    .organizer_form(phone::organizer::OrganizerPage::ContactForm, None);
                self.runtime.phone.organizer.draft[0] = "/card/sample.png".into();
                self.runtime.phone.contact_load_field(0);
            }
            103 => {
                self.runtime
                    .phone
                    .open_media(phone::media::Purpose::Pictures);
                self.runtime.phone.media_ui.entries = vec![
                    crate::storage::files::Entry {
                        name: "手机存储".into(),
                        path: "/phone".into(),
                        directory: true,
                        len: 0,
                    },
                    crate::storage::files::Entry {
                        name: "存储卡".into(),
                        path: "/card".into(),
                        directory: true,
                        len: 0,
                    },
                ];
            }
            104 => {
                self.runtime.phone.media_ui.path = "/card".into();
                self.runtime.phone.media_ui.entries = vec![crate::storage::files::Entry {
                    name: "sample.png".into(),
                    path: "/card/sample.png".into(),
                    directory: false,
                    len: 100,
                }];
                self.runtime.phone.index = 0;
            }
            105 => {
                self.runtime.phone.media_ui.viewing = true;
                self.runtime.phone.media_ui.selected = "/card/sample.png".into();
                self.runtime.phone.media_ui.frame = Some(sample_picture());
            }
            106 => {
                self.runtime.phone.media_ui.zoom = 2;
            }
            107 => {
                self.runtime.phone.media_ui.purpose = phone::media::Purpose::Videos;
                self.runtime.phone.media_ui.zoom = 1;
                self.runtime.phone.media_ui.position = 3.;
                self.runtime.phone.media_ui.duration = 6.;
                self.runtime.phone.media_ui.playing = true;
            }
            108 => {
                self.runtime.phone.launch(Screen::Settings);
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Time;
            }
            109 => {
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Connections;
            }
            110 => {
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Reset;
            }
            111 => {
                self.runtime.phone.launch(Screen::Music);
                self.runtime.phone.music_view = phone::MusicView::Playlists;
                self.runtime.phone.local_data.playlists = vec![phone::Playlist {
                    name: "收藏".into(),
                    paths: vec!["/card/test.wav".into()],
                }];
                self.runtime
                    .phone
                    .set_music_paths(vec!["/card/test.wav".into()]);
                self.runtime.phone.music.tracks = vec![("test.wav".into(), 60.)];
            }
            112 => {
                self.runtime.phone.music_view = phone::MusicView::Playlist;
                self.runtime.phone.playlist = 0;
                self.runtime.phone.index = 0;
            }
            113 => {
                self.runtime.phone.launch(Screen::Settings);
                self.runtime.phone.settings_page = phone::settings::SettingsPage::Ringtone;
                self.runtime.phone.index = 2;
            }
            114 => {
                self.runtime.phone.launch(Screen::Organizer(
                    phone::organizer::OrganizerPage::AlarmRingtone,
                ));
                self.runtime.phone.organizer.page = phone::organizer::OrganizerPage::AlarmRingtone;
                self.runtime.phone.index = 2;
            }
            115 => {
                self.runtime.phone.launch(Screen::Organizer(
                    phone::organizer::OrganizerPage::EventRingtone,
                ));
                self.runtime.phone.organizer.page = phone::organizer::OrganizerPage::EventRingtone;
                self.runtime.phone.index = 20;
            }
            116 => {
                self.runtime.phone.launch(Screen::Organizer(
                    phone::organizer::OrganizerPage::ContactRingtone,
                ));
                self.runtime.phone.organizer.draft = vec![String::new(); 10];
                self.runtime.phone.organizer.draft[7] = "Clock1".into();
                self.runtime.phone.index = 2;
            }
            117 => {
                self.runtime.phone.overlay = None;
                self.runtime.phone.compose = Compose::default();
                self.runtime.phone.compose.recipient = false;
                self.runtime.phone.go(Screen::Compose);
                for digit in [6, 4] {
                    self.runtime.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
            }
            118 => self.runtime.dispatch(Event {
                key: Key::Down,
                phase: Phase::Down,
            }),
            119 => {
                self.runtime.dispatch(Event {
                    key: Key::Up,
                    phase: Phase::Down,
                });
                self.runtime.dispatch(Event {
                    key: Key::Right,
                    phase: Phase::Down,
                });
                for _ in 0..4 {
                    self.runtime.dispatch(Event {
                        key: Key::Down,
                        phase: Phase::Down,
                    });
                }
            }
            120 => {
                self.runtime.phone.compose.body.prediction.selected = 0;
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
            }
            121 => {
                for digit in [4, 2, 6] {
                    self.runtime.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
            }
            122 => {
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                assert_eq!(self.runtime.phone.compose.body.text(), "你好");
                self.runtime.phone.compose.body.prediction.clear();
            }
            123 => {
                self.runtime.phone.compose.body.clear();
                for digit in [6, 4] {
                    self.runtime.dispatch(Event {
                        key: Key::Digit(digit),
                        phase: Phase::Down,
                    });
                }
                let p = &mut self.runtime.phone.compose.body.prediction;
                p.spelling_focus = false;
                p.selected = p
                    .candidates
                    .iter()
                    .position(|c| c.text == "酿")
                    .expect("Native learning fixture");
            }
            124 => {
                self.runtime.dispatch(Event {
                    key: Key::Center,
                    phase: Phase::Down,
                });
                assert_eq!(self.runtime.phone.compose.body.text(), "酿");
                self.flush();
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
