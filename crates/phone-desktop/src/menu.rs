//! Runtime menu order captured from this TA-1278; compiled registry alone was insufficient.
use crate::phone::{Screen, organizer::OrganizerPage, tools::ToolPage};
use phone_core::input::Key;
pub struct Entry {
    pub label: &'static str,
    pub screen: Screen,
    pub icon: usize,
}
pub const ENTRIES: [Entry; 16] = [
    Entry {
        label: "日历",
        screen: Screen::Organizer(OrganizerPage::Calendar),
        icon: 6,
    },
    Entry {
        label: "闹钟",
        screen: Screen::Organizer(OrganizerPage::Alarms),
        icon: 9,
    },
    Entry {
        label: "贪吃蛇",
        screen: Screen::Snake,
        icon: 10,
    },
    Entry {
        label: "互联网",
        screen: Screen::Browser,
        icon: 1,
    },
    Entry {
        label: "音乐",
        screen: Screen::Music,
        icon: 2,
    },
    Entry {
        label: "收音机",
        screen: Screen::Tools(ToolPage::Radio),
        icon: 11,
    },
    Entry {
        label: "信息",
        screen: Screen::Messages,
        icon: 0,
    },
    Entry {
        label: "通话历史记录",
        screen: Screen::Tools(ToolPage::Calls),
        icon: 3,
    },
    Entry {
        label: "文件",
        screen: Screen::Tools(ToolPage::Files),
        icon: 12,
    },
    Entry {
        label: "手电筒",
        screen: Screen::Tools(ToolPage::Flashlight),
        icon: 13,
    },
    Entry {
        label: "设置",
        screen: Screen::Settings,
        icon: 5,
    },
    Entry {
        label: "通讯录",
        screen: Screen::Contacts,
        icon: 4,
    },
    Entry {
        label: "录音",
        screen: Screen::Tools(ToolPage::Recorder),
        icon: 14,
    },
    Entry {
        label: "计算器",
        screen: Screen::Tools(ToolPage::Calculator),
        icon: 15,
    },
    Entry {
        label: "其他",
        screen: Screen::Tools(ToolPage::Other),
        icon: 16,
    },
    Entry {
        label: "计数器",
        screen: Screen::Tools(ToolPage::Counter),
        icon: 17,
    },
];
pub const INITIAL_FOCUS: usize = 4;
pub fn phone_disabled(screen: Screen) -> bool {
    matches!(screen, Screen::Tools(ToolPage::Calls))
}
pub fn visible(apps: crate::config::Apps) -> Vec<&'static Entry> {
    ENTRIES.iter().filter(|e| apps.allows(e.screen)).collect()
}
pub fn navigate(index: usize, key: Key, count: usize) -> usize {
    let count = count.max(1);
    let index = index.min(count - 1);
    match key {
        Key::Left => (index + count - 1) % count,
        Key::Right => (index + 1) % count,
        Key::Up => index.saturating_sub(3),
        Key::Down => (index + 3).min(count - 1),
        _ => index,
    }
}
