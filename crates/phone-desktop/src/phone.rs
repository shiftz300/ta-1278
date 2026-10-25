mod browser;
pub mod browser_data;
mod music;
pub mod organizer;
mod overlays;
mod sms;
pub mod snake;
pub mod tools;
pub use music::MusicView;
pub mod settings;
use crate::web;
pub use browser::BrowserView;
use phone_core::{
    browser::{Back, Browser, Element},
    editor::{Editor, VisualRow},
    input::{Event, Key, Phase},
    music::Music,
    sms::{Mailbox, Message},
};
use settings::{Preferences, SettingsPage};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Home,
    Menu,
    Messages,
    Drafts,
    Outbox,
    Compose,
    Browser,
    Music,
    Snake,
    Dial,
    Contacts,
    Settings,
    Tools(tools::ToolPage),
    Organizer(organizer::OrganizerPage),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuKind {
    Sms,
    ExitSms,
    SmsRecipient,
    SmsInsert,
    Symbols,
    Smileys,
    Templates,
    Browser,
    BrowserRoot,
    BrowserOpen,
    BrowserNavigation,
    BrowserSave,
    MusicLibrary,
    Music,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextTarget {
    Address,
    Field(usize),
    Tool(usize),
    Organizer(usize),
    BrowserHome,
}
#[derive(Clone, Debug)]
pub enum Overlay {
    Menu {
        kind: MenuKind,
        index: usize,
    },
    Text {
        target: TextTarget,
        editor: Box<Editor>,
    },
    Select {
        element: usize,
        index: usize,
        selected: Vec<bool>,
        multiple: bool,
    },
    ClosePage,
    CloseBrowser,
    Notice(String),
}
pub struct Compose {
    pub to: Editor,
    pub body: Editor,
    pub recipient: bool,
    pub draft: Option<usize>,
}
impl Default for Compose {
    fn default() -> Self {
        let mut to = Editor::new("", 40);
        to.numeric = true;
        let mut body = Editor::default();
        body.prediction
            .set_language(Some(phone_core::predict::Language::Chinese));
        Self {
            to,
            body,
            recipient: true,
            draft: None,
        }
    }
}
pub struct Phone {
    pub screen: Screen,
    pub index: usize,
    pub overlay: Option<Overlay>,
    pub mailbox: Mailbox,
    pub compose: Compose,
    pub browser: Browser,
    pub music: Music,
    pub snake: phone_core::snake::Snake,
    pub snake_view: snake::SnakeView,
    pub tools: tools::Tools,
    pub organizer: organizer::Organizer,
    pub local_data: organizer::LocalData,
    pub local_dirty: bool,
    last_tick: Option<u64>,
    pub music_view: MusicView,
    pub music_started: bool,
    pub dial: Editor,
    pub status: String,
    pub pending_url: Option<String>,
    pub editor_lines: Vec<VisualRow>,
    pub dirty: bool,
    pub notice_scroll: usize,
    pub notice_lines: usize,
    pub accent: usize,
    pub pending_compose_close: bool,
    menu_index: usize,
    app_origin: Screen,
    pub preferences: Preferences,
    pub preferences_dirty: bool,
    pub settings_page: SettingsPage,
    pub theme_preview: usize,
    pub browser_view: BrowserView,
    pub browser_settings_origin: BrowserView,
    pub browser_data: browser_data::BrowserData,
    pub browser_dirty: bool,
    pub requested_url: String,
}
impl Phone {
    pub fn new(mailbox: Mailbox) -> Self {
        let mut dial = Editor::new("", 40);
        dial.numeric = true;
        Self {
            screen: Screen::Home,
            index: 0,
            overlay: None,
            mailbox,
            compose: Compose::default(),
            browser: Browser::new(phone_core::browser::Page {
                flow: vec![],
                url: String::new(),
                title: String::new(),
                elements: vec![],
                forms: vec![],
            }),
            music: Music::default(),
            snake: Default::default(),
            snake_view: Default::default(),
            tools: Default::default(),
            organizer: Default::default(),
            local_data: Default::default(),
            local_dirty: false,
            last_tick: None,
            music_view: MusicView::Root,
            music_started: false,
            dial,
            status: String::new(),
            pending_url: None,
            editor_lines: vec![VisualRow {
                stops: vec![(0, 0)],
            }],
            dirty: false,
            notice_scroll: 0,
            notice_lines: 0,
            accent: crate::theme::Theme::load().startup_fallback_accent,
            pending_compose_close: false,
            menu_index: crate::menu::INITIAL_FOCUS,
            app_origin: Screen::Home,
            preferences: Preferences::default(),
            preferences_dirty: false,
            settings_page: SettingsPage::Root,
            theme_preview: 1,
            browser_view: BrowserView::Start,
            browser_settings_origin: BrowserView::Start,
            browser_data: Default::default(),
            browser_dirty: false,
            requested_url: String::new(),
        }
    }
    pub fn go(&mut self, s: Screen) {
        self.music.event(Event {
            key: Key::End,
            phase: Phase::Cancel,
        });
        if self.screen == Screen::Menu {
            self.menu_index = self.index;
        }
        if s == Screen::Menu && self.screen == Screen::Home {
            self.menu_index = crate::menu::INITIAL_FOCUS;
        }
        if self.screen == Screen::Snake && s != Screen::Snake {
            self.snake.running = false;
            self.snake_view = snake::SnakeView::Menu;
        }
        self.screen = s;
        self.index = if s == Screen::Menu {
            self.menu_index
        } else {
            0
        };
        self.overlay = None;
    }
    pub fn launch(&mut self, screen: Screen) {
        self.app_origin = if self.screen == Screen::Menu {
            Screen::Menu
        } else {
            Screen::Home
        };
        if let Screen::Tools(page) = screen {
            self.tools.page = page;
        }
        if let Screen::Organizer(page) = screen {
            self.organizer.page = page;
        }
        if screen == Screen::Contacts {
            self.organizer.page = organizer::OrganizerPage::Contacts;
        }
        if screen == Screen::Settings {
            self.settings_page = SettingsPage::Root;
        }
        self.go(screen);
    }
    pub fn leave_app(&mut self) {
        self.go(self.app_origin);
    }
    pub fn apply_preferences(&mut self, preferences: Preferences) {
        self.accent = preferences.accent;
        self.preferences = preferences;
    }
    pub fn notice(&mut self, s: impl Into<String>) {
        self.notice_scroll = 0;
        self.notice_lines = 0;
        self.overlay = Some(Overlay::Notice(s.into()));
    }
    pub fn menu_labels(&self, kind: MenuKind) -> Vec<String> {
        match kind {
            MenuKind::Sms => vec!["插入选项", "附件", "移除附件", "清除文字", "另存为草稿"],
            MenuKind::SmsRecipient => vec!["通讯录", "群组"],
            MenuKind::SmsInsert => vec!["插入表情符号", "插入符号", "插入模板"],
            MenuKind::Smileys => vec![
                ":)", ";)", ":D", ":P", ":*", ":o", ":|", ":/", "8)", ":(", ">:(", ">:o",
            ],
            MenuKind::Symbols => vec![
                ".", ",", "'", "?", "!", "\"", "-", "(", ")", "@", "/", ":", "_", ";", "+", "&",
                "%", "*", "=", "<", ">", "£", "€", "$", "¥", "₹", "¤", "[", "]", "{", "}", "\\",
                "~", "^", "`", "¡", "¿", "§", "#", "|", "", "\n",
            ],
            MenuKind::Templates => vec![
                "现在无法通话。我将稍后呼叫。",
                "我的电池电量低。请给我发送信息。",
                "请给我办公室拨电话。",
                "我在开会，晚些给你打电话。",
                "我要迟到了！到达时间为",
                "预计到达时间为",
                "老地方见。",
            ],
            MenuKind::ExitSms => vec!["保存并关闭", "放弃修改", "继续编辑"],
            MenuKind::Browser => vec![
                "打开",
                "导航",
                "保存",
                "下载管理",
                "分享链接",
                "页面详情",
                "设置",
                "关闭",
            ],
            MenuKind::BrowserRoot => vec!["主页", "设置", "关闭"],
            MenuKind::BrowserOpen => vec!["主页", "输入URL", "书签", "历史", "离线页面"],
            MenuKind::BrowserNavigation => vec!["刷新", "停止", "前进", "返回"],
            MenuKind::BrowserSave => vec!["添加书签", "保存离线页面"],
            MenuKind::Music => vec![
                "重复",
                "随机播放",
                "播放队列",
                "添加到播放列表",
                "设为铃声",
                "分享",
            ],
            MenuKind::MusicLibrary => vec!["手动刷新"],
        }
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
    pub fn menu_columns(kind: MenuKind) -> usize {
        match kind {
            MenuKind::Symbols => 7,
            MenuKind::Smileys => 4,
            _ => 1,
        }
    }
    pub fn menu_enabled(&self, kind: MenuKind, index: usize) -> bool {
        match kind {
            MenuKind::Sms => index != 2 && (index < 3 || !self.compose.body.units.is_empty()),
            MenuKind::Symbols => index != 40,
            MenuKind::BrowserNavigation => match index {
                1 => self.browser.loading,
                2 => self.browser.can_forward(),
                3 => self.browser.can_back(),
                _ => true,
            },
            _ => true,
        }
    }
    pub fn editor(&self) -> Option<&Editor> {
        match &self.overlay {
            Some(Overlay::Text { editor, .. }) => Some(editor),
            Some(_) => None,
            None => match self.screen {
                Screen::Compose => Some(if self.compose.recipient {
                    &self.compose.to
                } else {
                    &self.compose.body
                }),
                Screen::Dial => Some(&self.dial),
                _ => None,
            },
        }
    }
    pub fn editor_mut(&mut self) -> Option<&mut Editor> {
        match &mut self.overlay {
            Some(Overlay::Text { editor, .. }) => Some(editor),
            Some(_) => None,
            None => match self.screen {
                Screen::Compose => Some(if self.compose.recipient {
                    &mut self.compose.to
                } else {
                    &mut self.compose.body
                }),
                Screen::Dial => Some(&mut self.dial),
                _ => None,
            },
        }
    }
    pub fn text(&mut self, s: &str) {
        if let Some(e) = self.editor_mut()
            && !e.insert(s)
        {
            self.status = "Text limit reached".into();
        }
    }
    pub fn tab(&mut self) {
        if self.screen == Screen::Compose && self.overlay.is_none() {
            if !self.compose.to.accept_prediction() || !self.compose.body.accept_prediction() {
                self.status = "Text limit reached".into();
                return;
            }
            self.compose.to.commit();
            self.compose.body.commit();
            self.compose.recipient = !self.compose.recipient;
        }
    }
    pub fn input_mode(&mut self) {
        if let Some(e) = self.editor_mut() {
            e.cycle_mode();
        }
    }
    fn prediction_key(&mut self, key: Key) -> bool {
        let Some(e) = self.editor_mut() else {
            return false;
        };
        if !e.prediction.active() {
            return false;
        }
        match key {
            Key::Up => e.prediction.cycle(-1),
            Key::Down | Key::Star => e.prediction.cycle(1),
            Key::Left if !e.prediction.spellings.is_empty() => e.prediction.spelling_focus = true,
            Key::Right if !e.prediction.spellings.is_empty() => e.prediction.spelling_focus = false,
            Key::Left => e.prediction.cycle(-1),
            Key::Right => e.prediction.cycle(1),
            Key::Center | Key::SoftLeft => {
                if e.prediction.spelling_focus {
                    e.prediction.spelling_focus = false;
                    return true;
                }
                if !e.accept_prediction() {
                    self.status = "Text limit reached".into();
                }
            }
            Key::SoftRight | Key::Delete => e.backspace(),
            Key::End => e.prediction.clear(),
            _ => return false,
        }
        true
    }
    pub fn tick(&mut self, now: u64, dt: f32) {
        let elapsed = self
            .last_tick
            .replace(now)
            .map_or(0, |last| now.saturating_sub(last));
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.check_alarms(
            phone_core::calendar::Date::from_unix(wall, 8 * 3600),
            ((wall + 8 * 3600) / 60) % 1440,
        );
        self.snake.tick(elapsed);
        if self.snake.over && self.snake_view == snake::SnakeView::Game {
            self.snake_view = snake::SnakeView::Menu;
            self.index = 0;
            self.notice(format!("游戏结束\n得分 {}", self.snake.score));
        }
        self.tools.stopwatch.tick(elapsed);
        self.tools.timer.tick(elapsed);
        if self.tools.recording {
            self.tools.record_ms = self.tools.record_ms.saturating_add(elapsed);
        }
        if self.tools.timer.expired {
            self.tools.timer.expired = false;
            self.notice(format!("{}\n时间到", self.tools.timer_name));
        }

        if let Some(e) = self.editor_mut() {
            e.tick(now);
        }
        if self.screen != Screen::Music
            || self.music_view != MusicView::Player
            || self.overlay.is_some()
        {
            self.music.event(Event {
                key: Key::End,
                phase: Phase::Cancel,
            });
        }
        self.music.tick(dt);
    }
    pub fn handle(&mut self, event: Event, now: u64) {
        let Event { key, phase } = event;
        if phase == Phase::Cancel {
            self.music.event(event);
            if let Some(e) = self.editor_mut() {
                e.commit();
            }
            return;
        }
        if self.screen == Screen::Music
            && self.music_view == MusicView::Player
            && self.overlay.is_none()
        {
            self.music.event(event);
        }
        if phase == Phase::Long {
            if let Key::Digit(d) = key
                && let Some(e) = self.editor_mut()
            {
                e.long_digit(d);
            }
            return;
        }
        if !matches!(phase, Phase::Down | Phase::Repeat) {
            return;
        }
        if phase == Phase::Repeat
            && !matches!(
                key,
                Key::Up | Key::Down | Key::Left | Key::Right | Key::Delete
            )
        {
            return;
        }
        if self.prediction_key(key) {
            return;
        }
        if self.overlay.is_some() {
            self.overlay_event(key, now);
            return;
        }
        if key == Key::End {
            if self.screen == Screen::Compose {
                self.close_compose();
            } else {
                self.go(Screen::Home);
            }
            return;
        }
        if self.screen == Screen::Compose && self.overlay.is_none() {
            if key == Key::Star && !self.compose.recipient {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::Symbols,
                    index: 0,
                });
                return;
            }
            if key == Key::Down && self.compose.recipient {
                self.compose.recipient = false;
                return;
            }
            if key == Key::Up
                && !self.compose.recipient
                && self.editor_lines.first().is_some_and(|r| {
                    r.stops
                        .iter()
                        .any(|(offset, _)| *offset == self.compose.body.cursor)
                })
            {
                self.compose.recipient = true;
                return;
            }
        }
        if self.editor().is_some() && self.edit_key(key, now) {
            return;
        }
        match self.screen {
            Screen::Home => match key {
                Key::Center => self.go(Screen::Menu),
                Key::SoftLeft => self.shortcut(true),
                Key::SoftRight => self.shortcut(false),
                Key::Call => self.launch(Screen::Dial),
                Key::Digit(d) => {
                    self.launch(Screen::Dial);
                    self.dial.digit(d, now);
                }
                _ => {}
            },
            Screen::Menu => match key {
                Key::Up | Key::Down | Key::Left | Key::Right => {
                    self.index = crate::menu::navigate(self.index, key)
                }
                Key::Center | Key::SoftLeft => {
                    let entry = &crate::menu::ENTRIES[self.index];
                    if let Some(screen) = entry.screen {
                        self.launch(screen);
                    } else {
                        self.notice(format!(
                            "{}{}",
                            entry.label,
                            if self.index >= 16 {
                                "不在本次复刻范围。"
                            } else {
                                "尚未实现。"
                            }
                        ));
                    }
                }
                Key::SoftRight => self.go(Screen::Home),
                _ => {}
            },
            Screen::Messages => match key {
                Key::Up => self.index = self.index.saturating_sub(1),
                Key::Down => self.index = (self.index + 1).min(2),
                Key::Center | Key::SoftLeft => match self.index {
                    0 => {
                        self.compose = Compose::default();
                        self.go(Screen::Compose);
                    }
                    1 => self.go(Screen::Drafts),
                    _ => self.go(Screen::Outbox),
                },
                Key::SoftRight => self.leave_app(),
                _ => {}
            },
            Screen::Drafts | Screen::Outbox => self.mail_list_key(key),
            Screen::Compose => match key {
                Key::Center if self.compose.recipient => self.compose.recipient = false,
                Key::Center => self.queue_message(),
                Key::SoftLeft => {
                    self.overlay = Some(Overlay::Menu {
                        kind: if self.compose.recipient {
                            MenuKind::SmsRecipient
                        } else {
                            MenuKind::Sms
                        },
                        index: 0,
                    })
                }
                Key::SoftRight => {
                    if self.editor().is_some_and(|e| !e.units.is_empty()) {
                        self.editor_mut().unwrap().backspace();
                    } else {
                        self.close_compose();
                    }
                }
                Key::Up if self.compose.recipient => {}
                _ => {}
            },
            Screen::Browser => self.browser_key(key),
            Screen::Music => self.music_key(key),
            Screen::Snake => self.snake_key(key),
            Screen::Dial => match key {
                Key::Call | Key::Center | Key::SoftLeft => {
                    self.notice("Phone service is not connected. No call was made.")
                }
                Key::SoftRight => {
                    if self.dial.units.is_empty() {
                        self.leave_app();
                    } else {
                        self.dial.backspace();
                    }
                }
                _ => {}
            },
            Screen::Contacts | Screen::Organizer(_) => self.organizer_key(key),
            Screen::Settings => self.settings_key(key),
            Screen::Tools(_) => self.tools_key(key),
        }
    }
    pub(super) fn open_local_text(&mut self, target: TextTarget, value: String, numeric: bool) {
        let mut editor = Editor::new(&value, 128);
        if numeric {
            editor.numeric = true;
        } else {
            editor
                .prediction
                .set_language(Some(phone_core::predict::Language::Chinese));
        }
        self.overlay = Some(Overlay::Text {
            target,
            editor: Box::new(editor),
        });
    }
    fn edit_key(&mut self, key: Key, now: u64) -> bool {
        let lines = self.editor_lines.clone();
        let address = matches!(
            self.overlay,
            Some(Overlay::Text {
                target: TextTarget::Address,
                ..
            })
        );
        let e = self.editor_mut().unwrap();
        match key {
            Key::Digit(d) => e.digit(d, now),
            Key::Star => {
                e.insert("*");
            }
            Key::Hash => {
                self.input_mode();
            }
            Key::Delete => e.backspace(),
            Key::Left if address && e.cursor == 0 => {
                e.commit();
                e.cursor = e.units.len();
            }
            Key::Right if address && e.cursor == e.units.len() => {
                e.commit();
                e.cursor = 0;
            }
            Key::Left => e.move_horizontal(-1),
            Key::Right => e.move_horizontal(1),
            Key::Up => e.vertical_visual(-1, &lines),
            Key::Down => e.vertical_visual(1, &lines),
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
#[path = "phone_tests.rs"]
mod tests;
