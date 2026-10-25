//! Settings routes observed on firmware 17.00.17.02. Preferences belong to the replica.
use super::*;
use serde::{Deserialize, Serialize};

pub const SHORTCUTS: [&str; 18] = [
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
    "支付宝",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub accent: usize,
    pub left_shortcut: usize,
    pub right_shortcut: usize,
}
impl Default for Preferences {
    fn default() -> Self {
        // Initial replica profile follows the owner's observed shortcuts, not factory defaults.
        Self {
            accent: crate::theme::Theme::load().startup_fallback_accent,
            left_shortcut: 12,
            right_shortcut: 3,
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> Result<(), String> {
        if self.accent >= 6
            || self.left_shortcut >= SHORTCUTS.len()
            || self.right_shortcut >= SHORTCUTS.len()
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
}
impl Phone {
    pub fn settings_title(&self) -> &'static str {
        match self.settings_page {
            SettingsPage::Root => "设置",
            SettingsPage::Personalization => "个性化选项",
            SettingsPage::Keyboard => "键盘",
            SettingsPage::Shortcuts => "锁定屏幕快捷操作",
            SettingsPage::Shortcut { left: true } => "左键",
            SettingsPage::Shortcut { left: false } => "右键",
            SettingsPage::Theme => "主题",
        }
    }
    pub fn settings_labels(&self) -> Vec<String> {
        match self.settings_page {
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
        }
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
    pub(super) fn settings_open(&mut self, page: SettingsPage, index: usize) {
        self.settings_page = page;
        self.index = index;
    }
    pub(super) fn settings_key(&mut self, key: Key) {
        use SettingsPage::*;
        if self.settings_page == Theme {
            match key {
                Key::Left => self.theme_preview = self.theme_preview.saturating_sub(1),
                Key::Right => self.theme_preview = (self.theme_preview + 1).min(5),
                Key::Center | Key::SoftLeft => {
                    self.accent = self.theme_preview;
                    self.preferences.accent = self.accent;
                    self.preferences_dirty = true;
                    self.settings_open(Personalization, 3);
                }
                Key::SoftRight => self.settings_open(Personalization, 3),
                _ => {}
            }
            return;
        }
        let n = self.settings_labels().len();
        match key {
            Key::Up => self.index = (self.index + n - 1) % n,
            Key::Down => self.index = (self.index + 1) % n,
            Key::SoftRight => match self.settings_page {
                Root => self.leave_app(),
                Personalization => self.settings_open(Root, 2),
                Keyboard => self.settings_open(Personalization, 4),
                Shortcuts => self.settings_open(Keyboard, 1),
                Shortcut { left } => self.settings_open(Shortcuts, usize::from(left)),
                Theme => unreachable!(),
            },
            Key::Center | Key::SoftLeft => match (self.settings_page, self.index) {
                (Root, 2) => self.settings_open(Personalization, 0),
                (Personalization, 3) => {
                    self.theme_preview = self.accent;
                    self.settings_open(Theme, 0);
                }
                (Personalization, 4) => self.settings_open(Keyboard, 0),
                (Keyboard, 0) => {}
                (Keyboard, 1) => self.settings_open(Shortcuts, 0),
                (Shortcuts, index) => {
                    let left = index == 1;
                    let assigned = if left {
                        self.preferences.left_shortcut
                    } else {
                        self.preferences.right_shortcut
                    };
                    self.settings_open(Shortcut { left }, assigned);
                }
                (Shortcut { left }, index) => {
                    if index == 2 {
                        return;
                    }
                    if left {
                        self.preferences.left_shortcut = index;
                    } else {
                        self.preferences.right_shortcut = index;
                    }
                    self.preferences_dirty = true;
                    self.settings_open(Shortcuts, usize::from(left));
                }
                _ => self.notice("此设置尚未实现。"),
            },
            _ => {}
        }
    }
    pub(super) fn shortcut(&mut self, left: bool) {
        let index = if left {
            self.preferences.left_shortcut
        } else {
            self.preferences.right_shortcut
        };
        let screen = match index {
            0 => return,
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
            _ => {
                self.notice(format!("{}尚未实现。", SHORTCUTS[index]));
                return;
            }
        };
        self.launch(screen);
    }
}
