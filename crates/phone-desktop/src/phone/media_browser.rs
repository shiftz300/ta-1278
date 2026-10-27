//! One local folder browser for media libraries and callers choosing a file.
use super::*;
use crate::local_files::Entry;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Pictures,
    Videos,
    ContactPicture,
    ContactRingtone,
    EventRingtone,
    AlarmRingtone,
    Wallpaper,
    Ringtone,
}
impl Purpose {
    pub fn accepts(self, path: &str) -> bool {
        let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        match self {
            Self::Videos => matches!(ext.as_str(), "mp4" | "m4v" | "mov" | "3gp"),
            Self::ContactRingtone | Self::Ringtone | Self::EventRingtone | Self::AlarmRingtone => {
                crate::media::supported(path)
            }
            _ => matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "heic"
            ),
        }
    }
    pub fn picker(self) -> bool {
        !matches!(self, Self::Pictures | Self::Videos)
    }
}
pub enum Request {
    List(String),
    Open(String),
    Command(String),
    Stop,
}
pub struct MediaBrowser {
    pub purpose: Purpose,
    pub path: String,
    pub entries: Vec<Entry>,
    pub history: Vec<(String, usize)>,
    pub origin: Screen,
    pub return_index: usize,
    pub selected: String,
    pub viewing: bool,
    pub direct: bool,
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
    pub volume: u8,
    pub zoom: u8,
    pub pan: (i32, i32),
    pub frame: Option<image::RgbaImage>,
    pub pending: Vec<Request>,
}
impl Default for MediaBrowser {
    fn default() -> Self {
        Self {
            purpose: Purpose::Pictures,
            path: "/".into(),
            entries: vec![],
            history: vec![],
            origin: Screen::Home,
            return_index: 0,
            selected: String::new(),
            viewing: false,
            direct: false,
            playing: false,
            position: 0.,
            duration: 0.,
            volume: 6,
            zoom: 1,
            pan: (0, 0),
            frame: None,
            pending: vec![],
        }
    }
}
impl Phone {
    pub fn open_media(&mut self, purpose: Purpose) {
        if (purpose == Purpose::Videos && !self.features.video)
            || (!matches!(
                purpose,
                Purpose::Videos
                    | Purpose::Ringtone
                    | Purpose::ContactRingtone
                    | Purpose::EventRingtone
                    | Purpose::AlarmRingtone
            ) && !self.features.pictures)
        {
            self.notice("此功能已停用。");
            return;
        }
        if !self.features.files {
            self.notice("文件功能已停用。");
            return;
        }
        let origin = self.screen;
        let index = self.index;
        self.go(Screen::Media);
        self.media_ui = MediaBrowser {
            purpose,
            origin,
            return_index: index,
            pending: vec![Request::List("/".into())],
            ..Default::default()
        };
    }
    pub fn close_media(&mut self) {
        let origin = self.media_ui.origin;
        let index = self.media_ui.return_index;
        self.media_ui.pending.push(Request::Stop);
        self.media_ui.playing = false;
        self.go(origin);
        self.index = index;
    }
    fn choose_media(&mut self, path: String) {
        match self.media_ui.purpose {
            Purpose::ContactPicture => self.organizer.draft[0] = path,
            Purpose::ContactRingtone => {
                self.audio_preview = Some(path.clone());
                self.organizer.draft[7] = path;
            }
            Purpose::EventRingtone => {
                self.audio_preview = Some(path.clone());
                self.organizer.draft[4] = path;
            }
            Purpose::AlarmRingtone => {
                self.audio_preview = Some(path.clone());
                self.organizer.draft[3] = path;
            }
            Purpose::Wallpaper => {
                self.preferences.wallpaper = Some(path);
                self.preferences.wallpaper_enabled = true;
                self.preferences_dirty = true;
            }
            Purpose::Ringtone => {
                self.audio_preview = Some(path.clone());
                self.preferences.ringtone = path;
                self.preferences_dirty = true;
            }
            _ => return,
        }
        self.close_media();
    }
    pub(super) fn media_key(&mut self, key: Key) {
        use Request::*;
        if self.media_ui.viewing {
            let video = self.media_ui.purpose == Purpose::Videos;
            match key {
                Key::SoftRight => {
                    if self.media_ui.direct {
                        self.close_media();
                        return;
                    }
                    self.media_ui.viewing = false;
                    self.media_ui.playing = false;
                    self.media_ui.pending.push(Stop);
                }
                Key::Center if self.media_ui.purpose.picker() && self.media_ui.frame.is_some() => {
                    self.choose_media(self.media_ui.selected.clone())
                }
                Key::Center if self.media_ui.purpose.picker() => {}
                Key::Center if video => {
                    self.media_ui.playing = !self.media_ui.playing;
                    self.media_ui.pending.push(Command(
                        if self.media_ui.playing {
                            "play"
                        } else {
                            "pause"
                        }
                        .into(),
                    ));
                }
                Key::Left | Key::Right if video => {
                    let delta = if key == Key::Left { -5. } else { 5. };
                    let pos =
                        (self.media_ui.position + delta).clamp(0., self.media_ui.duration.max(0.));
                    self.media_ui.pending.push(Command(format!("seek {pos}")));
                }
                Key::Up | Key::Down if video => {
                    self.media_ui.volume = if key == Key::Up {
                        (self.media_ui.volume + 1).min(10)
                    } else {
                        self.media_ui.volume.saturating_sub(1)
                    };
                    self.media_ui
                        .pending
                        .push(Command(format!("volume {}", self.media_ui.volume)));
                }
                Key::Center if !video => {
                    self.media_ui.zoom = if self.media_ui.zoom == 4 {
                        1
                    } else {
                        self.media_ui.zoom * 2
                    };
                    self.media_ui.pan = (0, 0);
                }
                Key::Left | Key::Right
                    if self.media_ui.zoom == 1 && !self.media_ui.purpose.picker() =>
                {
                    let files: Vec<_> = self
                        .media_ui
                        .entries
                        .iter()
                        .filter(|e| !e.directory)
                        .map(|e| e.path.clone())
                        .collect();
                    if !files.is_empty() {
                        let i = files
                            .iter()
                            .position(|p| p == &self.media_ui.selected)
                            .unwrap_or(0);
                        let next = if key == Key::Left {
                            (i + files.len() - 1) % files.len()
                        } else {
                            (i + 1) % files.len()
                        };
                        self.media_ui.selected = files[next].clone();
                        self.media_ui.frame = None;
                        self.media_ui.pending.push(Open(files[next].clone()));
                    }
                }
                Key::Up => self.media_ui.pan.1 += 16,
                Key::Down => self.media_ui.pan.1 -= 16,
                Key::Left => self.media_ui.pan.0 += 16,
                Key::Right => self.media_ui.pan.0 -= 16,
                _ => {}
            }
            return;
        }
        let n = self.media_ui.entries.len();
        match key {
            Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
            Key::Down if n > 0 => self.index = (self.index + 1) % n,
            Key::SoftRight => {
                if let Some((path, index)) = self.media_ui.history.pop() {
                    self.media_ui.pending.push(List(path));
                    self.index = index;
                } else {
                    self.close_media();
                }
            }
            Key::Center | Key::SoftLeft if n > 0 => {
                let e = self.media_ui.entries[self.index.min(n - 1)].clone();
                if e.directory {
                    self.media_ui
                        .history
                        .push((self.media_ui.path.clone(), self.index));
                    self.index = 0;
                    self.media_ui.pending.push(List(e.path));
                } else if self.media_ui.purpose.picker()
                    && matches!(
                        self.media_ui.purpose,
                        Purpose::Ringtone
                            | Purpose::ContactRingtone
                            | Purpose::EventRingtone
                            | Purpose::AlarmRingtone
                    )
                {
                    self.choose_media(e.path);
                } else {
                    self.media_ui.selected = e.path.clone();
                    self.media_ui.frame = None;
                    self.media_ui.viewing = true;
                    self.media_ui.zoom = 1;
                    self.media_ui.pan = (0, 0);
                    self.media_ui.pending.push(Open(e.path));
                }
            }
            _ => {}
        }
    }
}
