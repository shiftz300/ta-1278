//! Local organizer data and transactional forms. No original contacts/events imported.
use super::*;
use phone_core::calendar::Date;
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct LocalData {
    #[serde(default = "local_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub reminders: super::alarm::Reminders,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub playlists: Vec<super::music::Playlist>,
    #[serde(default)]
    pub calls: Vec<super::calls::Record>,
    pub contacts: Vec<Vec<String>>,
    pub events: Vec<CalendarEvent>,
    pub alarms: Vec<Alarm>,
}
fn local_schema() -> u32 {
    1
}
impl Default for LocalData {
    fn default() -> Self {
        Self {
            schema_version: 1,
            reminders: Default::default(),
            groups: vec![],
            playlists: vec![],
            calls: vec![],
            contacts: vec![],
            events: vec![],
            alarms: vec![],
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub date: Date,
    pub fields: Vec<String>,
}
impl CalendarEvent {
    pub fn occurs(&self, date: Date) -> bool {
        if date < self.date {
            return false;
        }
        match self.fields.get(5).map(String::as_str) {
            Some("每天") => true,
            Some("每周") => date.weekday() == self.date.weekday(),
            Some("每月") => date.day == self.date.day,
            Some("每年") => date.month == self.date.month && date.day == self.date.day,
            _ => date == self.date,
        }
    }
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
    EventFormMenu,
    EventReminder,
    EventRepeat,
    EventRingtone,
    Alarms,
    AlarmForm,
    AlarmDays,
    AlarmSnooze,
    AlarmRingtone,
    AlarmMenu,
    AlarmFormMenu,
    DeleteAllAlarms,
    DeleteEvent,
    DeleteAlarm,
    DeletePast,
}
pub struct Organizer {
    pub events: Vec<CalendarEvent>,
    pub alarms: Vec<Alarm>,
    pub reminders: super::alarm::Reminders,
    pub page: OrganizerPage,
    pub alarm: super::alarm::AlarmUi,
    pub event: super::calendar::EventUi,
    pub date: Date,
    pub draft: Vec<String>,
    pub editing: Option<usize>,
    pub selected: usize,
    pub last_alarm_minute: Option<(Date, u64)>,
    pub last_event_minute: Option<(Date, u64)>,
}
impl Default for Organizer {
    fn default() -> Self {
        Self {
            events: vec![],
            alarms: vec![],
            reminders: Default::default(),
            page: OrganizerPage::Calendar,
            alarm: super::alarm::AlarmUi::default(),
            event: Default::default(),
            date: Date::default(),
            draft: vec![],
            editing: None,
            selected: 0,
            last_alarm_minute: None,
            last_event_minute: None,
        }
    }
}
pub const EVENT_FIELDS: [&str; 7] = ["标题", "时间", "提醒", "位置", "铃声", "重复", "备注"];
impl Shell {
    pub fn organizer_title(&self) -> &'static str {
        use OrganizerPage::*;
        match self.organizer.page {
            Calendar => "日历",
            CalendarMenu => "选项",
            Events => "日程",
            EventForm => "新建事件",
            EventFormMenu => "选项",
            EventReminder => "提醒",
            EventRepeat => "重复",
            EventRingtone => "铃声",
            Alarms => "闹钟",
            AlarmForm => "闹钟",
            AlarmDays => "重复，每周",
            AlarmSnooze => "重响时间",
            AlarmRingtone => "闹钟铃声",
            AlarmMenu | AlarmFormMenu => "选项",
            DeleteAllAlarms => "确认",
            DeleteEvent | DeleteAlarm | DeletePast => "确认",
        }
    }
    pub fn organizer_items(&self) -> Vec<String> {
        use OrganizerPage::*;
        match self.organizer.page {
            CalendarMenu => ["添加新事件", "转至另一日期", "删除今天前的所有项"]
                .map(str::to_owned)
                .to_vec(),
            EventReminder => ["关闭", "准时", "5分钟前", "15分钟前", "30分钟前", "1小时前"]
                .map(str::to_owned)
                .to_vec(),
            EventRepeat => ["不重复", "每天", "每周", "每月", "每年"]
                .map(str::to_owned)
                .to_vec(),
            EventRingtone => crate::audio::native::tone_labels("默认"),
            Events => self
                .organizer
                .events
                .iter()
                .filter(|e| e.occurs(self.organizer.date))
                .map(|e| e.fields.first().cloned().unwrap_or_default())
                .collect(),
            EventForm => EVENT_FIELDS
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    format!(
                        "{n}  {}",
                        self.organizer.draft.get(i).map_or("", String::as_str)
                    )
                })
                .collect(),
            _ => vec![],
        }
    }
    pub fn organizer_form(&mut self, page: OrganizerPage, editing: Option<usize>) {
        use OrganizerPage::*;
        self.organizer.editing = editing;
        self.organizer.page = page;
        self.index = 0;
        self.organizer.draft = match page {
            AlarmForm => editing
                .and_then(|i| self.organizer.alarms.get(i).map(|a| a.fields.clone()))
                .unwrap_or_else(|| {
                    ["09:00", "不必重复", "闹钟", "Clock1", "10分钟"]
                        .map(str::to_owned)
                        .to_vec()
                }),
            EventForm => editing
                .and_then(|i| self.organizer.events.get(i).map(|e| e.fields.clone()))
                .unwrap_or_else(|| {
                    ["", "12:00", "关闭", "", "默认", "不重复", ""]
                        .map(str::to_owned)
                        .to_vec()
                }),
            _ => vec![],
        };
        if page == AlarmForm {
            self.organizer.alarm.name = Editor::new(&self.organizer.draft[2], 20);
            self.organizer.alarm.digit = 0;
        }
        if page == EventForm {
            self.organizer.event.date = editing
                .and_then(|i| self.organizer.events.get(i))
                .map_or(self.organizer.date, |event| event.date);
            self.event_load_field(0);
        }
    }
    pub(super) fn organizer_save(&mut self) {
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
                    date: self.organizer.event.date,
                    fields,
                };
                if let Some(i) = editing {
                    self.organizer.events[i] = event;
                } else {
                    self.organizer.events.push(event);
                }
                self.organizer.page = Calendar;
            }
            _ => return,
        }
        self.records_dirty = true;
        self.index = 0;
    }
    pub(super) fn organizer_key(&mut self, key: Key) {
        use OrganizerPage::*;
        let page = self.organizer.page;
        if self.alarm_key(key) {
            return;
        }
        if self.event_form_key(key) {
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
            Key::Up | Key::Down => {
                vertical(&mut self.index, n, key, Boundary::Wrap);
            }
            Key::SoftRight => {
                self.organizer.page = match page {
                    CalendarMenu | Events | EventForm | DeletePast => Calendar,
                    DeleteAlarm => Alarms,
                    DeleteEvent => Events,
                    EventReminder | EventRepeat | EventRingtone => EventForm,
                    _ => {
                        self.leave_app();
                        return;
                    }
                };
                self.index = if matches!(page, EventReminder | EventRepeat | EventRingtone) {
                    self.organizer.event.return_index
                } else {
                    0
                };
            }
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
                        if i < self.organizer.alarms.len() {
                            self.organizer.alarms.remove(i);
                        }
                        Alarms
                    }
                    DeleteEvent => {
                        let ids = self.day_events();
                        if let Some(&id) = ids.get(i) {
                            self.organizer.events.remove(id);
                        }
                        Events
                    }
                    _ => {
                        let today = self.local_clock().0;
                        self.organizer.events.retain(|e| {
                            e.date >= today || e.fields.get(5).is_some_and(|s| s != "不重复")
                        });
                        Calendar
                    }
                };
                self.records_dirty = true;
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
                EventReminder | EventRepeat => {
                    let field = if page == EventReminder { 2 } else { 5 };
                    self.organizer.draft[field] = self.organizer_items()[self.index].clone();
                    self.organizer.page = EventForm;
                    self.index = field;
                }
                EventRingtone => {
                    if self.index == crate::audio::native::file_index() {
                        self.organizer.page = EventForm;
                        self.index = 4;
                        self.open_media(media::Purpose::EventRingtone);
                    } else {
                        self.organizer.draft[4] = self.organizer_items()[self.index].clone();
                        self.audio_preview = Some(self.organizer.draft[4].clone());
                        self.organizer.page = EventForm;
                        self.index = 4;
                    }
                }
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
        self.organizer
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.occurs(self.organizer.date))
            .map(|(i, _)| i)
            .collect()
    }
    pub fn check_events(&mut self, date: Date, minute: u64) {
        if !self.apps.calendar {
            return;
        }
        if minute >= 1440 || self.organizer.last_event_minute == Some((date, minute)) {
            return;
        }
        self.organizer.last_event_minute = Some((date, minute));
        if self.organizer.reminders.last_event_fired == Some((date, minute)) {
            return;
        }
        for event in &self.organizer.events {
            let lead = match event.fields[2].as_str() {
                "准时" => 0,
                "5分钟前" => 5,
                "15分钟前" => 15,
                "30分钟前" => 30,
                "1小时前" => 60,
                _ => continue,
            };
            let due = minute + lead;
            let day = if due >= 1440 { date.next(1) } else { date };
            let time = &event.fields[1];
            if self.organizer.reminders.alerts.len() + self.organizer.reminders.snoozed.len() < 1024
                && event.occurs(day)
                && valid_time(time)
                && time == &format!("{:02}:{:02}", due % 1440 / 60, due % 60)
            {
                self.records_dirty = true;
                self.organizer.reminders.last_event_fired = Some((date, minute));
                self.organizer
                    .reminders
                    .alerts
                    .push_back(super::alarm::Alert {
                        source: super::alarm::ReminderSource::Calendar,
                        tone: event.fields[4].clone(),
                        name: event.fields[0].clone(),
                        time: time.clone(),
                        snooze: 10,
                    });
            }
        }
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
        }
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
