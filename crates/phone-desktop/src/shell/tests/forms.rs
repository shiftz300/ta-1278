use super::press;
use super::*;
use organizer::OrganizerPage as Page;

#[test]
fn calendar_inline_fields_commit_together_and_editing_a_recurrence_keeps_start_date() {
    let mut p = Shell::new(Mailbox::default());
    p.go(Screen::Organizer(Page::Calendar));
    p.organizer_form(Page::EventForm, None);
    p.text("Appointment");
    assert!(p.overlay.is_none());
    press(&mut p, Key::Down);
    assert_eq!(p.organizer.draft[0], "Appointment");
    let date = p.organizer.event.date;
    p.organizer.event.digit = 8;
    press(&mut p, Key::Digit(9)); // Not a valid hours tens digit.
    assert_eq!(p.organizer.draft[1], "12:00");
    press(&mut p, Key::Down); // Reminder choice.
    press(&mut p, Key::Down); // Location editor.
    p.text("Desk");
    assert_eq!(p.editor().unwrap().limit, 40);
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.organizer.page, Page::EventFormMenu);
    press(&mut p, Key::SoftRight); // Menu cancel preserves the form.
    assert_eq!(p.organizer.draft[3], "Desk");
    press(&mut p, Key::Center); // Save directly from an inline field.
    assert_eq!(p.organizer.events.len(), 1);
    assert_eq!(p.organizer.events[0].date, date);
    p.organizer.events[0].fields[5] = "每周".into();
    p.organizer.date = date.next(7);
    p.organizer_form(Page::EventForm, Some(0));
    press(&mut p, Key::Center);
    assert_eq!(p.organizer.events[0].date, date);
    p.organizer_form(Page::EventForm, Some(0));
    p.text("not saved");
    press(&mut p, Key::End);
    assert_eq!(p.organizer.events[0].fields[0], "Appointment");
}

#[test]
fn file_checkbox_selection_requires_confirmation_and_pending_work_survives_navigation() {
    use files::{FileRequest, FileView};
    use tools::ToolPage;
    let mut p = Shell::new(Mailbox::default());
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.pending = None;
    p.files.view = FileView::Browse;
    p.files.writable = true;
    p.files.entries = ["a.txt", "b.txt"]
        .map(|s| crate::storage::files::Entry {
            name: s.into(),
            path: format!("/{s}"),
            directory: false,
            len: 1,
        })
        .into();
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.files.view, FileView::DeleteSelect);
    press(&mut p, Key::Center);
    assert_eq!(p.files.marked.len(), 1);
    assert!(p.files.pending.is_none());
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert_eq!(p.files.view, FileView::DeleteConfirm);
    press(&mut p, Key::SoftRight);
    assert!(p.files.pending.is_none());
    assert_eq!(p.files.marked.len(), 1);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Down);
    press(&mut p, Key::Center); // Select all.
    assert_eq!(p.files.marked.len(), 2);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    press(&mut p, Key::Center);
    assert!(matches!(&p.files.pending,Some(FileRequest::RemoveMany(paths)) if paths.len()==2));
    press(&mut p, Key::End);
    assert!(p.files.pending.as_ref().unwrap().mutating());
    p.launch(Screen::Tools(ToolPage::Files));
    assert!(p.files.pending.as_ref().unwrap().mutating());
    assert_ne!(p.screen, Screen::Tools(ToolPage::Files));
}

#[test]
fn filtered_file_selection_cancel_restores_the_unfiltered_cursor() {
    use files::FileView;
    use tools::ToolPage;
    let mut p = Shell::new(Mailbox::default());
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.pending = None;
    p.files.entries = [true, false, false]
        .into_iter()
        .enumerate()
        .map(|(i, directory)| crate::storage::files::Entry {
            name: i.to_string(),
            path: format!("/{i}"),
            directory,
            len: 0,
        })
        .collect();
    p.files.selected = 2;
    p.files.view = FileView::DeleteSelect;
    p.files.delete_kind = Some(false);
    assert_eq!(p.delete_entries().len(), 2);
    assert!(p.delete_entries().iter().all(|e| !e.directory));
    p.files.delete_kind = Some(true);
    assert_eq!(p.delete_entries().len(), 1);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.files.view, FileView::Browse);
    assert_eq!(p.index, 2);
}

