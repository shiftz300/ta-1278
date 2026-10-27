pub mod alarm;
pub mod browser;
pub mod calendar;
pub mod contacts;
pub mod conversation;
pub mod converter;
mod editor;
pub mod files;
pub mod mail;
pub mod media;
mod music;
pub mod organizer;
mod overlays;
mod sms;
pub mod snake;
mod timer;
pub mod tools;
pub use music::MusicView;
#[cfg(any(feature = "desktop", test))]
pub use music::Playlist;
pub mod radio;
pub mod settings;
use crate::ui::navigation::{Boundary, vertical};
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
    Conversation,
    Drafts,
    Inbox,
    Outbox,
    Mail(mail::MailPage),
    Compose,
    Browser,
    Music,
    Media,
    Snake,
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
    FileName(bool),
    Setting(usize),
    ContactGroup,
    ContactGroupRename,
    PlaylistName,
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
    TimerElapsed,
    ServiceAlert {
        backend: String,
        id: String,
        title: String,
        body: String,
    },
}
pub struct Compose {
    pub to: Editor,
    pub body: Editor,
    pub recipient: bool,
    pub draft: Option<usize>,
    pub time: u64,
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
            time: phone_core::sms::timestamp(),
        }
    }
}
pub struct Phone {
    pub(crate) wall_ms: Option<u64>,
    pub music_paths: Vec<String>,
    music_lookup: std::collections::HashMap<String, usize>,
    pub playlist: usize,
    pub playlist_row: usize,
    pub cues: Vec<crate::effects::Cue>,
    pub audio_preview: Option<String>,
    pub media_ui: media::MediaBrowser,
    pub pictures: std::collections::HashMap<String, image::RgbaImage>,
    pub features: crate::config::Features,
    pub apps: crate::config::Apps,
    pub files: files::Files,
    pub music_refresh: bool,
    pub music_loading: bool,
    pub music_wait_dismissed: bool,
    pub music_return_file: bool,
    pub screen: Screen,
    pub index: usize,
    pub overlay: Option<Overlay>,
    pub mailbox: Mailbox,
    pub service_backend: Option<String>,
    pub prediction_engine: std::sync::Arc<dyn phone_core::predict::Engine>,
    pub compose: Compose,
    pub mail_ui: mail::MailUi,
    pub chat: conversation::ChatUi,
    pub browser: Browser,
    pub music: Music,
    pub snake: phone_core::snake::Snake,
    pub snake_view: snake::SnakeView,
    pub snake_level_preview: u8,
    pub snake_data: snake::SnakeData,
    pub snake_dirty: bool,
    pub tools: tools::Tools,
    pub organizer: organizer::Organizer,
    pub local_data: organizer::LocalData,
    pub local_dirty: bool,
    pub(crate) last_tick: Option<u64>,
    pub music_view: MusicView,
    pub music_started: bool,
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
    pub browser_data: browser::data::BrowserData,
    pub browser_dirty: bool,
    pub browser_clear: [bool; 2],
    pub requested_url: String,
}
impl Phone {
    pub fn new(mailbox: Mailbox) -> Self {
        Self {
            wall_ms: None,
            music_paths: vec![],
            music_lookup: Default::default(),
            playlist: 0,
            playlist_row: 0,
            cues: vec![],
            audio_preview: None,
            media_ui: Default::default(),
            pictures: Default::default(),
            features: Default::default(),
            apps: Default::default(),
            files: Default::default(),
            music_refresh: false,
            music_loading: false,
            music_wait_dismissed: false,
            music_return_file: false,
            screen: Screen::Home,
            index: 0,
            overlay: None,
            mailbox,
            service_backend: None,
            prediction_engine: phone_core::predict::default_engine(),
            compose: Compose::default(),
            mail_ui: mail::MailUi::default(),
            chat: Default::default(),
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
            snake_level_preview: 1,
            snake_data: Default::default(),
            snake_dirty: false,
            tools: Default::default(),
            organizer: Default::default(),
            local_data: Default::default(),
            local_dirty: false,
            last_tick: None,
            music_view: MusicView::Root,
            music_started: false,
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
            browser_clear: [false, true],
            requested_url: String::new(),
        }
    }
    pub fn timestamp(&self) -> u64 {
        self.wall_ms.unwrap_or_else(phone_core::sms::timestamp)
    }
    pub fn new_compose(&self) -> Compose {
        Compose {
            time: self.timestamp(),
            ..Compose::default()
        }
    }
    pub fn go(&mut self, s: Screen) {
        if crate::menu::phone_disabled(s) || !self.apps.allows(s) {
            return;
        }
        if (s == Screen::Music && !self.features.music)
            || (s == Screen::Tools(tools::ToolPage::Files) && !self.features.files)
            || (s == Screen::Tools(tools::ToolPage::Gallery)
                && (!self.features.files || !self.features.pictures))
            || (s == Screen::Tools(tools::ToolPage::Video)
                && (!self.features.files || !self.features.video))
        {
            self.notice("此功能已停用。");
            return;
        }
        if s != Screen::Tools(tools::ToolPage::Files)
            && !self
                .files
                .pending
                .as_ref()
                .is_some_and(files::FileRequest::mutating)
        {
            self.files.pending = None;
        }
        self.music.event(Event {
            key: Key::End,
            phase: Phase::Cancel,
        });
        if self.screen == Screen::Menu {
            self.menu_index = self.index;
        }
        if s == Screen::Menu && self.screen == Screen::Home {
            self.menu_index = crate::menu::visible(self.apps)
                .iter()
                .position(|e| e.screen == Screen::Music)
                .unwrap_or(0);
        }
        if self.screen == Screen::Snake && s != Screen::Snake {
            self.snake.running = false;
            self.snake_view = snake::SnakeView::Menu;
        }
        self.screen = s;
        self.index = if s == Screen::Menu {
            self.menu_index
                .min(crate::menu::visible(self.apps).len().saturating_sub(1))
        } else {
            0
        };
        self.overlay = None;
    }
    pub fn launch(&mut self, screen: Screen) {
        if screen == Screen::Tools(tools::ToolPage::Files)
            && (self.files.busy
                || self
                    .files
                    .pending
                    .as_ref()
                    .is_some_and(files::FileRequest::mutating))
        {
            self.notice("文件操作尚未结束，请稍候。");
            return;
        }
        if crate::menu::phone_disabled(screen) || !self.apps.allows(screen) {
            return;
        }
        if screen == Screen::Music {
            self.music_return_file = false;
        }
        self.app_origin = if self.screen == Screen::Menu {
            Screen::Menu
        } else {
            Screen::Home
        };
        if let Screen::Tools(page) = screen {
            self.tools.page = page;
            if page == tools::ToolPage::Timer {
                self.open_timer();
            }
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
        if let Screen::Tools(page @ (tools::ToolPage::Gallery | tools::ToolPage::Video)) = screen {
            self.open_media(if page == tools::ToolPage::Video {
                media::Purpose::Videos
            } else {
                media::Purpose::Pictures
            });
            self.media_ui.origin = self.app_origin;
        }
        if screen == Screen::Tools(tools::ToolPage::Files) && self.features.files {
            self.files = Default::default();
            self.files.pending = Some(files::FileRequest::Storage);
        }
        if screen == Screen::Organizer(organizer::OrganizerPage::Alarms)
            && !self.local_data.alarms.is_empty()
        {
            self.index = 1;
        }
    }
    pub fn leave_app(&mut self) {
        self.go(self.app_origin);
    }
    pub fn apply_preferences(&mut self, preferences: Preferences) {
        self.accent = preferences.accent;
        self.preferences = preferences;
        self.organizer.date = self.local_clock().0;
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
            MenuKind::BrowserSave => vec!["作为主页", "书签", "网页", "图像"],
            MenuKind::Music => vec![
                "重复",
                "随机播放",
                "播放队列",
                "添加到播放列表",
                "设为铃声",
                "分享",
            ],
            MenuKind::MusicLibrary => vec!["手动更新"],
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
            // Native action 31 is present, but this text adapter has no image backend.
            MenuKind::BrowserSave => index < 3 && !self.browser.page.url.is_empty(),
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
    pub fn text(&mut self, s: &str) {
        if self.conversion_active() {
            self.conversion_text(s);
            return;
        }
        if let Some(e) = self.editor_mut()
            && !e.insert(s)
        {
            self.status = "Text limit reached".into();
        }
    }
    pub fn tab(&mut self) {
        if self.active_reminder().is_some() || self.mail_ui.pending.is_some() {
            return;
        }
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
                if !e.choose_prediction() {
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
        let (date, minute) = self.local_clock();
        self.check_alarms(date, minute);
        self.check_events(date, minute);
        let segments = self.snake.body.len();
        self.snake.tick(elapsed);
        if self.snake_data.sound && self.snake.body.len() > segments {
            self.cues.push(crate::effects::Cue::Food);
        }
        if self.snake.over && self.snake_view == snake::SnakeView::Game {
            if self.snake_data.sound {
                self.cues.push(crate::effects::Cue::Over);
            }
            self.snake_data.record(self.snake.kind, self.snake.score);
            self.snake_dirty = true;
            self.snake_view = snake::SnakeView::GameOver;
            self.index = 0;
        }
        if self.apps.utilities {
            self.tools.stopwatch.tick(elapsed);
        }
        let timer_expired = self.tools.timer.expired;
        if self.apps.utilities {
            self.tools.timer.tick(elapsed);
        }
        if self.tools.timer.expired != timer_expired {
            self.tools.dirty = true;
        }
        if self.apps.recorder && self.tools.recording {
            self.tools.record_ms = self.tools.record_ms.saturating_add(elapsed);
        }
        if self.apps.utilities
            && self.tools.timer.expired
            && !self.tools.timer_notice
            && self.overlay.is_none()
            && self.active_reminder().is_none()
        {
            self.cues.push(crate::effects::Cue::Alert);
            self.overlay = Some(Overlay::TimerElapsed);
            self.tools.timer_notice = true;
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
        if self.active_reminder().is_some() {
            if phase == Phase::Down {
                self.alarm_alert_key(key);
            }
            return;
        }
        if self.mail_ui.pending.is_some() {
            return;
        }
        if phase == Phase::Cancel {
            self.music.event(event);
            if let Some(e) = self.editor_mut() {
                e.commit();
            }
            return;
        }
        if self.screen == Screen::Music
            && self.music_view == MusicView::Player
            && !self.music_loading
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
        if phase == Phase::Down && self.preferences.keypad_sound && self.cues.len() < 8 {
            self.cues.push(crate::effects::Cue::Key);
        }
        if phase == Phase::Repeat
            && !matches!(
                key,
                Key::Up | Key::Down | Key::Left | Key::Right | Key::Delete
            )
        {
            return;
        }
        if matches!(key, Key::SoftRight | Key::End) {
            self.audio_preview = Some("无".into());
        }
        if self.prediction_key(key) {
            return;
        }
        if self.overlay.is_some() {
            self.overlay_event(key, now);
            return;
        }
        if key == Key::End
            && matches!(self.screen, Screen::Conversation | Screen::Messages)
            && self.chat.popup != conversation::Popup::None
        {
            self.chat.popup = conversation::Popup::None;
            return;
        }
        if key == Key::End {
            if matches!(self.screen, Screen::Compose | Screen::Conversation) {
                self.close_compose();
            } else {
                self.go(Screen::Home);
            }
            return;
        }
        if self.screen == Screen::Conversation && self.chat_event(key) {
            return;
        }
        if matches!(self.screen, Screen::Compose | Screen::Conversation) && self.overlay.is_none() {
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
                && self.screen == Screen::Compose
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
        if self.editor().is_some()
            && !((self.alarm_name_active()
                || self.contact_editor_active()
                || self.event_editor_active()
                || self.timer_name_active())
                && matches!(key, Key::Up | Key::Down))
            && self.edit_key(key, now)
        {
            return;
        }
        match self.screen {
            Screen::Home => match key {
                Key::Center => self.go(Screen::Menu),
                Key::SoftLeft => self.shortcut(true),
                Key::SoftRight => self.shortcut(false),
                Key::Call | Key::Digit(_) => {}
                _ => {}
            },
            Screen::Menu => match key {
                Key::Up | Key::Down | Key::Left | Key::Right => {
                    self.index = crate::menu::navigate(
                        self.index,
                        key,
                        crate::menu::visible(self.apps).len(),
                    )
                }
                Key::Center | Key::SoftLeft => {
                    let entries = crate::menu::visible(self.apps);
                    let entry = entries[self.index.min(entries.len() - 1)];
                    self.launch(entry.screen);
                }
                Key::SoftRight => self.go(Screen::Home),
                _ => {}
            },
            Screen::Messages => self.messages_key(key),
            Screen::Drafts | Screen::Outbox | Screen::Inbox => self.mail_list_key(key),
            Screen::Mail(page) => self.mail_key(page, key),
            Screen::Compose | Screen::Conversation => match key {
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
            Screen::Media => self.media_key(key),
            Screen::Snake => self.snake_key(key),
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
mod tests;
