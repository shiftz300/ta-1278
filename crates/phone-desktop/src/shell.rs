pub mod alarm;
mod alerts;
pub mod browser;
pub mod calendar;
pub mod calls;
pub mod contacts;
pub mod conversation;
pub mod converter;
mod editor;
pub mod files;
pub mod mail;
pub mod media;
mod menu;
pub mod music;
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
    Dial,
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
pub struct Shell {
    pub contacts: contacts::State,
    pub records_dirty: bool,
    pub messaging: mail::State,
    pub browsing: browser::State,
    pub playback: music::State,
    pub gaming: snake::State,
    pub settings: settings::State,

    pub(crate) wall_ms: Option<u64>,
    pub cues: Vec<crate::effects::Cue>,
    pub audio_preview: Option<String>,
    pub media: media::State,
    pub features: crate::config::Features,
    pub apps: crate::config::Apps,
    pub files: files::Files,
    pub screen: Screen,
    pub index: usize,
    pub overlay: Option<Overlay>,
    pub prediction_engine: std::sync::Arc<dyn phone_core::predict::Engine>,
    pub tools: tools::Tools,
    pub calls: calls::Calls,
    pub organizer: organizer::Organizer,
    pub(crate) last_tick: Option<u64>,
    pub status: String,
    pub editor_lines: Vec<VisualRow>,
    pub notice_scroll: usize,
    pub notice_lines: usize,
    menu_index: usize,
    app_origin: Screen,
}
impl Shell {
    pub fn new(mailbox: Mailbox) -> Self {
        Self {
            wall_ms: None,
            cues: vec![],
            audio_preview: None,
            media: Default::default(),
            features: Default::default(),
            apps: Default::default(),
            files: Default::default(),
            screen: Screen::Home,
            index: 0,
            overlay: None,
            prediction_engine: phone_core::predict::default_engine(),
            tools: Default::default(),
            calls: Default::default(),
            organizer: Default::default(),
            last_tick: None,
            status: String::new(),
            editor_lines: vec![VisualRow {
                stops: vec![(0, 0)],
            }],
            notice_scroll: 0,
            notice_lines: 0,
            menu_index: crate::menu::INITIAL_FOCUS,
            app_origin: Screen::Home,
            contacts: Default::default(),
            records_dirty: false,
            messaging: mail::State {
                mailbox,
                ..Default::default()
            },
            browsing: Default::default(),
            playback: Default::default(),
            gaming: Default::default(),
            settings: Default::default(),
        }
    }
    pub fn records(&self) -> organizer::LocalData {
        organizer::LocalData {
            schema_version: 1,
            contacts: self.contacts.records.clone(),
            groups: self.contacts.groups.clone(),
            playlists: self.playback.playlists.clone(),
            calls: self.calls.records.clone(),
            events: self.organizer.events.clone(),
            alarms: self.organizer.alarms.clone(),
            reminders: self.organizer.reminders.clone(),
        }
    }
    pub fn apply_records(&mut self, data: organizer::LocalData) {
        self.contacts.records = data.contacts;
        self.contacts.groups = data.groups;
        self.playback.playlists = data.playlists;
        self.calls.records = data.calls;
        self.organizer.events = data.events;
        self.organizer.alarms = data.alarms;
        self.organizer.reminders = data.reminders;
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
        if !self.apps.allows(s) {
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
        self.playback.model.event(Event {
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
            self.gaming.model.running = false;
            self.gaming.view = snake::SnakeView::Menu;
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
        if !self.apps.allows(screen) {
            return;
        }
        if screen == Screen::Music {
            self.playback.return_file = false;
        }
        self.app_origin = if self.screen == Screen::Menu {
            Screen::Menu
        } else {
            Screen::Home
        };
        if let Screen::Tools(page) = screen {
            self.calls.confirm_delete = false;
            self.tools.page = page;
            if page == tools::ToolPage::Timer {
                self.open_timer();
            }
        }
        if let Screen::Organizer(page) = screen {
            self.organizer.page = page;
        }
        if screen == Screen::Contacts {
            self.contacts.page = contacts::Page::Contacts;
        }
        if screen == Screen::Settings {
            self.settings.page = SettingsPage::Root;
        }
        self.go(screen);
        if let Screen::Tools(page @ (tools::ToolPage::Gallery | tools::ToolPage::Video)) = screen {
            self.open_media(if page == tools::ToolPage::Video {
                media::Purpose::Videos
            } else {
                media::Purpose::Pictures
            });
            self.media.ui.origin = self.app_origin;
        }
        if screen == Screen::Tools(tools::ToolPage::Files) && self.features.files {
            self.files = Default::default();
            self.files.pending = Some(files::FileRequest::Storage);
        }
        if screen == Screen::Organizer(organizer::OrganizerPage::Alarms)
            && !self.organizer.alarms.is_empty()
        {
            self.index = 1;
        }
    }
    pub fn leave_app(&mut self) {
        self.go(self.app_origin);
    }
    pub fn apply_preferences(&mut self, preferences: Preferences) {
        self.settings.accent = preferences.accent;
        self.settings.preferences = preferences;
        self.organizer.date = self.local_clock().0;
    }
    pub fn notice(&mut self, s: impl Into<String>) {
        self.notice_scroll = 0;
        self.notice_lines = 0;
        self.overlay = Some(Overlay::Notice(s.into()));
    }
    pub fn text(&mut self, s: &str) {
        if self.screen == Screen::Tools(tools::ToolPage::Dial)
            && self.overlay.is_none()
            && self.active_reminder().is_none()
        {
            self.dial_text(s);
            return;
        }
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
        if self.active_reminder().is_some() || self.messaging.ui.pending.is_some() {
            return;
        }
        if self.screen == Screen::Compose && self.overlay.is_none() {
            if !self.messaging.compose.to.accept_prediction()
                || !self.messaging.compose.body.accept_prediction()
            {
                self.status = "Text limit reached".into();
                return;
            }
            self.messaging.compose.to.commit();
            self.messaging.compose.body.commit();
            self.messaging.compose.recipient = !self.messaging.compose.recipient;
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
        if let Some((_, remaining)) = &mut self.tools.timer_feedback {
            *remaining = remaining.saturating_sub(elapsed);
            if *remaining == 0 {
                self.tools.timer_feedback = None;
            }
        }
        let (date, minute) = self.local_clock();
        self.check_alarms(date, minute);
        self.check_events(date, minute);
        let segments = self.gaming.model.body.len();
        self.gaming.model.tick(elapsed);
        if self.gaming.data.sound && self.gaming.model.body.len() > segments {
            self.cues.push(crate::effects::Cue::Food);
        }
        if self.gaming.model.over && self.gaming.view == snake::SnakeView::Game {
            if self.gaming.data.sound {
                self.cues.push(crate::effects::Cue::Over);
            }
            self.gaming
                .data
                .record(self.gaming.model.kind, self.gaming.model.score);
            self.gaming.dirty = true;
            self.gaming.view = snake::SnakeView::GameOver;
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
            || self.playback.view != MusicView::Player
            || self.overlay.is_some()
        {
            self.playback.model.event(Event {
                key: Key::End,
                phase: Phase::Cancel,
            });
        }
        self.playback.model.tick(dt);
    }
    pub fn handle(&mut self, event: Event, now: u64) {
        let Event { key, phase } = event;
        if self.active_reminder().is_some() {
            if phase == Phase::Down {
                self.alarm_alert_key(key);
            }
            return;
        }
        if self.messaging.ui.pending.is_some() {
            return;
        }
        if phase == Phase::Cancel {
            self.playback.model.event(event);
            if let Some(e) = self.editor_mut() {
                e.commit();
            }
            return;
        }
        if self.screen == Screen::Music
            && self.playback.view == MusicView::Player
            && !self.playback.loading
            && self.overlay.is_none()
        {
            self.playback.model.event(event);
        }
        if phase == Phase::Long {
            if self.screen == Screen::Tools(tools::ToolPage::Dial) {
                return;
            }
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
        if phase == Phase::Down && self.settings.preferences.keypad_sound && self.cues.len() < 8 {
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
            && self.messaging.chat.popup != conversation::Popup::None
        {
            self.messaging.chat.popup = conversation::Popup::None;
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
            if key == Key::Star && !self.messaging.compose.recipient {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::Symbols,
                    index: 0,
                });
                return;
            }
            if key == Key::Down && self.messaging.compose.recipient {
                self.messaging.compose.recipient = false;
                return;
            }
            if key == Key::Up
                && self.screen == Screen::Compose
                && !self.messaging.compose.recipient
                && self.editor_lines.first().is_some_and(|r| {
                    r.stops
                        .iter()
                        .any(|(offset, _)| *offset == self.messaging.compose.body.cursor)
                })
            {
                self.messaging.compose.recipient = true;
                return;
            }
        }
        if self.screen != Screen::Tools(tools::ToolPage::Dial)
            && self.editor().is_some()
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
                Key::Call => self.launch(Screen::Tools(tools::ToolPage::Calls)),
                Key::Digit(d) if d < 10 => self.open_dialer(d.to_string()),
                Key::Star => self.open_dialer("*".into()),
                Key::Hash => self.open_dialer("#".into()),
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
                Key::Center if self.messaging.compose.recipient => {
                    self.messaging.compose.recipient = false
                }
                Key::Center => self.queue_message(),
                Key::SoftLeft => {
                    self.overlay = Some(Overlay::Menu {
                        kind: if self.messaging.compose.recipient {
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
                Key::Up if self.messaging.compose.recipient => {}
                _ => {}
            },
            Screen::Browser => self.browser_key(key),
            Screen::Music => self.music_key(key),
            Screen::Media => self.media_key(key),
            Screen::Snake => self.snake_key(key),
            Screen::Contacts => {
                self.contacts_key(key);
            }
            Screen::Organizer(_) => self.organizer_key(key),
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
