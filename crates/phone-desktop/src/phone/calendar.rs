//! Inline schedule form: native child editors, transactional local data.
use super::*;
use organizer::OrganizerPage as Page;
use phone_core::calendar::Date;

pub struct EventUi {
    pub input: Editor,
    pub field: usize,
    pub digit: usize,
    pub date: Date,
    pub return_index: usize,
}
impl Default for EventUi {
    fn default() -> Self {
        Self {
            input: Editor::new("", 40),
            field: 0,
            digit: 0,
            date: Date::default(),
            return_index: 0,
        }
    }
}
impl Phone {
    pub fn event_editor_active(&self) -> bool {
        matches!(self.screen, Screen::Organizer(_))
            && self.organizer.page == Page::EventForm
            && matches!(self.index, 0 | 3 | 6)
    }
    pub(super) fn event_load_field(&mut self, field: usize) {
        self.index = field;
        self.organizer.event.field = field;
        self.organizer.event.digit = 0;
        let mut input = Editor::new(
            self.organizer.draft.get(field).map_or("", String::as_str),
            40,
        );
        input
            .prediction
            .set_language(Some(phone_core::predict::Language::Chinese));
        self.organizer.event.input = input;
    }
    fn event_sync(&mut self) -> bool {
        let edit = &mut self.organizer.event;
        if matches!(edit.field, 0 | 3 | 6) {
            if !edit.input.accept_prediction() {
                return false;
            }
            edit.input.commit();
            self.organizer.draft[edit.field] = edit.input.text();
        }
        true
    }
    pub fn event_datetime(&self) -> String {
        format!(
            "{} {}",
            self.organizer.event.date.iso(),
            self.organizer.draft[1]
        )
    }
    pub(super) fn event_form_key(&mut self, key: Key) -> bool {
        if self.organizer.page == Page::EventFormMenu {
            match key {
                Key::Center => {
                    self.organizer.page = Page::EventForm;
                    self.index = self.organizer.event.return_index;
                    self.organizer_save();
                }
                Key::SoftRight => {
                    self.organizer.page = Page::EventForm;
                    self.index = self.organizer.event.return_index;
                }
                _ => {}
            }
            return true;
        }
        if self.organizer.page != Page::EventForm {
            return false;
        }
        match key {
            Key::Up | Key::Down => {
                if self.event_sync() {
                    let next = if key == Key::Up {
                        (self.index + 6) % 7
                    } else {
                        (self.index + 1) % 7
                    };
                    self.event_load_field(next);
                }
            }
            Key::Left if self.index == 1 => {
                self.organizer.event.digit = self.organizer.event.digit.saturating_sub(1);
            }
            Key::Right if self.index == 1 => {
                self.organizer.event.digit = (self.organizer.event.digit + 1).min(11);
            }
            Key::Digit(d) if self.index == 1 && d <= 9 => {
                let mut value = self.event_datetime().into_bytes();
                let offset = DATETIME_DIGITS[self.organizer.event.digit];
                value[offset] = b'0' + d;
                let value = String::from_utf8(value).unwrap();
                if let Some(date) = Date::parse(&value[..10])
                    && organizer::valid_time(&value[11..])
                {
                    self.organizer.event.date = date;
                    self.organizer.draft[1] = value[11..].into();
                    self.organizer.event.digit = (self.organizer.event.digit + 1).min(11);
                }
            }
            Key::Center => {
                if !self.event_sync() {
                    return true;
                }
                if matches!(self.index, 2 | 4 | 5) {
                    self.contacts.ui.return_index = self.index;
                    self.organizer.page = match self.index {
                        2 => Page::EventReminder,
                        4 => Page::EventRingtone,
                        _ => Page::EventRepeat,
                    };
                    let current = &self.organizer.draft[self.index];
                    self.index = self
                        .organizer_items()
                        .iter()
                        .position(|s| s == current)
                        .unwrap_or(0);
                } else {
                    self.organizer_save();
                }
            }
            Key::SoftLeft => {
                if self.event_sync() {
                    self.organizer.event.return_index = self.index;
                    self.organizer.page = Page::EventFormMenu;
                    self.index = 0;
                }
            }
            Key::SoftRight
                if self.event_editor_active() && !self.organizer.event.input.units.is_empty() =>
            {
                self.organizer.event.input.backspace();
            }
            Key::SoftRight => {
                self.organizer.page = Page::Calendar;
                self.index = 0;
            }
            _ => {}
        }
        true
    }
}
pub const DATETIME_DIGITS: [usize; 12] = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15];