#[test]
fn converter_pair_navigation_bidirectional_fields_and_numeric_limits() {
    use tools::ToolPage;
    let mut p = Shell::new(Mailbox::default());
    p.launch(Screen::Tools(ToolPage::Converter));
    press(&mut p, Key::Center);
    assert_eq!(p.tools.page, ToolPage::ConversionPairs(0));
    assert_eq!(p.tools_items(), ["华氏-摄氏", "开氏-华氏", "开氏-摄氏"]);
    press(&mut p, Key::Center);
    assert!(p.tools.conversion.fields.iter().all(|e| e.units.is_empty()));
    p.text("32");
    assert_eq!(p.tools.conversion.fields[1].text(), "-0.0000002222");
    let before = p.tools.conversion.fields.each_ref().map(|e| e.text());
    press(&mut p, Key::Down);
    assert_eq!(p.tools.conversion.focus, 1);
    assert_eq!(
        p.tools.conversion.fields.each_ref().map(|e| e.text()),
        before
    );
    assert_eq!(p.conversion_pair().labels, ["华氏：", "摄氏："]);
    press(&mut p, Key::SoftLeft);
    p.text("100");
    assert_eq!(p.tools.conversion.fields[0].text(), "212");
    press(&mut p, Key::SoftRight); // Deletes instead of leaving the form.
    assert_eq!(p.tools.page, ToolPage::Conversion(0));
    assert_eq!(p.tools.conversion.fields[1].text(), "10");
    press(&mut p, Key::Left);
    p.text("2");
    assert_eq!(p.tools.conversion.fields[1].text(), "120");
    p.text("x");
    assert_eq!(p.tools.conversion.fields[1].text(), "120");
    press(&mut p, Key::SoftLeft);
    p.text("123456789012");
    p.text("3");
    assert_eq!(p.tools.conversion.fields[1].text(), "123456789012");
    press(&mut p, Key::SoftLeft);
    p.text("1");
    press(&mut p, Key::Hash);
    press(&mut p, Key::Digit(5));
    press(&mut p, Key::Hash); // A second decimal separator is rejected.
    assert_eq!(p.tools.conversion.fields[1].text(), "1.5");
    assert!(p.tools.conversion.fields[1].numeric);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.tools.page, ToolPage::ConversionPairs(0));
    press(&mut p, Key::Down);
    press(&mut p, Key::Center);
    assert_eq!(p.tools.conversion.pair, 1);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.index, 1);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.tools.page, ToolPage::Converter);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.tools.page, ToolPage::Other);
    assert_eq!(p.index, 3);
}

#[test]
fn radio_menu_cancel_keeps_tuning_and_disabled_speaker_does_not_mutate() {
    use tools::ToolPage;
    let mut p = Shell::new(Mailbox::default());
    p.launch(Screen::Tools(ToolPage::Radio));
    p.tools.frequency = 995;
    p.tools.radio_on = true;
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.tools.page, ToolPage::RadioOptions);
    assert_eq!(p.tools_items()[1], "添加至收藏夹");
    for _ in 0..2 {
        press(&mut p, Key::Down);
    }
    let speaker = p.tools.radio_speaker;
    press(&mut p, Key::Center);
    assert_eq!(p.tools.radio_speaker, speaker);
    assert_eq!(p.tools.page, ToolPage::RadioOptions);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.tools.page, ToolPage::Radio);
    assert_eq!(p.tools.frequency, 995);
    assert!(p.tools.radio_on);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Up); // Wrap to Close without activating it.
    assert_eq!(p.index, 6);
    assert!(p.tools.radio_on);
    press(&mut p, Key::Center);
    assert!(!p.tools.radio_on);
    assert_eq!(p.screen, Screen::Home);
}

#[test]
fn music_refresh_wait_blocks_stale_library_activation_but_can_leave() {
    let mut p = Shell::new(Mailbox::default());
    p.launch(Screen::Music);
    p.playback.view = MusicView::Root;
    p.playback.model.tracks = vec![("old.wav".into(), 2.)];
    press(&mut p, Key::SoftLeft);
    assert_eq!(p.menu_labels(MenuKind::MusicLibrary), ["手动更新"]);
    press(&mut p, Key::SoftRight); // Cancel must not launch a scan.
    assert!(!p.playback.refresh);
    press(&mut p, Key::SoftLeft);
    press(&mut p, Key::Center);
    assert!(p.music_refresh_visible());
    for key in [Key::Down, Key::Center, Key::SoftLeft] {
        press(&mut p, key);
    }
    assert_eq!(p.index, 0);
    assert!(p.overlay.is_none());
    assert!(!p.playback.model.playing);
    p.playback.refresh = false; // Worker consumed the request, still pending.
    p.playback.loading = true;
    press(&mut p, Key::Center);
    assert!(p.music_refresh_visible());
    press(&mut p, Key::SoftRight);
    assert_eq!(p.screen, Screen::Music);
    assert!(p.playback.wait_dismissed);
    assert!(!p.music_refresh_visible());
    assert!(p.playback.loading); // Leaving does not claim the scan was canceled.
    p.playback.loading = false;
    p.launch(Screen::Music);
    assert!(!p.music_refresh_visible());
    press(&mut p, Key::Down);
    assert_eq!(p.index, 1);
}

