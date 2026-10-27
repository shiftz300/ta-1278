//! Settings routes observed on firmware 17.00.17.02. Preferences belong to the replica.
mod state;
use super::*;
use serde::{Deserialize, Serialize};
pub use state::State;

pub const SHORTCUTS: [&str; 17] = [
    "无",
    "通讯录",
    "通话历史记录",
    "信息",
    "音乐",
    "视频",
    "收音机",
    "闹钟",
    "文件",
    "多媒体资料",
    "日历",
    "计算器",
    "互联网",
    "录音",
    "设置",
    "其他",
    "计数器",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub accent: usize,
    pub left_shortcut: usize,
    pub right_shortcut: usize,
    pub greeting: String,
    pub wallpaper: Option<String>,
    pub wallpaper_enabled: bool,
    pub ringtone: String,
    pub volume: u8,
    pub silent: bool,
    pub keypad_sound: bool,
    pub timezone: i32,
    pub clock_12h: bool,
    pub clock_adjustment: i64,
    pub mobile_data: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        // Initial replica profile follows the owner's observed shortcuts, not factory defaults.
        Self {
            accent: crate::theme::Theme::load().startup_fallback_accent,
            left_shortcut: 12,
            right_shortcut: 3,
            greeting: String::new(),
            wallpaper: None,
            wallpaper_enabled: true,
            ringtone: "预设".into(),
            volume: 6,
            silent: false,
            keypad_sound: false,
            timezone: 8,
            clock_12h: false,
            clock_adjustment: 0,
            mobile_data: true,
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> Result<(), String> {
        if self.accent >= 6
            || self.left_shortcut >= SHORTCUTS.len()
            || self.right_shortcut >= SHORTCUTS.len()
            || self.volume > 10
            || !(-12..=14).contains(&self.timezone)
            || self.greeting.chars().count() > 128
            || self.clock_adjustment.unsigned_abs() > 100 * 366 * 86400
            || self
                .wallpaper
                .as_ref()
                .is_some_and(|p| !p.starts_with('/') || p.len() > 4096)
        {
            Err("Invalid saved preferences; original file retained".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SettingsPage {
    #[default]
    Root,
    Personalization,
    Keyboard,
    Shortcuts,
    Shortcut {
        left: bool,
    },
    Theme,
    System,
    Connections,
    Time,
    Security,
    Sounds,
    Wallpaper,
    Ringtone,
    Reset,
}
impl Shell {
    pub fn settings_title(&self) -> &'static str {
        match self.settings.page {
            SettingsPage::Root => "设置",
            SettingsPage::Personalization => "个性化选项",
            SettingsPage::Keyboard => "键盘",
            SettingsPage::Shortcuts => "锁定屏幕快捷操作",
            SettingsPage::Shortcut { left: true } => "左键",
            SettingsPage::Shortcut { left: false } => "右键",
            SettingsPage::Theme => "主题",
            SettingsPage::System => "系统",
            SettingsPage::Connections => "连接功能",
            SettingsPage::Time => "时间和语言",
            SettingsPage::Security => "安全选项",
            SettingsPage::Sounds => "声音",
            SettingsPage::Wallpaper => "锁定屏幕背景",
            SettingsPage::Ringtone => "铃声",
            SettingsPage::Reset => "恢复设置",
        }
    }
    pub fn settings_labels(&self) -> Vec<String> {
        if self.settings.page == SettingsPage::Ringtone {
            return crate::audio::native::tone_labels("预设");
        }
        match self.settings.page {
            SettingsPage::Root => vec![
                "系统",
                "连接功能",
                "个性化选项",
                "时间和语言",
                "安全选项",
                "STK",
            ],
            SettingsPage::Personalization => vec!["问候语", "声音", "锁定屏幕背景", "主题", "键盘"],
            SettingsPage::Keyboard => vec!["单键拨号", "锁定屏幕快捷操作"],
            SettingsPage::Shortcuts => vec!["右键", "左键"],
            SettingsPage::Shortcut { .. } => SHORTCUTS.to_vec(),
            SettingsPage::Theme => vec![],
            SettingsPage::System => vec!["关于", "存储", "恢复设置"],
            SettingsPage::Connections => vec!["移动数据", "蓝牙", "USB"],
            SettingsPage::Time => vec!["时间", "日期", "时区", "时间格式"],
            SettingsPage::Security => vec!["SIM 卡 PIN 码", "手机锁"],
            SettingsPage::Sounds => vec!["静音", "铃声音量", "铃声", "按键音"],
            SettingsPage::Wallpaper => vec!["默认", "无", "从文件选择"],
            SettingsPage::Ringtone => vec!["预设", "无", "从文件选择"],
            SettingsPage::Reset => vec!["取消", "恢复设置"],
        }
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
    pub(super) fn settings_open(&mut self, page: SettingsPage, index: usize) {
        self.settings.page = page;
        self.index = index;
        self.settings.first = if matches!(page, SettingsPage::Shortcut { .. }) {
            index
        } else {
            0
        };
    }
    pub(super) fn settings_key(&mut self, key: Key) {
        use SettingsPage::*;
        if self.settings.page == Theme {
            match key {
                Key::Left => self.settings.preview = self.settings.preview.saturating_sub(1),
                Key::Right => self.settings.preview = (self.settings.preview + 1).min(5),
                Key::Center | Key::SoftLeft => {
                    self.settings.accent = self.settings.preview;
                    self.settings.preferences.accent = self.settings.accent;
                    self.settings.dirty = true;
                    self.settings_open(Personalization, 3);
                }
                Key::SoftRight => self.settings_open(Personalization, 3),
                _ => {}
            }
            return;
        }
        let n = self.settings_labels().len();
        match key {
            Key::Up | Key::Down => {
                self.index = if key == Key::Up {
                    (self.index + n - 1) % n
                } else {
                    (self.index + 1) % n
                };
                self.settings.first = self
                    .settings
                    .first
                    .min(self.index)
                    .max(self.index.saturating_sub(5));
            }
            Key::SoftRight => match self.settings.page {
                Root => self.leave_app(),
                Personalization => self.settings_open(Root, 2),
                Keyboard => self.settings_open(Personalization, 4),
                Shortcuts => self.settings_open(Keyboard, 1),
                Shortcut { left } => self.settings_open(Shortcuts, usize::from(left)),
                Theme => unreachable!(),
                System | Connections | Time | Security => self.settings_open(
                    Root,
                    match self.settings.page {
                        System => 0,
                        Connections => 1,
                        Time => 3,
                        _ => 4,
                    },
                ),
                Sounds => self.settings_open(Personalization, 1),
                Wallpaper => self.settings_open(Personalization, 2),
                Ringtone => self.settings_open(Sounds, 2),
                Reset => self.settings_open(System, 2),
            },
            Key::Center | Key::SoftLeft => match (self.settings.page, self.index) {
                _ if !self.setting_enabled(self.index) => {}
                (Root, 0) => self.settings_open(System, 0),
                (Root, 1) => self.settings_open(Connections, 0),
                (Root, 3) => self.settings_open(Time, 0),
                (Root, 4) => self.settings_open(Security, 0),
                (System, 0) => self.notice(
                    "TA-1278 Replica\n本地交互系统\n参考固件 17.00.17.02\n无电话及短信网络服务。",
                ),
                (System, 1) => {
                    self.launch(Screen::Tools(tools::ToolPage::Files));
                    self.files.return_to = Some((Screen::Settings, 1));
                }
                (System, 2) => self.settings_open(Reset, 0),
                (Reset, 0) => self.settings_open(System, 2),
                (Reset, 1) => {
                    self.apply_preferences(Preferences::default());
                    self.settings.dirty = true;
                    self.settings_open(Root, 0);
                }
                (Connections, 0) => {
                    self.settings.preferences.mobile_data = !self.settings.preferences.mobile_data;
                    self.settings.dirty = true;
                }
                (Connections, 2) => self.notice("存储由 local folder 提供。USB 设备模式不可用。"),
                (Personalization, 0) => self.open_local_text(
                    TextTarget::Setting(0),
                    self.settings.preferences.greeting.clone(),
                    false,
                ),
                (Personalization, 1) => self.settings_open(Sounds, 0),
                (Personalization, 2) => self.settings_open(Wallpaper, 0),
                (Sounds, 0) => {
                    self.settings.preferences.silent = !self.settings.preferences.silent;
                    self.settings.dirty = true;
                }
                (Sounds, 1) => self.open_local_text(
                    TextTarget::Setting(1),
                    self.settings.preferences.volume.to_string(),
                    true,
                ),
                (Sounds, 2) => self.settings_open(Ringtone, 0),
                (Sounds, 3) => {
                    self.settings.preferences.keypad_sound =
                        !self.settings.preferences.keypad_sound;
                    self.settings.dirty = true;
                }
                (Wallpaper, 0..=1) => {
                    self.settings.preferences.wallpaper = None;
                    self.settings.preferences.wallpaper_enabled = self.index == 0;
                    self.settings.dirty = true;
                    self.settings_open(Personalization, 2);
                }
                (Wallpaper, 2) => self.open_media(media::Purpose::Wallpaper),
                (Ringtone, i) if i == crate::audio::native::file_index() => {
                    self.open_media(media::Purpose::Ringtone)
                }
                (Ringtone, i) => {
                    self.settings.preferences.ringtone = self.settings_labels()[i].clone();
                    self.audio_preview = Some(self.settings.preferences.ringtone.clone());
                    self.settings.dirty = true;
                }
                (Time, 0) => {
                    let (_, minutes) = self.local_clock();
                    self.open_local_text(
                        TextTarget::Setting(3),
                        format!("{:02}:{:02}", minutes / 60, minutes % 60),
                        false,
                    );
                }
                (Time, 1) => {
                    let (date, _) = self.local_clock();
                    self.open_local_text(TextTarget::Setting(4), date.iso(), false);
                }
                (Time, 2) => self.open_local_text(
                    TextTarget::Setting(2),
                    self.settings.preferences.timezone.to_string(),
                    false,
                ),
                (Time, 3) => {
                    self.settings.preferences.clock_12h = !self.settings.preferences.clock_12h;
                    self.settings.dirty = true;
                }
                (Root, 2) => self.settings_open(Personalization, 0),
                (Personalization, 3) => {
                    self.settings.preview = self.settings.accent;
                    self.settings_open(Theme, 0);
                }
                (Personalization, 4) => self.settings_open(Keyboard, 0),
                (Keyboard, 0) => {}
                (Keyboard, 1) => self.settings_open(Shortcuts, 0),
                (Shortcuts, index) => {
                    let left = index == 1;
                    let assigned = if left {
                        self.settings.preferences.left_shortcut
                    } else {
                        self.settings.preferences.right_shortcut
                    };
                    self.settings_open(Shortcut { left }, assigned);
                }
                (Shortcut { left }, index) => {
                    if left {
                        self.settings.preferences.left_shortcut = index;
                    } else {
                        self.settings.preferences.right_shortcut = index;
                    }
                    self.settings.dirty = true;
                    self.settings_open(Shortcuts, usize::from(left));
                }
                _ => {}
            },
            _ => {}
        }
    }
    pub fn setting_enabled(&self, index: usize) -> bool {
        if matches!(self.settings.page, SettingsPage::Shortcut { .. })
            && shortcut_screen(index).is_some_and(|s| !self.apps.allows(s))
        {
            return false;
        }
        !matches!(
            (self.settings.page, index),
            (SettingsPage::Root, 5)
                | (SettingsPage::Connections, 1)
                | (SettingsPage::Security, _)
                | (SettingsPage::Keyboard, 0)
        )
    }
    pub fn local_clock(&self) -> (phone_core::calendar::Date, u64) {
        let wall = (self.timestamp() / 1000) as i64;
        let seconds = (wall
            + self.settings.preferences.clock_adjustment
            + self.settings.preferences.timezone as i64 * 3600)
            .max(0) as u64;
        (
            phone_core::calendar::Date::from_unix(seconds, 0),
            seconds / 60 % 1440,
        )
    }
    pub(super) fn setting_text(&mut self, i: usize, value: String) {
        use phone_core::calendar::Date;
        let valid = match i {
            0 => {
                self.settings.preferences.greeting = value.chars().take(128).collect();
                true
            }
            1 => value
                .parse::<u8>()
                .ok()
                .filter(|n| *n <= 10)
                .map(|n| self.settings.preferences.volume = n)
                .is_some(),
            2 => value
                .parse::<i32>()
                .ok()
                .filter(|n| (-12..=14).contains(n))
                .map(|n| self.settings.preferences.timezone = n)
                .is_some(),
            3 => {
                let parts: Vec<_> = value.split(':').collect();
                if let [a, b] = parts.as_slice() {
                    if let (Ok(h), Ok(m)) = (a.parse::<u64>(), b.parse::<u64>()) {
                        if h < 24 && m < 60 {
                            let (_, current) = self.local_clock();
                            self.settings.preferences.clock_adjustment +=
                                (h * 60 + m) as i64 * 60 - current as i64 * 60;
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            4 => {
                if let Some(date) = Date::parse(&value).filter(|d| (1970..=2099).contains(&d.year))
                {
                    let current = self.local_clock().0;
                    fn days(d: Date) -> i64 {
                        let mut total = 0;
                        for y in 1970..d.year {
                            total += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                                366
                            } else {
                                365
                            };
                        }
                        for m in 1..d.month {
                            total += Date::new(d.year, m, 1).unwrap().month_days() as i64;
                        }
                        total + d.day as i64 - 1
                    }
                    self.settings.preferences.clock_adjustment +=
                        (days(date) - days(current)) * 86400;
                    self.organizer.date = date;
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        if valid {
            self.settings.dirty = true;
        } else {
            self.notice(match i {
                1 => "音量范围 0–10。",
                2 => "时区范围 -12–14。",
                3 => "时间格式 HH:MM。",
                _ => "日期格式 YYYY-MM-DD，范围 1970–2099。",
            });
        }
    }
    pub(super) fn shortcut(&mut self, left: bool) {
        let index = if left {
            self.settings.preferences.left_shortcut
        } else {
            self.settings.preferences.right_shortcut
        };
        if let Some(screen) = shortcut_screen(index) {
            self.launch(screen);
        }
    }
    pub fn visible_shortcut(&self, left: bool) -> usize {
        let index = if left {
            self.settings.preferences.left_shortcut
        } else {
            self.settings.preferences.right_shortcut
        };
        if shortcut_screen(index).is_none_or(|s| self.apps.allows(s)) {
            index
        } else {
            0
        }
    }
}
pub fn shortcut_screen(index: usize) -> Option<Screen> {
    Some(match index {
        0 => return None,
        1 => Screen::Contacts,
        2 => Screen::Tools(tools::ToolPage::Calls),
        3 => Screen::Messages,
        4 => Screen::Music,
        5 => Screen::Tools(tools::ToolPage::Video),
        6 => Screen::Tools(tools::ToolPage::Radio),
        7 => Screen::Organizer(organizer::OrganizerPage::Alarms),
        10 => Screen::Organizer(organizer::OrganizerPage::Calendar),
        8 => Screen::Tools(tools::ToolPage::Files),
        9 => Screen::Tools(tools::ToolPage::Gallery),
        11 => Screen::Tools(tools::ToolPage::Calculator),
        12 => Screen::Browser,
        13 => Screen::Tools(tools::ToolPage::Recorder),
        15 => Screen::Tools(tools::ToolPage::Other),
        16 => Screen::Tools(tools::ToolPage::Counter),
        14 => Screen::Settings,
        _ => return None,
    })
}
