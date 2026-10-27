//! CPU-only layout checks. No App, Media, USB, window or audio output is created.
use super::*;
use crate::phone::{files::FileView, tools::ToolPage};

fn frame(c: &mut Canvas, p: &mut Phone, name: &str) {
    render(c, p);
    if let Some(path) = std::env::var_os("TA1278_AUDIT_FRAMES") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&path).unwrap();
        c.save(&path.join(format!("{name}.png"))).unwrap();
    }
}

#[test]
fn audited_timer_fields_and_storage_pages_render_without_external_services() {
    let mut p = Phone::new(Default::default());
    p.features.disable("sounds,music,video,network").unwrap();
    p.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Tools(ToolPage::Timer));
    frame(&mut c, &mut p, "timer-empty");
    p.index = 1;
    p.text("A");
    frame(&mut c, &mut p, "timer-named");
    assert_eq!(c.caret.unwrap().1, 125);
    let time_band = c.pixels[24 * 240 * 4..100 * 240 * 4].to_vec();
    p.editor_mut().unwrap().insert(&"W".repeat(19));
    frame(&mut c, &mut p, "timer-long-name");
    assert_eq!(c.caret.unwrap().1, 125);
    assert_eq!(&c.pixels[24 * 240 * 4..100 * 240 * 4], time_band.as_slice());
    assert!(!p.editor_mut().unwrap().insert("overflow"));
    p.tools.timer_name = "A".into();
    p.tools.timer.start(60);
    p.tools.page = ToolPage::TimerMenu;
    p.index = 0;
    frame(&mut c, &mut p, "timer-menu");
    p.overlay = Some(Overlay::TimerElapsed);
    frame(&mut c, &mut p, "timer-expired");
    p.overlay = None;
    for (page, name) in [
        (ToolPage::Other, "other"),
        (ToolPage::Calculator, "calculator"),
        (ToolPage::Stopwatch, "stopwatch"),
        (ToolPage::Radio, "radio"),
        (ToolPage::Recorder, "recorder"),
    ] {
        p.launch(Screen::Tools(page));
        frame(&mut c, &mut p, name);
    }
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.capacity = Some((32 * 1024 * 1024, 30 * 1024 * 1024));
    p.files.writable = true;
    p.files.entries = ["Alarm", "Audio", "Backups", "browser", "Ebook", "Filearray"]
        .map(|name| crate::storage::files::Entry {
            name: name.into(),
            path: format!("/{name}"),
            directory: true,
            len: 0,
        })
        .into();
    for (view, name) in [
        (FileView::Storage, "files-root"),
        (FileView::StorageOptions, "files-options"),
        (FileView::StorageDetails, "files-details"),
        (FileView::Browse, "files-folders"),
        (FileView::Options, "folder-options"),
    ] {
        p.index = 0;
        p.files.view = view;
        frame(&mut c, &mut p, name);
    }
    p.files.view = FileView::Browse;
    p.overlay = Some(Overlay::Text {
        target: TextTarget::FileName(true),
        editor: Box::new(Editor::new("文件夹 1", 255)),
    });
    p.editor_mut()
        .unwrap()
        .prediction
        .set_language(Some(phone_core::predict::Language::Chinese));
    frame(&mut c, &mut p, "folder-name");
    assert_eq!(c.caret.unwrap().1, 221);
}

