//! Local organizer data and transactional forms. No original contacts/events imported.
use super::*;
use phone_core::calendar::Date;
use serde::{Deserialize, Serialize};
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LocalData {
    pub contacts: Vec<Vec<String>>,
    pub events: Vec<CalendarEvent>,
    pub alarms: Vec<Alarm>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub date: Date,
    pub fields: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub enabled: bool,
    pub fields: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrganizerPage {
    Calendar,
    CalendarMenu,
    Events,
    EventForm,
    Alarms,
    AlarmForm,
    AlarmDays,
    AlarmSnooze,
    AlarmMenu,
    AlarmFormMenu,
    DeleteAllAlarms,
    Contacts,
    ContactsMenu,
    ContactFormMenu,
    ContactForm,
    ContactDetail,
    ContactMenu,
    DeleteContact,
    DeleteEvent,
    DeleteAlarm,
    DeletePast,
}
pub struct Organizer {
    pub page: OrganizerPage,
    pub contact: super::contacts::ContactUi,
    pub alarm: super::alarm::AlarmUi,
    pub date: Date,
    pub draft: Vec<String>,
    pub editing: Option<usize>,
    pub selected: usize,
    pub pick_recipient: bool,
    pub last_alarm_minute: Option<(Date, u64)>,
}
impl Default for Organizer {
    fn default() -> Self {
        Self {
            page: OrganizerPage::Calendar,
            contact: Default::default(),
            alarm: super::alarm::AlarmUi::default(),
            date: Date::default(),
            draft: vec![],
            editing: None,
            selected: 0,
            pick_recipient: false,
            last_alarm_minute: None,
        }
    }
}
pub const CONTACT_FIELDS: [&str; 9] = [
    "图片",
    "姓",
    "名",
    "移动电话",
    "住宅",
    "单位",
    "生日",
    "铃声",
    "组",
];
pub const EVENT_FIELDS: [&str; 7] = ["标题", "时间", "提醒", "位置", "铃声", "重复", "备注"];
impl Phone {
    pub fn organizer_title(&self) -> &'static str {
        use OrganizerPage::*;
        match self.organizer.page {
            Calendar => "日历",
            CalendarMenu | ContactMenu | ContactsMenu | ContactFormMenu => "选项",
            Events => "日程",
            EventForm => "添加事件",
            Alarms => "闹钟",
            AlarmForm => "闹钟",
            AlarmDays => "重复，每周",
            AlarmSnooze => "重响时间",
            AlarmMenu | AlarmFormMenu => "选项",
            DeleteAllAlarms => "确认",
            Contacts => "通讯录",
            ContactForm => "新联系人",
            ContactDetail => "详情",
            DeleteContact | DeleteEvent | DeleteAlarm | DeletePast => "确认",
        }
    }
    pub fn organizer_items(&self) -> Vec<String> {
        use OrganizerPage::*;
        match self.organizer.page {
            CalendarMenu => ["添加事件", "转到日期", "删除今天之前的项目"]
                .map(str::to_owned)
                .to_vec(),
            Contacts => self
                .local_data
                .contacts
                .iter()
                .map(|c| contact_name(c))
                .collect(),
            ContactMenu => super::contacts::DETAIL_MENU.map(str::to_owned).to_vec(),
            ContactsMenu => super::contacts::ROOT_MENU.map(str::to_owned).to_vec(),
            ContactFormMenu => vec!["保存".into()],
            Events => self
                .local_data
                .events
                .iter()
                .filter(|e| e.date == self.organizer.date)
                .map(|e| e.fields.first().cloned().unwrap_or_default())
                .collect(),
            EventForm | ContactForm => {
                let names: &[&str] = match self.organizer.page {
                    EventForm => &EVENT_FIELDS,
                    _ => &CONTACT_FIELDS,
                };
                names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| {
                        format!(
                            "{n}  {}",
                            self.organizer.draft.get(i).map_or("", String::as_str)
                        )
                    })
                    .collect()
            }
            ContactDetail => self
                .local_data
                .contacts
                .get(self.organizer.selected)
                .map(|fields| {
                    CONTACT_FIELDS
                        .iter()
                        .zip(fields)
                        .filter(|(_, v)| !v.is_empty())
                        .map(|(n, v)| format!("{n}  {v}"))
                        .collect()
                })
                .unwrap_or_default(),
            _ => vec![],
        }
    }
    pub fn organizer_form(&mut self, page: OrganizerPage, editing: Option<usize>) {
        use OrganizerPage::*;
        self.organizer.editing = editing;
        self.organizer.page = page;
        self.index = 0;
        self.organizer.draft = match page {
            ContactForm => editing
                .and_then(|i| self.local_data.contacts.get(i).cloned())
                .unwrap_or_else(|| vec![String::new(); CONTACT_FIELDS.len()]),
            AlarmForm => editing
                .and_then(|i| self.local_data.alarms.get(i).map(|a| a.fields.clone()))
                .unwrap_or_else(|| {
                    ["09:00", "不必重复", "闹钟", "Clock1", "10分钟"]
                        .map(str::to_owned)
                        .to_vec()
                }),
            EventForm => editing
                .and_then(|i| self.local_data.events.get(i).map(|e| e.fields.clone()))
                .unwrap_or_else(|| {
                    ["", "12:00", "关闭", "", "默认", "不重复", ""]
                        .map(str::to_owned)
                        .to_vec()
                }),
            _ => vec![],
        };
        if page == ContactForm {
            if editing.is_none() {
                self.organizer.draft[6] = "1900-01-01".into();
                self.organizer.draft[7] = "预设".into();
                self.organizer.draft[8] = "无".into();
            }
            self.contact_load_field(1);
        }
        if page == AlarmForm {
            self.organizer.alarm.name = Editor::new(&self.organizer.draft[2], 20);
            self.organizer.alarm.digit = 0;
        }
    }
    fn organizer_save(&mut self) {
        use OrganizerPage::*;
        let fields = self.organizer.draft.clone();
        let editing = self.organizer.editing;
        match self.organizer.page {
            EventForm => {
                if fields[0].trim().is_empty() || !valid_time(&fields[1]) {
                    self.notice("请输入标题和有效时间（HH:MM）。");
                    return;
                }
                let event = CalendarEvent {
                    date: self.organizer.date,
                    fields,
                };
                if let Some(i) = editing {
                    self.local_data.events[i] = event;
                } else {
                    self.local_data.events.push(event);
                }
                self.organizer.page = Calendar;
            }
            _ => return,
        }
        self.local_dirty = true;
        self.index = 0;
    }
    pub(super) fn organizer_key(&mut self, key: Key) {
        use OrganizerPage::*;
        let page = self.organizer.page;
        if self.contacts_key(key) {
            return;
        }
        if self.alarm_key(key) {
            return;
        }
        if page == Calendar {
            match key {
                Key::Left => self.organizer.date = self.organizer.date.next(-1),
                Key::Right => self.organizer.date = self.organizer.date.next(1),
                Key::Up => self.organizer.date = self.organizer.date.next(-7),
                Key::Down => self.organizer.date = self.organizer.date.next(7),
                Key::Star => self.organizer.date = self.organizer.date.month_shift(-1),
                Key::Hash => self.organizer.date = self.organizer.date.month_shift(1),
                Key::SoftLeft => {
                    self.organizer.page = CalendarMenu;
                    self.index = 0;
                }
                Key::Center => {
                    self.organizer.page = Events;
                    self.index = 0;
                }
                Key::SoftRight => self.leave_app(),
                _ => {}
            }
            return;
        }
        let n = self.organizer_items().len();
        match key {
            Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
            Key::Down if n > 0 => self.index = (self.index + 1) % n,
            Key::SoftRight => {
                self.organizer.page = match page {
                    CalendarMenu | Events | EventForm | DeletePast => Calendar,
                    DeleteAlarm => Alarms,
                    DeleteEvent => Events,
                    _ => {
                        if self.organizer.pick_recipient {
                            self.organizer.pick_recipient = false;
                            self.go(Screen::Compose);
                        } else {
                            self.leave_app();
                        }
                        return;
                    }
                };
                self.index = 0;
            }
            Key::SoftLeft if page == EventForm => self.organizer_save(),
            Key::SoftLeft if page == Events => self.organizer_form(EventForm, None),
            Key::Delete if n > 0 && matches!(page, Alarms | Events) => {
                self.organizer.selected = self.index;
                self.organizer.page = match page {
                    Alarms => DeleteAlarm,
                    _ => DeleteEvent,
                };
            }
            Key::Center if matches!(page, DeleteAlarm | DeleteEvent | DeletePast) => {
                let i = self.organizer.selected;
                self.organizer.page = match page {
                    DeleteAlarm => {
                        if i < self.local_data.alarms.len() {
                            self.local_data.alarms.remove(i);
                        }
                        Alarms
                    }
                    DeleteEvent => {
                        let ids = self.day_events();
                        if let Some(&id) = ids.get(i) {
                            self.local_data.events.remove(id);
                        }
                        Events
                    }
                    _ => {
                        self.local_data.events.retain(|e| e.date >= Date::default());
                        Calendar
                    }
                };
                self.local_dirty = true;
                self.index = 0;
            }
            Key::Center if n > 0 => match page {
                CalendarMenu => match self.index {
                    0 => self.organizer_form(EventForm, None),
                    1 => self.open_local_text(
                        TextTarget::Organizer(100),
                        self.organizer.date.iso(),
                        false,
                    ),
                    _ => self.organizer.page = DeletePast,
                },
                EventForm => self.open_local_text(
                    TextTarget::Organizer(self.index),
                    self.organizer.draft[self.index].clone(),
                    false,
                ),
                Events => {
                    let id = self.day_events()[self.index];
                    self.organizer_form(EventForm, Some(id));
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn day_events(&self) -> Vec<usize> {
        self.local_data
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.date == self.organizer.date)
            .map(|(i, _)| i)
            .collect()
    }
    pub(super) fn organizer_text(&mut self, field: usize, value: String) {
        if field == 100 {
            match Date::parse(&value) {
                Some(d) => {
                    self.organizer.date = d;
                    self.organizer.page = OrganizerPage::Calendar;
                }
                None => self.notice("日期格式：YYYY-MM-DD"),
            }
        } else if let Some(v) = self.organizer.draft.get_mut(field) {
            *v = value.clone();
            if self.organizer.page == OrganizerPage::ContactForm
                && self.organizer.contact.field == field
            {
                self.organizer.contact.input = Editor::new(&value, 40);
                self.organizer.contact.input.numeric = (3..=6).contains(&field);
            }
        }
    }
}
pub fn contact_name(c: &[String]) -> String {
    let name = c.iter().skip(1).take(2).cloned().collect::<String>();
    if name.is_empty() {
        c.iter()
            .skip(3)
            .take(3)
            .find(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| "无姓名".into())
    } else {
        name
    }
}
pub(super) fn valid_time(s: &str) -> bool {
    if s.len() != 5
        || !s.bytes().enumerate().all(|(i, b)| {
            if i == 2 {
                b == b':'
            } else {
                b.is_ascii_digit()
            }
        })
    {
        return false;
    }
    let Some((h, m)) = s.split_once(':') else {
        return false;
    };
    matches!((h.parse::<u8>(),m.parse::<u8>()),(Ok(h),Ok(m)) if h<24&&m<60)
}