#[test]
fn optional_apps_cannot_be_opened_by_menu_shortcut_or_direct_route() {
    let mut p = Shell::new(Mailbox::default());
    let mut value = serde_json::to_value(crate::config::Apps::default()).unwrap();
    for enabled in value.as_object_mut().unwrap().values_mut() {
        *enabled = false.into();
    }
    p.apps = serde_json::from_value(value).unwrap();
    p.features = crate::config::RuntimeConfig {
        apps: p.apps,
        ..Default::default()
    }
    .effective_features();
    let visible = crate::menu::visible(p.apps);
    assert_eq!(
        visible.iter().map(|e| e.label).collect::<Vec<_>>(),
        ["信息", "通话历史记录", "文件", "设置", "通讯录"]
    );
    for e in &crate::menu::ENTRIES {
        if !p.apps.allows(e.screen) {
            p.launch(e.screen);
            assert_eq!(p.screen, Screen::Home);
            p.go(e.screen);
            assert_eq!(p.screen, Screen::Home);
        }
    }
    for shortcut in [4, 5, 6, 7, 9, 10, 11, 12, 13, 15, 16] {
        p.settings.preferences.left_shortcut = shortcut;
        assert_eq!(p.visible_shortcut(true), 0);
        press(&mut p, Key::SoftLeft);
        assert_eq!(p.screen, Screen::Home);
        p.settings.page = settings::SettingsPage::Shortcut { left: true };
        assert!(!p.setting_enabled(shortcut));
    }
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Menu);
    assert_eq!(p.index, 0);
    for _ in 0..30 {
        press(&mut p, Key::Right);
        assert!(p.index < visible.len());
    }
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Messages);
    // Independently disabling the flashlight also removes its Other sub-route.
    p.apps.utilities = true;
    assert_eq!(
        p.other_pages(),
        [
            tools::ToolPage::Timer,
            tools::ToolPage::Stopwatch,
            tools::ToolPage::Converter
        ]
    );
    p.launch(Screen::Tools(tools::ToolPage::Other));
    p.index = 2;
    press(&mut p, Key::Center);
    assert_eq!(p.tools.page, tools::ToolPage::Converter);
    press(&mut p, Key::SoftRight);
    assert_eq!(p.index, 2);
}

#[test]
fn disabled_app_reminders_stay_saved_without_blocking_input_or_queuing_sound() {
    use alarm::{Alert, ReminderSource};
    let mut p = Shell::new(Mailbox::default());
    p.apps.alarms = false;
    p.apps.calendar = false;
    p.apps.utilities = false;
    for source in [
        ReminderSource::Alarm,
        ReminderSource::Calendar,
        ReminderSource::Legacy,
    ] {
        p.organizer.reminders.alerts.push_back(Alert {
            source,
            name: "Fixture".into(),
            tone: "无".into(),
            time: "12:00".into(),
            snooze: 10,
        });
    }
    p.tools.timer.expired = true;
    p.tick(1000, 0.);
    assert!(p.active_reminder().is_none());
    assert!(p.overlay.is_none());
    assert!(p.cues.is_empty());
    press(&mut p, Key::Center);
    assert_eq!(p.screen, Screen::Menu);
    assert_eq!(p.organizer.reminders.alerts.len(), 3);
    p.apps.calendar = true;
    assert!(matches!(
        p.active_reminder().unwrap().source,
        ReminderSource::Calendar
    ));
    press(&mut p, Key::Center); // Dismiss only the active calendar reminder.
    assert_eq!(p.organizer.reminders.alerts.len(), 2);
    assert!(p.active_reminder().is_none());
    p.apps.alarms = true;
    assert!(matches!(
        p.active_reminder().unwrap().source,
        ReminderSource::Alarm
    ));
    let legacy: Alert =
        serde_json::from_str(r#"{"tone":"无","name":"old","time":"12:00","snooze":10}"#).unwrap();
    assert!(matches!(legacy.source, ReminderSource::Legacy));
}

#[test]
fn contact_editing_isolated_from_calendar_draft_and_return_focus() {
    let mut p = shell();
    p.launch(Screen::Organizer(organizer::OrganizerPage::EventForm));
    p.organizer_form(organizer::OrganizerPage::EventForm, None);
    p.organizer.draft[0] = "Appointment".into();
    p.organizer.event.return_index = 4;
    let draft = p.organizer.draft.clone();
    p.launch(Screen::Contacts);
    p.contact_form(None);
    p.text("Example");
    p.contact_load_field(3);
    p.contacts.ui.input.insert("123");
    assert_eq!(p.organizer.draft, draft);
    assert_eq!(p.organizer.event.return_index, 4);
    p.launch(Screen::Organizer(organizer::OrganizerPage::Calendar));
    assert!(!p.contact_editor_active());
}