#[test]
fn calendar_viewport_preserves_title_and_footer_and_file_checks_use_shared_rows() {
    use crate::phone::organizer::OrganizerPage as Page;
    use phone_core::input::{Event, Key, Phase};
    let mut p = Phone::new(Default::default());
    p.features.disable("sounds,music,video,network").unwrap();
    p.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Organizer(Page::Calendar));
    p.organizer_form(Page::EventForm, None);
    p.organizer.draft[1] = "09:00".into();
    p.organizer.draft[2] = "15分钟前".into();
    frame(&mut c, &mut p, "calendar-inline-empty");
    let title = c.pixels[..24 * 240 * 4].to_vec();
    let footer = c.pixels[284 * 240 * 4..].to_vec();
    for _ in 0..6 {
        p.handle(
            Event {
                key: Key::Down,
                phase: Phase::Down,
            },
            0,
        );
    }
    frame(&mut c, &mut p, "calendar-inline-note");
    assert_eq!(&c.pixels[..24 * 240 * 4], title.as_slice());
    assert_eq!(&c.pixels[284 * 240 * 4..], footer.as_slice());
    assert_eq!(c.caret.unwrap().1, 244);
    p.text("Note");
    frame(&mut c, &mut p, "calendar-inline-note-filled");
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        0,
    );
    frame(&mut c, &mut p, "calendar-form-menu");
    p.overlay = None;
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.pending = None;
    p.files.writable = true;
    p.files.entries = ["Alarm", "Audio", "Backups", "browser", "Ebook", "Filearray"]
        .map(|name| crate::storage::files::Entry {
            name: name.into(),
            path: format!("/{name}"),
            directory: true,
            len: 0,
        })
        .into();
    p.files.view = FileView::DeleteSelect;
    frame(&mut c, &mut p, "files-delete-select");
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        0,
    );
    frame(&mut c, &mut p, "files-delete-checked");
    p.handle(
        Event {
            key: Key::SoftLeft,
            phase: Phase::Down,
        },
        0,
    );
    frame(&mut c, &mut p, "files-delete-options");
    p.handle(
        Event {
            key: Key::Center,
            phase: Phase::Down,
        },
        0,
    );
    frame(&mut c, &mut p, "files-delete-confirm");
    assert!(p.files.pending.is_none()); // Rendering cannot submit a deletion.
}

#[test]
fn converter_native_fields_keep_units_and_baselines_when_focus_changes() {
    use phone_core::input::{Event, Key, Phase};
    fn press(p: &mut Phone, key: Key) {
        p.handle(
            Event {
                key,
                phase: Phase::Down,
            },
            0,
        );
    }
    let mut p = Phone::new(Default::default());
    p.features.disable("sounds,music,video,network").unwrap();
    p.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Tools(ToolPage::Converter));
    frame(&mut c, &mut p, "converter-root");
    press(&mut p, Key::Center);
    frame(&mut c, &mut p, "converter-temperature-pairs");
    press(&mut p, Key::Center);
    frame(&mut c, &mut p, "converter-empty");
    p.text("32");
    frame(&mut c, &mut p, "converter-result");
    let units = p.conversion_pair().labels;
    assert_eq!(c.caret.unwrap().1, 57);
    assert_eq!(p.tools.conversion.fields[1].text(), "-0.0000002222");
    press(&mut p, Key::Down);
    frame(&mut c, &mut p, "converter-reverse-focus");
    assert_eq!(c.caret.unwrap().1, 131);
    assert_eq!(p.conversion_pair().labels, units);
    assert_eq!(p.tools.conversion.fields[0].text(), "32");
    // Neither label overlaps either entry. The native input underline remains at y=82/156.
    assert_eq!(
        &c.pixels[(156 * 240 + 6) * 4..(156 * 240 + 7) * 4],
        &c.accent(3)
    );
    p.launch(Screen::Tools(ToolPage::Calculator));
    for d in [5, 0] {
        press(&mut p, Key::Digit(d));
    }
    press(&mut p, Key::SoftLeft);
    for d in [2, 0, 0] {
        press(&mut p, Key::Digit(d));
    }
    press(&mut p, Key::Center);
    frame(&mut c, &mut p, "calculator-percent-ratio");
    assert_eq!(p.tools.calculator.display(), "25");
}

