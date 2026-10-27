use super::*;
use crate::shell::settings::Preferences;
use phone_core::sms::Mailbox;
use std::path::PathBuf;
#[test]
fn save_reload_and_corrupt_file_is_retained() {
    let dir = std::env::temp_dir().join(format!("ta1278-store-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut m = Mailbox::default();
    m.save(
        None,
        phone_core::sms::Message {
            to: "123".into(),
            body: "中文".into(),
            ..Default::default()
        },
    );
    save(&dir, &m).unwrap();
    assert_eq!(load(&dir).unwrap().drafts, m.drafts);
    std::fs::write(dir.join("messages.json"), b"{bad").unwrap();
    assert!(load(&dir).is_err());
    assert_eq!(std::fs::read(dir.join("messages.json")).unwrap(), b"{bad");
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn preferences_roundtrip_and_invalid_values_preserve_original() {
    let dir = std::env::temp_dir().join(format!("ta1278-preferences-test-{}", std::process::id()));
    let prefs = Preferences {
        accent: 4,
        left_shortcut: 3,
        right_shortcut: 12,
        ..Preferences::default()
    };
    save_preferences(&dir, &prefs).unwrap();
    let restored = load_preferences(&dir).unwrap();
    assert_eq!(
        (
            restored.accent,
            restored.left_shortcut,
            restored.right_shortcut
        ),
        (4, 3, 12)
    );
    let invalid = b"{\"accent\":99}";
    std::fs::write(dir.join("preferences.json"), invalid).unwrap();
    assert!(load_preferences(&dir).is_err());
    assert_eq!(
        std::fs::read(dir.join("preferences.json")).unwrap(),
        invalid
    );
    std::fs::remove_dir_all(dir).unwrap();
}

use crate::shell::{Shell, alarm, organizer, tools::Tools};
use phone_core::{
    calendar::Date,
    input::{Event, Key, Phase},
};
use std::sync::atomic::{AtomicU64, Ordering};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ta1278-persistence-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn press(p: &mut Shell, key: Key) {
    p.handle(
        Event {
            key,
            phase: Phase::Down,
        },
        0,
    );
}
fn restored(dir: &Path) -> Shell {
    let mut p = Shell::new(Mailbox::default());
    p.apply_records(load_organizer(dir).unwrap());
    p
}
fn checkpoint(dir: &Path, p: &Shell) {
    save_json(dir, "organizer.json", &p.records(), 8 * 1024 * 1024).unwrap();
}
fn one_alarm(repeat: &str) -> organizer::Alarm {
    organizer::Alarm {
        enabled: true,
        fields: ["23:58", repeat, "Restart test", "Clock1", "5分钟"]
            .map(str::to_owned)
            .to_vec(),
    }
}
#[test]
fn backup_failure_and_interrupted_temp_never_replace_current_data() {
    let dir = Temp::new();
    let p = &dir.0;
    save_json(p, "sample.json", &vec![1], 1024).unwrap();
    std::fs::write(p.join("sample.json.tmp"), b"interrupted legacy writer").unwrap();
    save_json(p, "sample.json", &vec![2], 1024).unwrap();
    assert_eq!(load_json::<Vec<u8>>(p, "sample.json", 1024).unwrap(), [2]);
    assert_eq!(
        load_json::<Vec<u8>>(p, "sample.json.bak", 1024).unwrap(),
        [1]
    );
    assert_eq!(
        std::fs::read(p.join("sample.json.tmp")).unwrap(),
        b"interrupted legacy writer"
    );
    let current = std::fs::read(p.join("sample.json")).unwrap();
    assert!(save_json(p, "sample.json", &vec![0; 2048], 1024).is_err());
    assert_eq!(std::fs::read(p.join("sample.json")).unwrap(), current);
    std::fs::remove_file(p.join("sample.json.bak")).unwrap();
    std::fs::create_dir(p.join("sample.json.bak")).unwrap();
    assert!(save_json(p, "sample.json", &vec![3], 1024).is_err());
    assert_eq!(std::fs::read(p.join("sample.json")).unwrap(), current);
    assert!(!std::fs::read_dir(p).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".sample.json")
    }));
    std::fs::remove_dir(p.join("sample.json.bak")).unwrap();
    save_json(p, "sample.json", &vec![3], 1024).unwrap();
    assert_eq!(load_json::<Vec<u8>>(p, "sample.json", 1024).unwrap(), [3]);
    std::fs::write(p.join("sample.json"), b"broken").unwrap();
    assert!(save_json(p, "sample.json", &vec![4], 1024).is_err());
    assert_eq!(std::fs::read(p.join("sample.json")).unwrap(), b"broken");
    assert_eq!(
        load_json::<Vec<u8>>(p, "sample.json.bak", 1024).unwrap(),
        [2]
    );
}
#[test]
fn exclusive_profile_lock_is_released_without_deleting_the_lock_file() {
    let dir = Temp::new();
    let first = DataLock::acquire(&dir.0).unwrap();
    assert!(DataLock::acquire(&dir.0).is_err());
    drop(first);
    assert!(DataLock::acquire(&dir.0).is_ok());
    assert!(dir.0.join(".instance.lock").exists());
}
#[test]
fn older_organizer_schema_migrates_and_future_or_invalid_data_is_preserved() {
    let dir = Temp::new();
    let path = dir.0.join("organizer.json");
    std::fs::write(&path, br#"{"contacts":[],"events":[],"alarms":[]}"#).unwrap();
    let data = load_organizer(&dir.0).unwrap();
    assert_eq!(data.schema_version, 1);
    assert!(data.reminders.alerts.is_empty());
    let invalid = br#"{"schema_version":2,"contacts":[],"events":[],"alarms":[]}"#;
    std::fs::write(&path, invalid).unwrap();
    assert!(load_organizer(&dir.0).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), invalid);
    let mut data = organizer::LocalData::default();
    data.reminders.snoozed.push((
        Date {
            year: 2026,
            month: 0,
            day: 1,
        },
        1440,
        alarm::Alert {
            source: alarm::ReminderSource::Legacy,
            tone: "无".into(),
            name: "invalid".into(),
            time: "12:00".into(),
            snooze: 5,
        },
    ));
    std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(load_organizer(&dir.0).is_err());
}
#[test]
fn fired_once_alarm_and_snooze_survive_restart_and_midnight() {
    let dir = Temp::new();
    let mut p = Shell::new(Mailbox::default());
    p.organizer.alarms.push(one_alarm("一次"));
    let day = Date::new(2026, 12, 31).unwrap();
    p.check_alarms(day, 1438);
    checkpoint(&dir.0, &p);
    let mut p = restored(&dir.0);
    assert!(!p.organizer.alarms[0].enabled);
    assert_eq!(p.organizer.reminders.alerts.len(), 1);
    p.check_alarms(day, 1438);
    press(&mut p, Key::SoftLeft);
    checkpoint(&dir.0, &p);
    let mut p = restored(&dir.0);
    assert!(p.organizer.reminders.alerts.is_empty());
    assert_eq!(p.organizer.reminders.snoozed.len(), 1);
    p.check_alarms(day.next(1), 2);
    assert!(p.organizer.reminders.alerts.is_empty());
    p.check_alarms(day.next(1), 4);
    assert_eq!(p.organizer.reminders.alerts.len(), 1);
    assert!(p.organizer.reminders.snoozed.is_empty());
    press(&mut p, Key::Center);
    checkpoint(&dir.0, &p);
    let mut p = restored(&dir.0);
    p.check_alarms(day.next(1), 4);
    assert!(p.organizer.reminders.alerts.is_empty());
}
#[test]
fn dismissed_recurring_alarm_and_calendar_event_do_not_repeat_on_same_minute_restart() {
    let dir = Temp::new();
    let mut p = Shell::new(Mailbox::default());
    let day = Date::new(2026, 10, 26).unwrap();
    p.organizer.alarms.push(one_alarm("每天"));
    p.organizer.events.push(organizer::CalendarEvent {
        date: day,
        fields: ["Calendar", "23:58", "准时", "", "无", "每天", ""]
            .map(str::to_owned)
            .to_vec(),
    });
    p.check_alarms(day, 1438);
    p.check_events(day, 1438);
    assert_eq!(p.organizer.reminders.alerts.len(), 2);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    checkpoint(&dir.0, &p);
    let mut p = restored(&dir.0);
    p.check_alarms(day, 1438);
    p.check_events(day, 1438);
    assert!(p.organizer.reminders.alerts.is_empty());
    p.check_alarms(day.next(1), 1438);
    p.check_events(day.next(1), 1438);
    assert_eq!(p.organizer.reminders.alerts.len(), 2);
}
#[test]
fn failed_alarm_checkpoint_keeps_old_record_and_live_alert_for_retry() {
    let dir = Temp::new();
    let mut p = Shell::new(Mailbox::default());
    p.organizer.alarms.push(one_alarm("一次"));
    checkpoint(&dir.0, &p);
    std::fs::create_dir(dir.0.join("organizer.json.bak")).unwrap();
    p.check_alarms(Date::new(2026, 10, 26).unwrap(), 1438);
    assert!(save_json(&dir.0, "organizer.json", &p.records(), 8 * 1024 * 1024).is_err());
    let before = restored(&dir.0);
    assert!(before.organizer.alarms[0].enabled);
    assert!(before.organizer.reminders.alerts.is_empty());
    assert!(!p.organizer.alarms[0].enabled && p.records_dirty);
    assert_eq!(p.organizer.reminders.alerts.len(), 1);
    std::fs::remove_dir(dir.0.join("organizer.json.bak")).unwrap();
    checkpoint(&dir.0, &p);
    assert_eq!(restored(&dir.0).organizer.reminders.alerts.len(), 1);
}
#[test]
fn running_and_paused_clocks_restore_elapsed_time_without_restarting_a_finished_timer() {
    let dir = Temp::new();
    let mut tools = Tools::default();
    tools.stopwatch.running = true;
    tools.stopwatch.tick(1250);
    tools.stopwatch.lap_or_reset();
    tools.timer.start(10);
    tools.timer_name = "Tea".into();
    tools.stations = vec![875, 1017];
    tools.frequency = 1017;
    utilities::Snapshot::capture(&tools, 10000)
        .save(&dir.0)
        .unwrap();
    let snapshot = utilities::Snapshot::load(&dir.0).unwrap();
    let mut tools = Tools::default();
    snapshot.apply(&mut tools, 15000);
    assert_eq!(tools.stopwatch.elapsed_ms, 6250);
    assert_eq!(tools.stopwatch.laps, [1250]);
    assert_eq!(tools.timer.remaining_ms, 5000);
    assert_eq!(tools.stations, [875, 1017]);
    tools.stopwatch.running = false;
    tools.timer.running = false;
    let paused = utilities::Snapshot::capture(&tools, 15000);
    paused.apply(&mut tools, 25000);
    assert_eq!(tools.stopwatch.elapsed_ms, 6250);
    assert_eq!(tools.timer.remaining_ms, 5000);
    let mut backwards = Tools::default();
    snapshot.apply(&mut backwards, 9000);
    assert_eq!(backwards.timer.remaining_ms, 10000);
    snapshot.apply(&mut tools, 30000);
    assert!(tools.timer.expired && !tools.timer.running);
    assert_eq!(tools.timer.remaining_ms, 0);
    utilities::Snapshot::capture(&tools, 30000)
        .save(&dir.0)
        .unwrap();
    let mut p = Shell::new(Mailbox::default());
    utilities::Snapshot::load(&dir.0)
        .unwrap()
        .apply(&mut p.tools, 40000);
    p.tick(0, 0.);
    assert!(p.tools.timer_notice && p.tools.timer.expired);
    press(&mut p, Key::SoftRight);
    assert!(!p.tools.timer.expired);
    utilities::Snapshot::capture(&p.tools, 40000)
        .save(&dir.0)
        .unwrap();
    let mut tools = Tools::default();
    utilities::Snapshot::load(&dir.0)
        .unwrap()
        .apply(&mut tools, 50000);
    assert!(!tools.timer.expired && !tools.timer.running);
}
#[test]
fn timer_expiry_does_not_destroy_an_open_editor_and_can_be_acknowledged_later() {
    let mut p = Shell::new(Mailbox::default());
    p.tick(0, 0.);
    p.tools.timer.start(1);
    p.overlay = Some(crate::shell::Overlay::Text {
        target: crate::shell::TextTarget::Tool(1),
        editor: Box::new(phone_core::editor::Editor::new("draft", 20)),
    });
    p.tick(1000, 0.);
    assert!(p.tools.timer.expired);
    assert!(!p.tools.timer_notice);
    assert!(matches!(
        p.overlay,
        Some(crate::shell::Overlay::Text { .. })
    ));
    press(&mut p, Key::End);
    p.tick(1100, 0.);
    assert!(p.tools.timer_notice);
    press(&mut p, Key::SoftRight);
    assert!(!p.tools.timer.expired);
}
#[test]
fn utility_future_schema_is_rejected_and_failed_checkpoint_remains_retryable() {
    let dir = Temp::new();
    let mut store = crate::persistence::Directory::new(&dir.0);
    let mut runtime = crate::Runtime::load_profile(
        Default::default(),
        crate::Time::default(),
        &mut store,
        Box::new(phone_core::services::Offline),
        crate::persistence::Resume::Paused,
    )
    .unwrap();
    runtime.shell.tools.timer_name = "before".into();
    runtime.shell.tools.dirty = true;
    runtime.flush(&mut store, false).unwrap();
    std::fs::create_dir(dir.0.join("utilities.json.bak")).unwrap();
    runtime.shell.tools.timer_name = "after".into();
    runtime.shell.tools.dirty = true;
    assert!(runtime.flush(&mut store, false).is_err());
    assert!(runtime.shell.tools.dirty);
    assert_eq!(
        utilities::Snapshot::load(&dir.0).unwrap().data.timer_name,
        "before"
    );
    std::fs::remove_dir(dir.0.join("utilities.json.bak")).unwrap();
    runtime.flush(&mut store, false).unwrap();
    assert!(!runtime.shell.tools.dirty);
    let mut future = utilities::Snapshot::capture(&runtime.shell.tools, 1000);
    future.schema_version = 9;
    let original = serde_json::to_vec(&future).unwrap();
    std::fs::write(dir.0.join("utilities.json"), &original).unwrap();
    assert!(utilities::Snapshot::load(&dir.0).is_err());
    assert_eq!(
        std::fs::read(dir.0.join("utilities.json")).unwrap(),
        original
    );
}

#[test]
fn retired_shortcut_loads_as_none_without_changing_other_settings() {
    let dir = std::env::temp_dir().join(format!("ta1278-retired-shortcut-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("preferences.json"),
        br#"{"left_shortcut":17,"right_shortcut":3,"volume":4}"#,
    )
    .unwrap();
    let p = load_preferences(&dir).unwrap();
    assert_eq!((p.left_shortcut, p.right_shortcut, p.volume), (0, 3, 4));
    std::fs::remove_dir_all(dir).unwrap();
}
