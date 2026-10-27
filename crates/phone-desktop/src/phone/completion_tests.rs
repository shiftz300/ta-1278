use super::tests::press;
use super::*;
use organizer::OrganizerPage as Page;

#[test]
fn calendar_inline_fields_commit_together_and_editing_a_recurrence_keeps_start_date() {
    let mut p = Phone::new(Mailbox::default());
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
    assert_eq!(p.local_data.events.len(), 1);
    assert_eq!(p.local_data.events[0].date, date);
    p.local_data.events[0].fields[5] = "每周".into();
    p.organizer.date = date.next(7);
    p.organizer_form(Page::EventForm, Some(0));
    press(&mut p, Key::Center);
    assert_eq!(p.local_data.events[0].date, date);
    p.organizer_form(Page::EventForm, Some(0));
    p.text("not saved");
    press(&mut p, Key::End);
    assert_eq!(p.local_data.events[0].fields[0], "Appointment");
}

#[test]
fn file_checkbox_selection_requires_confirmation_and_pending_work_survives_navigation() {
    use files::{FileRequest, FileView};
    use tools::ToolPage;
    let mut p = Phone::new(Mailbox::default());
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.pending = None;
    p.files.view = FileView::Browse;
    p.files.writable = true;
    p.files.entries = ["a.txt", "b.txt"]
        .map(|s| crate::local_files::Entry {
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
    let mut p = Phone::new(Mailbox::default());
    p.launch(Screen::Tools(ToolPage::Files));
    p.files.pending = None;
    p.files.entries = [true, false, false]
        .into_iter()
        .enumerate()
        .map(|(i, directory)| crate::local_files::Entry {
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