#[test]
fn music_and_radio_popups_preserve_their_background_and_wait_has_only_close() {
    let mut p = Phone::new(Default::default());
    p.features.disable("sounds,video,network").unwrap();
    p.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.launch(Screen::Tools(ToolPage::Radio));
    p.tools.radio_on = true;
    frame(&mut c, &mut p, "radio-base");
    let base = c.pixels.clone();
    p.tools.page = ToolPage::RadioOptions;
    frame(&mut c, &mut p, "radio-options");
    // The frequency panel survives above the menu, dimmed instead of erased.
    let pixel = (30 * 240 + 20) * 4;
    assert_eq!(c.pixels[pixel], (u16::from(base[pixel]) * 51 / 255) as u8);
    assert!(c.pixels[pixel] > 0);
    assert_eq!(&c.pixels[(48 * 240) * 4..(48 * 240 + 1) * 4], &c.accent(3));
    // No base-page strip is exposed under the final menu row.
    assert_eq!(
        &c.pixels[(280 * 240 + 20) * 4..(280 * 240 + 21) * 4],
        &[0, 0, 0, 255]
    );
    p.index = 6;
    frame(&mut c, &mut p, "radio-close-option");
    p.launch(Screen::Music);
    p.music_view = MusicView::Root;
    frame(&mut c, &mut p, "music-library");
    let library = c.pixels.clone();
    p.overlay = Some(Overlay::Menu {
        kind: MenuKind::MusicLibrary,
        index: 0,
    });
    frame(&mut c, &mut p, "music-library-options");
    assert_eq!(
        c.pixels[pixel],
        (u16::from(library[pixel]) * 51 / 255) as u8
    );
    assert!(c.pixels[pixel] > 0);
    p.overlay = None;
    p.music_loading = true;
    frame(&mut c, &mut p, "music-refresh-wait");
    let footer = c.pixels[284 * 240 * 4..].to_vec();
    Softkeys::new(Button::None, Button::None, Button::Close).draw(&mut c);
    assert_eq!(footer, c.pixels[284 * 240 * 4..]);
}

#[test]
fn configured_core_menu_renders_with_hidden_shortcut_and_compact_focus() {
    let mut p = Phone::new(Default::default());
    p.features.disable("sounds,network").unwrap();
    let mut flags = serde_json::to_value(crate::config::Apps::default()).unwrap();
    for value in flags.as_object_mut().unwrap().values_mut() {
        *value = false.into();
    }
    p.apps = serde_json::from_value(flags).unwrap();
    let mut c = Canvas::new().unwrap();
    assert_eq!(p.visible_shortcut(true), 0); // Stored browser shortcut is retained but hidden.
    frame(&mut c, &mut p, "configured-home");
    p.go(Screen::Menu);
    assert_eq!(p.index, 0);
    frame(&mut c, &mut p, "configured-core-menu");
    p.index = 4;
    frame(&mut c, &mut p, "configured-core-last");
    assert_eq!(
        crate::menu::visible(p.apps)[p.index].screen,
        Screen::Contacts
    );
}

#[test]
fn shared_popup_app_pages_render_with_synthetic_content() {
    use crate::phone::organizer::OrganizerPage as Page;
    let mut p = Phone::new(Default::default());
    p.features.disable("sounds,music,video,network").unwrap();
    p.accent = 4;
    let mut c = Canvas::new().unwrap();
    p.local_data.contacts.push(
        ["", "Example", "", "1230000", "", "", "", "预设", ""]
            .map(str::to_owned)
            .into(),
    );
    p.launch(Screen::Contacts);
    for (page, name) in [
        (Page::ContactsMenu, "framework-contacts-menu"),
        (Page::ContactMenu, "framework-contact-menu"),
        (Page::ContactGroupMenu, "framework-contact-group-menu"),
    ] {
        p.organizer.page = page;
        p.organizer.selected = 0;
        p.index = 0;
        frame(&mut c, &mut p, name);
    }
    p.organizer_form(Page::ContactForm, None);
    p.organizer.page = Page::ContactFormMenu;
    frame(&mut c, &mut p, "framework-contact-save");
    p.launch(Screen::Organizer(Page::Calendar));
    p.organizer.date = phone_core::calendar::Date::new(2026, 1, 1).unwrap();
    p.organizer.page = Page::CalendarMenu;
    frame(&mut c, &mut p, "framework-calendar-menu");
    p.launch(Screen::Organizer(Page::Alarms));
    p.organizer.page = Page::AlarmMenu;
    frame(&mut c, &mut p, "framework-alarm-menu");
}
