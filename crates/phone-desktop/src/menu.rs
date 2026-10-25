//! Runtime menu order captured from this TA-1278; compiled registry alone was insufficient.
use crate::phone::{Screen, organizer::OrganizerPage, tools::ToolPage};
use phone_core::input::Key;
pub struct Entry {
    pub label: &'static str,
    pub screen: Option<Screen>,
    pub icon: usize,
}
pub const ENTRIES: [Entry; 18] = [
    Entry {
        label: "日历",
        screen: Some(Screen::Organizer(OrganizerPage::Calendar)),
        icon: 6,
    },
    Entry {
        label: "闹钟",
        screen: Some(Screen::Organizer(OrganizerPage::Alarms)),
        icon: 9,
    },
    Entry {
        label: "贪吃蛇",
        screen: Some(Screen::Snake),
        icon: 10,
    },
    Entry {
        label: "互联网",
        screen: Some(Screen::Browser),
        icon: 1,
    },
    Entry {
        label: "音乐",
        screen: Some(Screen::Music),
        icon: 2,
    },
    Entry {
        label: "收音机",
        screen: Some(Screen::Tools(ToolPage::Radio)),
        icon: 11,
    },
    Entry {
        label: "信息",
        screen: Some(Screen::Messages),
        icon: 0,
    },
    Entry {
        label: "通话历史记录",
        screen: Some(Screen::Tools(ToolPage::Calls)),
        icon: 3,
    },
    Entry {
        label: "文件",
        screen: Some(Screen::Tools(ToolPage::Files)),
        icon: 12,
    },
    Entry {
        label: "手电筒",
        screen: Some(Screen::Tools(ToolPage::Flashlight)),
        icon: 13,
    },
    Entry {
        label: "设置",
        screen: Some(Screen::Settings),
        icon: 5,
    },
    Entry {
        label: "通讯录",
        screen: Some(Screen::Contacts),
        icon: 4,
    },
    Entry {
        label: "录音",
        screen: Some(Screen::Tools(ToolPage::Recorder)),
        icon: 14,
    },
    Entry {
        label: "计算器",
        screen: Some(Screen::Tools(ToolPage::Calculator)),
        icon: 15,
    },
    Entry {
        label: "其他",
        screen: Some(Screen::Tools(ToolPage::Other)),
        icon: 16,
    },
    Entry {
        label: "计数器",
        screen: Some(Screen::Tools(ToolPage::Counter)),
        icon: 17,
    },
    Entry {
        label: "支付宝",
        screen: None,
        icon: 18,
    },
    Entry {
        label: "喜马拉雅",
        screen: None,
        icon: 19,
    },
];
pub const INITIAL_FOCUS: usize = 4;
pub fn navigate(index: usize, key: Key) -> usize {
    match key {
        Key::Left => (index + ENTRIES.len() - 1) % ENTRIES.len(),
        Key::Right => (index + 1) % ENTRIES.len(),
        Key::Up => index.saturating_sub(3),
        Key::Down => (index + 3).min(ENTRIES.len() - 1),
        _ => index,
    }
}
