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
        .map(|name| crate::local_files::Entry {
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
        .map(|name| crate::local_files::Entry {
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
