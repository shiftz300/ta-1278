//! Local phonebook routes; the nine-field persisted contact schema is unchanged.
mod groups;
use super::*;
use organizer::{OrganizerPage as Page, contact_name};
pub struct ContactUi {
    pub query: Editor,
    pub input: Editor,
    pub field: usize,
    pub return_index: usize,
    pub root_index: usize,
    pub date_digit: usize,
    pub group_filter: Option<String>,
    pub selected_group: Option<usize>,
}
impl Default for ContactUi {
    fn default() -> Self {
        Self {
            query: Editor::new("", 40),
            input: Editor::new("", 40),
            field: 1,
            return_index: 0,
            root_index: 0,
            date_digit: 0,
            group_filter: None,
            selected_group: None,
        }
    }
}
pub fn is_contact(page: Page) -> bool {
    matches!(
        page,
        Page::Contacts
            | Page::ContactsMenu
            | Page::ContactForm
            | Page::ContactFormMenu
            | Page::ContactDetail
            | Page::ContactMenu
            | Page::DeleteContact
            | Page::ContactGroups
            | Page::ContactGroupChoice
            | Page::ContactRingtone
            | Page::ContactGroupMenu
            | Page::ContactGroupDelete
    )
}
pub const ROOT_MENU: [&str; 6] = [
    "通话组",
    "增加新联系人",
    "删除联系人",
    "分享联系人",
    "单键拨号",
    "设置",
];
pub const DETAIL_MENU: [&str; 4] = ["编辑联系人", "删除联系人", "分享联系人", "单键拨号"];
impl Phone {
    pub fn contact_editor_active(&self) -> bool {
        matches!(self.screen, Screen::Contacts | Screen::Organizer(_))
            && (self.organizer.page == Page::Contacts
                || (self.organizer.page == Page::ContactForm && (1..=5).contains(&self.index)))
    }
    pub fn contact_ids(&self) -> Vec<usize> {
        let query = self.organizer.contact.query.text().to_lowercase();
        self.local_data
            .contacts
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                (self
                    .organizer
                    .contact
                    .group_filter
                    .as_ref()
                    .is_none_or(|g| c.get(8) == Some(g)))
                    && (query.is_empty()
                        || c.iter()
                            .skip(1)
                            .take(5)
                            .any(|s| s.to_lowercase().contains(&query)))
            })
            .map(|(i, _)| i)
            .collect()
    }
    pub fn contact_numbers(&self) -> Vec<(usize, String)> {
        self.local_data
            .contacts
            .get(self.organizer.selected)
            .map(|c| {
                (3..=5)
                    .filter_map(|i| c.get(i).filter(|s| !s.is_empty()).map(|s| (i, s.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn contact_menu_enabled(&self, index: usize) -> bool {
        match self.organizer.page {
            Page::ContactsMenu => index <= 1 || (index == 2 && !self.contact_ids().is_empty()),
            Page::ContactMenu => index < 2,
            _ => true,
        }
    }
    pub fn contact_load_field(&mut self, field: usize) {
        self.index = field;
        self.organizer.contact.field = field;
        let value = self.organizer.draft.get(field).cloned().unwrap_or_default();
        let mut input = Editor::new(&value, 40);
        input.numeric = (3..=6).contains(&field);
        if field == 6 {
            self.organizer.contact.date_digit = 0;
        }
        if matches!(field, 1 | 2) {
            input
                .prediction
                .set_language(Some(phone_core::predict::Language::Chinese));
        }
        self.organizer.contact.input = input;
    }
    fn contact_sync(&mut self) -> bool {
        if !(1..=5).contains(&self.organizer.contact.field) {
            return true;
        }
        if !self.organizer.contact.input.accept_prediction() {
            return false;
        }
        self.organizer.contact.input.commit();
        let field = self.organizer.contact.field;
        if let Some(value) = self.organizer.draft.get_mut(field) {
            *value = self.organizer.contact.input.text();
        }
        true
    }
    fn contact_return(&mut self) {
        if self.chat.contact_return {
            self.chat.contact_return = false;
            self.go(Screen::Conversation);
        } else {
            self.organizer.page = if self.organizer.editing.is_some() {
                Page::ContactDetail
            } else {
                Page::Contacts
            };
            self.index = if self.organizer.editing.is_some() {
                0
            } else {
                self.organizer.contact.root_index
            };
        }
    }
    fn contact_save(&mut self) {
        if !self.contact_sync() {
            return;
        }
        if self
            .organizer
            .draft
            .iter()
            .skip(1)
            .take(5)
            .all(|s| s.trim().is_empty())
        {
            self.notice("请输入姓名或号码。");
            return;
        }
        let date = &self.organizer.draft[6];
        if !date.is_empty() && phone_core::calendar::Date::parse(date).is_none() {
            self.notice("日期格式：YYYY-MM-DD");
            return;
        }
        let fields = self.organizer.draft.clone();
        if let Some(i) = self.organizer.editing {
            self.local_data.contacts[i] = fields;
        } else {
            self.local_data.contacts.push(fields);
        }
        self.local_dirty = true;
        self.organizer.editing = None;
        self.contact_return();
    }
    fn contact_compose(&mut self, number: String) {
        if !self.organizer.pick_recipient {
            self.compose = self.new_compose();
            self.chat = Default::default();
        }
        self.compose.to = Editor::new(&number, 40);
        self.compose.to.numeric = true;
        self.compose.recipient = false;
        self.organizer.pick_recipient = false;
        self.go(Screen::Compose);
    }
    pub(super) fn contacts_key(&mut self, key: Key) -> bool {
        let page = self.organizer.page;
        if !is_contact(page) {
            return false;
        }
        if self.contact_groups_key(key) {
            return true;
        }
        if page == Page::ContactForm && self.index == 6 {
            match key {
                Key::Left => {
                    self.organizer.contact.date_digit =
                        self.organizer.contact.date_digit.saturating_sub(1);
                    return true;
                }
                Key::Right => {
                    self.organizer.contact.date_digit =
                        (self.organizer.contact.date_digit + 1).min(7);
                    return true;
                }
                Key::Digit(d) if d < 10 => {
                    let value = &self.organizer.draft[6];
                    let mut bytes = if phone_core::calendar::Date::parse(value).is_some() {
                        value.as_bytes().to_vec()
                    } else {
                        b"1900-01-01".to_vec()
                    };
                    bytes[phone_core::calendar::Date::DIGIT_OFFSETS
                        [self.organizer.contact.date_digit]] = b'0' + d;
                    let next = String::from_utf8(bytes).unwrap();
                    if phone_core::calendar::Date::parse(&next).is_some() {
                        self.organizer.draft[6] = next;
                        self.organizer.contact.date_digit =
                            (self.organizer.contact.date_digit + 1).min(7);
                    }
                    return true;
                }
                _ => {}
            }
        }
        match page {
            Page::ContactForm => match key {
                Key::Delete if matches!(self.index, 0 | 7 | 8) => {
                    self.organizer.draft[self.index] = match self.index {
                        0 => String::new(),
                        _ => "无".into(),
                    }
                }
                Key::Center if matches!(self.index, 0 | 7 | 8) => {
                    if !self.contact_sync() {
                        return true;
                    }
                    match self.index {
                        0 => self.open_media(media::Purpose::ContactPicture),
                        7 => {
                            self.organizer.page = Page::ContactRingtone;
                            self.index = 0;
                        }
                        _ => {
                            self.organizer.page = Page::ContactGroupChoice;
                            self.index = 0;
                        }
                    }
                }
                Key::Up | Key::Down => {
                    if self.contact_sync() {
                        let next = if key == Key::Up {
                            (self.index + 8) % 9
                        } else {
                            (self.index + 1) % 9
                        };
                        self.contact_load_field(next);
                    }
                }
                Key::SoftLeft => {
                    if self.contact_sync() {
                        self.organizer.contact.return_index = self.index;
                        self.organizer.page = Page::ContactFormMenu;
                        self.index = 0;
                    }
                }
                Key::SoftRight => {
                    if (1..=5).contains(&self.index)
                        && (!self.organizer.contact.input.units.is_empty()
                            || self.organizer.contact.input.prediction.active())
                    {
                        self.organizer.contact.input.backspace();
                    } else {
                        self.contact_return();
                    }
                }
                _ => {}
            },
            Page::ContactFormMenu => match key {
                Key::Center => self.contact_save(),
                Key::SoftRight => {
                    self.organizer.page = Page::ContactForm;
                    self.index = self.organizer.contact.return_index;
                }
                _ => {}
            },
            Page::Contacts => {
                let ids = self.contact_ids();
                match key {
                    Key::Up if !ids.is_empty() => {
                        self.index = (self.index + ids.len() - 1) % ids.len()
                    }
                    Key::Down if !ids.is_empty() => self.index = (self.index + 1) % ids.len(),
                    Key::SoftLeft => {
                        self.organizer.contact.return_index = self.index;
                        self.organizer.contact.root_index = self.index;
                        self.organizer.page = Page::ContactsMenu;
                        self.index = 0;
                    }
                    Key::Center if !ids.is_empty() => {
                        self.organizer.contact.root_index = self.index;
                        self.organizer.selected = ids[self.index.min(ids.len() - 1)];
                        if self.organizer.pick_recipient {
                            if let Some((_, n)) = self.contact_numbers().first().cloned() {
                                self.contact_compose(n);
                            }
                        } else {
                            self.organizer.page = Page::ContactDetail;
                            self.index = 0;
                        }
                    }
                    Key::SoftRight => {
                        if self.organizer.pick_recipient {
                            self.organizer.pick_recipient = false;
                            self.go(Screen::Compose);
                        } else if self.organizer.contact.group_filter.take().is_some() {
                            self.organizer.page = Page::ContactGroups;
                            self.index = 0;
                        } else {
                            self.leave_app();
                        }
                    }
                    _ => {}
                }
            }
            Page::ContactsMenu | Page::ContactMenu => {
                let n = if page == Page::ContactsMenu {
                    ROOT_MENU.len()
                } else {
                    DETAIL_MENU.len()
                };
                match key {
                    Key::Up => self.index = (self.index + n - 1) % n,
                    Key::Down => self.index = (self.index + 1) % n,
                    Key::SoftRight => {
                        self.organizer.page = if page == Page::ContactsMenu {
                            Page::Contacts
                        } else {
                            Page::ContactDetail
                        };
                        self.index = self.organizer.contact.return_index;
                    }
                    Key::Center if self.contact_menu_enabled(self.index) => {
                        if page == Page::ContactsMenu && self.index == 0 {
                            self.organizer.page = Page::ContactGroups;
                            self.index = 0;
                        } else if page == Page::ContactsMenu && self.index == 1 {
                            self.organizer_form(Page::ContactForm, None);
                        } else if page == Page::ContactMenu && self.index == 0 {
                            self.organizer_form(Page::ContactForm, Some(self.organizer.selected));
                        } else {
                            if page == Page::ContactsMenu {
                                let ids = self.contact_ids();
                                self.organizer.selected =
                                    ids[self.organizer.contact.return_index.min(ids.len() - 1)];
                            }
                            self.organizer.page = Page::DeleteContact;
                            self.index = 0;
                        }
                    }
                    _ => {}
                }
            }
            Page::ContactDetail => match key {
                Key::Up | Key::Down => {
                    let n = self.contact_numbers().len() * 2;
                    if n > 0 {
                        self.index = if key == Key::Up {
                            (self.index + n - 1) % n
                        } else {
                            (self.index + 1) % n
                        };
                    }
                }
                Key::SoftLeft => {
                    self.organizer.contact.return_index = self.index;
                    self.organizer.page = Page::ContactMenu;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.organizer.page = Page::Contacts;
                    self.index = self.organizer.contact.root_index;
                }
                Key::Center if self.index % 2 == 1 => {
                    if let Some((_, number)) = self.contact_numbers().get(self.index / 2).cloned() {
                        self.contact_compose(number);
                    }
                }
                _ => {} // Calling is deliberately disabled, including the physical call key.
            },
            Page::DeleteContact => match key {
                Key::Center => {
                    if self.organizer.selected < self.local_data.contacts.len() {
                        self.local_data.contacts.remove(self.organizer.selected);
                        self.local_dirty = true;
                    }
                    self.organizer.page = Page::Contacts;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.organizer.page = Page::ContactDetail;
                    self.index = 0;
                }
                _ => {}
            },
            _ => {}
        }
        true
    }
    pub fn selected_contact_name(&self) -> String {
        self.local_data
            .contacts
            .get(self.organizer.selected)
            .map(|c| contact_name(c))
            .unwrap_or_default()
    }
}
