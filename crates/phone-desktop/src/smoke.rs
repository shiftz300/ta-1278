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
            _ => {}
        }
        self.smoke_stage += 1;
    }
}
