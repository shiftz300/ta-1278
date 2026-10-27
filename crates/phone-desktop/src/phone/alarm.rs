//! Alarm controls recovered from 17.00.17.02; see docs/alarm-recovery.md.
//! Scheduling and alert presentation are local replica policy, not an OS service.
use super::*;
use organizer::{Alarm, OrganizerPage};
use phone_core::calendar::Date;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, sync::OnceLock};

#[derive(Deserialize)]
pub struct Tables {
    pub weekday_labels: [String; 7],
    pub weekday_bits: [u8; 7],
    pub snooze_labels: [String; 5],
    pub snooze_minutes: [u64; 5],
}
pub fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../assets/ui/native-alarm.json"))
            .expect("verified alarm tables")
    })
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderSource {
    #[default]
    Legacy,
    Alarm,
    Calendar,
}
impl ReminderSource {
    pub fn enabled(self, apps: crate::config::Apps) -> bool {
        match self {
            Self::Legacy => apps.alarms && apps.calendar,
            Self::Alarm => apps.alarms,
            Self::Calendar => apps.calendar,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    #[serde(default)]
    pub source: ReminderSource,
    pub tone: String,
    pub name: String,
    pub time: String,
    pub snooze: u64,
}
#[derive(Default)]
pub struct AlarmUi {
    pub name: Editor,
    pub digit: usize,
    pub days: u8,
    pub return_field: usize,
}
/// Durable reminder state; stored with alarm definitions in organizer.json.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Reminders {
    pub alerts: VecDeque<Alert>,
    pub snoozed: Vec<(Date, u64, Alert)>,
    pub last_alarm_fired: Option<(Date, u64)>,
    pub last_event_fired: Option<(Date, u64)>,
}
impl Reminders {
    pub fn valid(&self) -> bool {
        let moment =
            |(d, m): &(Date, u64)| Date::new(d.year, d.month, d.day).is_some() && *m < 1440;
        let alert = |a: &Alert| {
            a.name.chars().count() <= 256
                && a.tone.len() <= 4096
                && super::organizer::valid_time(&a.time)
                && (1..=60).contains(&a.snooze)
        };
        self.alerts.len() + self.snoozed.len() <= 1024
            && self.alerts.iter().all(alert)
            && self
                .snoozed
                .iter()
                .all(|(d, m, a)| moment(&(*d, *m)) && alert(a))
            && self.last_alarm_fired.as_ref().is_none_or(moment)
            && self.last_event_fired.as_ref().is_none_or(moment)
    }
}
pub fn repeat_mask(s: &str) -> u8 {
    match s {
        "每天" => 127,
        "工作日" => 62,
        "周末" => 65,
        _ => tables()
            .weekday_labels
            .iter()
            .zip(tables().weekday_bits)
            .filter(|(label, _)| s.split('、').any(|day| day == label.as_str()))
            .fold(0, |mask, (_, bit)| mask | bit),
    }
}
pub fn repeat_label(mask: u8) -> String {
    match mask {
        0 => "不必重复".into(),
        127 => "每天".into(),
        62 => "工作日".into(),
        65 => "周末".into(),
        _ => tables()
            .weekday_labels
            .iter()
            .zip(tables().weekday_bits)
            .filter(|(_, bit)| mask & bit != 0)
            .map(|(s, _)| s.as_str())
            .collect::<Vec<_>>()
            .join("、"),
    }
}
pub fn snooze_index(s: &str) -> usize {
    let compact = s.replace(' ', "");
    tables()
        .snooze_labels
        .iter()
        .position(|v| v == &compact)
        .unwrap_or(2)
}
pub fn is_alarm(page: OrganizerPage) -> bool {
    use OrganizerPage::*;
    matches!(
        page,
        Alarms
            | AlarmForm
            | AlarmDays
            | AlarmSnooze
            | AlarmRingtone
            | AlarmMenu
            | AlarmFormMenu
            | DeleteAllAlarms
    )
}
impl Phone {
    pub fn alarm_name_active(&self) -> bool {
        matches!(self.screen, Screen::Organizer(_))
            && self.organizer.page == OrganizerPage::AlarmForm
            && self.index == 2
    }
    pub fn active_reminder(&self) -> Option<&Alert> {
        self.organizer
            .reminders
            .alerts
            .iter()
            .find(|a| a.source.enabled(self.apps))
    }
    pub fn check_alarms(&mut self, date: Date, minute: u64) {
        if minute >= 1440 || self.organizer.last_alarm_minute == Some((date, minute)) {
            return;
        }
        self.organizer.last_alarm_minute = Some((date, minute));
        let time = format!("{:02}:{:02}", minute / 60, minute % 60);
        let ui = &mut self.organizer.reminders;
        let mut pending = Vec::new();
        for (d, m, mut alert) in ui.snoozed.drain(..) {
            if (d, m) <= (date, minute) && alert.source.enabled(self.apps) {
                alert.time = time.clone();
                ui.alerts.push_back(alert);
                self.records_dirty = true;
            } else {
                pending.push((d, m, alert));
            }
        }
        ui.snoozed = pending;
        if !self.apps.alarms || ui.last_alarm_fired == Some((date, minute)) {
            return;
        }
        for a in &mut self.organizer.alarms {
            if !a.enabled || a.fields.first() != Some(&time) {
                continue;
            }
            let repeat = a.fields.get(1).map_or("一次", String::as_str);
            let mask = repeat_mask(repeat);
            let once = matches!(repeat, "一次" | "不必重复" | "");
            if (once || mask & tables().weekday_bits[date.weekday()] != 0)
                && ui.alerts.len() + ui.snoozed.len() < 1024
            {
                ui.last_alarm_fired = Some((date, minute));
                self.records_dirty = true;
                ui.alerts.push_back(Alert {
                    source: ReminderSource::Alarm,
                    tone: a.fields.get(3).cloned().unwrap_or_else(|| "默认".into()),
                    name: a
                        .fields
                        .get(2)
                        .filter(|s| !s.is_empty())
                        .cloned()
                        .unwrap_or_else(|| "闹钟".into()),
                    time: time.clone(),
                    snooze: tables().snooze_minutes
                        [snooze_index(a.fields.get(4).map_or("10分钟", String::as_str))],
                });
                if once {
                    a.enabled = false;
                    self.records_dirty = true;
                }
            }
        }
    }
    pub(super) fn alarm_alert_key(&mut self, key: Key) -> bool {
        let Some(index) = self
            .organizer
            .reminders
            .alerts
            .iter()
            .position(|a| a.source.enabled(self.apps))
        else {
            return false;
        };
        match key {
            Key::SoftLeft => {
                let (date, minute) = self
                    .organizer
                    .last_alarm_minute
                    .unwrap_or_else(|| self.local_clock());
                let alert = self.organizer.reminders.alerts.remove(index).unwrap();
                self.records_dirty = true;
                let total = minute + alert.snooze;
                self.organizer.reminders.snoozed.push((
                    date.next((total / 1440) as i32),
                    total % 1440,
                    alert,
                ));
            }
            Key::Center | Key::SoftRight | Key::End => {
                self.organizer.reminders.alerts.remove(index);
                self.records_dirty = true;
            }
            _ => {}
        }
        true
    }
    fn alarm_root(&mut self, row: usize) {
        self.organizer.page = OrganizerPage::Alarms;
        self.index = row.min(self.organizer.alarms.len());
    }
    fn alarm_return(&mut self, field: usize) {
        self.organizer.page = OrganizerPage::AlarmForm;
        self.index = field;
    }
    fn alarm_sync_name(&mut self) -> bool {
        if !self.organizer.alarm.name.accept_prediction() {
            return false;
        }
        self.organizer.alarm.name.commit();
        self.organizer.draft[2] = self.organizer.alarm.name.text();
        true
    }
    fn alarm_save(&mut self) {
        if !self.alarm_sync_name() {
            return;
        }
        let fields = self.organizer.draft.clone();
        if !organizer::valid_time(&fields[0]) {
            self.notice("请输入有效时间（HH:MM）。");
            return;
        }
        let row = if let Some(i) = self.organizer.editing {
            self.organizer.alarms[i].fields = fields;
            i + 1
        } else {
            // The native string states a five-alarm limit; not a recovered capacity branch.
            if self.organizer.alarms.len() >= 5 {
                self.notice("最多只能设置5个闹钟");
                return;
            }
            self.organizer.alarms.push(Alarm {
                enabled: true,
                fields,
            });
            self.organizer.alarms.len()
        };
        self.records_dirty = true;
        self.alarm_root(row);
    }
    pub(super) fn alarm_key(&mut self, key: Key) -> bool {
        use OrganizerPage::*;
        let page = self.organizer.page;
        if !is_alarm(page) {
            return false;
        }
        match page {
            Alarms => {
                let count = self.organizer.alarms.len() + 1;
                match key {
                    Key::Up => self.index = (self.index + count - 1) % count,
                    Key::Down => self.index = (self.index + 1) % count,
                    Key::Center if self.index == 0 => {
                        if count > 5 {
                            self.notice("最多只能设置5个闹钟");
                        } else {
                            self.organizer_form(AlarmForm, None);
                        }
                    }
                    Key::Center => {
                        let a = &mut self.organizer.alarms[self.index - 1];
                        a.enabled = !a.enabled;
                        self.records_dirty = true;
                    }
                    Key::SoftLeft if self.index > 0 => {
                        self.organizer.selected = self.index - 1;
                        self.organizer.page = AlarmMenu;
                        self.index = 0;
                    }
                    Key::SoftRight => self.leave_app(),
                    _ => {}
                }
            }
            AlarmMenu => match key {
                Key::Up => self.index = (self.index + 2) % 3,
                Key::Down => self.index = (self.index + 1) % 3,
                Key::SoftRight => self.alarm_root(self.organizer.selected + 1),
                Key::Center => match self.index {
                    0 => self.organizer_form(AlarmForm, Some(self.organizer.selected)),
                    1 => {
                        self.organizer.alarms.remove(self.organizer.selected);
                        self.records_dirty = true;
                        self.alarm_root(self.organizer.selected + 1);
                    }
                    _ => self.organizer.page = DeleteAllAlarms,
                },
                _ => {}
            },
            DeleteAllAlarms => match key {
                Key::Center => {
                    self.organizer.alarms.clear();
                    self.records_dirty = true;
                    self.alarm_root(0);
                }
                Key::SoftRight => {
                    self.organizer.page = AlarmMenu;
                    self.index = 2;
                }
                _ => {}
            },
            AlarmForm => match key {
                Key::Up | Key::Down => {
                    if self.index == 2 && !self.alarm_sync_name() {
                        return true;
                    }
                    self.index = if key == Key::Up {
                        self.index.saturating_sub(1)
                    } else {
                        (self.index + 1).min(4)
                    };
                }
                Key::Left if self.index == 0 => {
                    self.organizer.alarm.digit = self.organizer.alarm.digit.saturating_sub(1)
                }
                Key::Right if self.index == 0 => {
                    self.organizer.alarm.digit = (self.organizer.alarm.digit + 1).min(3)
                }
                Key::Digit(d) if self.index == 0 && d < 10 => {
                    let positions = [0, 1, 3, 4];
                    let mut time = self.organizer.draft[0].as_bytes().to_vec();
                    if time.len() == 5 {
                        time[positions[self.organizer.alarm.digit]] = b'0' + d;
                        let value = String::from_utf8(time).unwrap_or_default();
                        // Permit an intermediate 29:00 while replacing 09:00 with 23:00.
                        // Save validates the completed value; this is host editing policy.
                        self.organizer.draft[0] = value;
                        self.organizer.alarm.digit = (self.organizer.alarm.digit + 1) % 4;
                    }
                }
                Key::SoftRight
                    if self.index == 2 && !self.organizer.alarm.name.units.is_empty() =>
                {
                    self.organizer.alarm.name.backspace()
                }
                Key::SoftRight => self.alarm_root(self.organizer.editing.map_or(0, |i| i + 1)),
                Key::SoftLeft => {
                    self.organizer.alarm.return_field = self.index;
                    self.organizer.page = AlarmFormMenu;
                    self.index = 0;
                }
                Key::Center => match self.index {
                    0 | 2 => self.alarm_save(),
                    1 => {
                        self.organizer.alarm.days = repeat_mask(&self.organizer.draft[1]);
                        self.organizer.page = AlarmDays;
                        self.index = 0;
                    }
                    4 => {
                        self.index = snooze_index(&self.organizer.draft[4]);
                        self.organizer.page = AlarmSnooze;
                    }
                    _ => {
                        self.organizer.page = AlarmRingtone;
                        self.index = crate::audio::native::tone_labels("预设")
                            .iter()
                            .position(|s| s == &self.organizer.draft[3])
                            .unwrap_or(0);
                    }
                },
                _ => {}
            },
            AlarmFormMenu => match key {
                Key::Center => self.alarm_save(),
                Key::SoftRight => self.alarm_return(self.organizer.alarm.return_field),
                _ => {}
            },
            AlarmDays => match key {
                Key::Up => self.index = (self.index + 6) % 7,
                Key::Down => self.index = (self.index + 1) % 7,
                Key::Center => self.organizer.alarm.days ^= tables().weekday_bits[self.index],
                Key::SoftRight => {
                    self.organizer.draft[1] = repeat_label(self.organizer.alarm.days);
                    self.alarm_return(1);
                }
                _ => {}
            },
            AlarmSnooze => match key {
                Key::Up => self.index = (self.index + 4) % 5,
                Key::Down => self.index = (self.index + 1) % 5,
                Key::Center => {
                    self.organizer.draft[4] = tables().snooze_labels[self.index].clone();
                    self.alarm_return(4);
                }
                Key::SoftRight => self.alarm_return(4),
                _ => {}
            },
            AlarmRingtone => {
                let labels = crate::audio::native::tone_labels("预设");
                match key {
                    Key::Up => self.index = (self.index + labels.len() - 1) % labels.len(),
                    Key::Down => self.index = (self.index + 1) % labels.len(),
                    Key::SoftRight => self.alarm_return(3),
                    Key::Center if self.index == crate::audio::native::file_index() => {
                        self.alarm_return(3);
                        self.open_media(media::Purpose::AlarmRingtone);
                    }
                    Key::Center => {
                        self.organizer.draft[3] = labels[self.index].clone();
                        self.audio_preview = Some(labels[self.index].clone());
                        self.alarm_return(3);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        true
    }
}
