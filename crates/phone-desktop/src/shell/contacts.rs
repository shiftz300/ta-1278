//! Local phonebook routes; the nine-field persisted contact schema is unchanged.
mod groups;
use super::*;
use crate::ui::menu::Item;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Contacts,
    ContactsMenu,
    ContactFormMenu,
    ContactForm,
    ContactDetail,
    ContactMenu,
    DeleteContact,
    ContactGroups,
    ContactGroupChoice,
    ContactRingtone,
    ContactGroupMenu,
    ContactGroupDelete,
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
#[derive(Default)]
pub struct State {
    pub page: Page,
    pub draft: Vec<String>,
    pub editing: Option<usize>,
    pub selected: usize,
    pub pick_recipient: bool,
    pub ui: ContactUi,
    pub records: Vec<Vec<String>>,
    pub groups: Vec<String>,
}
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
#[derive(Clone, Copy)]
pub enum Action {
    Groups,
    New,
    Edit,
    Delete,
    Share,
    SpeedDial,
    Settings,
}
pub const ROOT_MENU: [Item<'static, Action>; 6] = [
    Item::new(Action::Groups, "通话组"),
    Item::new(Action::New, "增加新联系人"),
    Item::new(Action::Delete, "删除联系人"),
    Item::new(Action::Share, "分享联系人"),
    Item::new(Action::SpeedDial, "单键拨号"),
    Item::new(Action::Settings, "设置"),
];
pub const DETAIL_MENU: [Item<'static, Action>; 4] = [
    Item::new(Action::Edit, "编辑联系人"),
    Item::new(Action::Delete, "删除联系人"),
    Item::new(Action::Share, "分享联系人"),
    Item::new(Action::SpeedDial, "单键拨号"),
];
impl Shell {
    pub fn contact_form(&mut self, editing: Option<usize>) {
        self.contacts.open(editing);
        self.index = 1;
    }
    pub fn contact_load_field(&mut self, field: usize) {
        self.contacts.load_field(field);
        self.index = field;
    }
    fn contact_save(&mut self) {
        match self.contacts.save() {
            Ok(true) => {
                self.records_dirty = true;
                self.contact_return();
            }
            Ok(false) => {}
            Err(error) => self.notice(error),
        }
    }

    pub fn contact_title(&self) -> &'static str {
        match self.contacts.page {
            Page::Contacts => "通讯录",
            Page::ContactForm => "新联系人",
            Page::ContactDetail => "详情",
            Page::ContactGroups | Page::ContactGroupChoice => "通话组",
            Page::ContactRingtone => "铃声",
            Page::DeleteContact | Page::ContactGroupDelete => "确认",
            _ => "选项",
        }
    }
    pub fn contact_editor_active(&self) -> bool {
        self.screen == Screen::Contacts
            && (self.contacts.page == Page::Contacts
                || (self.contacts.page == Page::ContactForm && (1..=5).contains(&self.index)))
    }
    pub fn contact_ids(&self) -> Vec<usize> {
        let query = self.contacts.ui.query.text().to_lowercase();
        self.contacts
            .records
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                (self
                    .contacts
                    .ui
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
        self.contacts
            .records
            .get(self.contacts.selected)
            .map(|c| {
                (3..=5)
                    .filter_map(|i| c.get(i).filter(|s| !s.is_empty()).map(|s| (i, s.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn contact_menu_item(&self, index: usize) -> Item<'static, Action> {
        let root = self.contacts.page == Page::ContactsMenu;
        let item = if root {
            &ROOT_MENU[index]
        } else {
            &DETAIL_MENU[index]
        };
        let enabled = match item.action {
            Action::Groups | Action::New | Action::Edit => true,
            Action::Delete => !root || !self.contact_ids().is_empty(),
            Action::Share | Action::SpeedDial | Action::Settings => false,
        };
        Item {
            action: item.action,
            label: item.label,
            enabled,
            marker: item.marker,
        }
    }
    fn contact_return(&mut self) {
        if self.messaging.chat.contact_return {
            self.messaging.chat.contact_return = false;
            self.go(Screen::Conversation);
        } else {
            self.contacts.page = if self.contacts.editing.is_some() {
                Page::ContactDetail
            } else {
                Page::Contacts
            };
            self.index = if self.contacts.editing.is_some() {
                0
            } else {
                self.contacts.ui.root_index
            };
        }
    }
    fn contact_compose(&mut self, number: String) {
        if !self.contacts.pick_recipient {
            self.messaging.compose = self.new_compose();
            self.messaging.chat = Default::default();
        }
        self.messaging.compose.to = Editor::new(&number, 40);
        self.messaging.compose.to.numeric = true;
        self.messaging.compose.recipient = false;
        self.contacts.pick_recipient = false;
        self.go(Screen::Compose);
    }
    pub(super) fn contacts_key(&mut self, key: Key) -> bool {
        let page = self.contacts.page;
        if page == Page::Contacts && key == Key::Call && !self.contacts.pick_recipient {
            if let Some(id) = self.contact_ids().get(self.index).copied() {
                self.contacts.selected = id;
                if let Some((_, number)) = self.contact_numbers().first().cloned() {
                    self.open_dialer(number);
                }
            }
            return true;
        }
        if self.contact_groups_key(key) {
            return true;
        }
        if page == Page::ContactForm && self.index == 6 {
            match key {
                Key::Left => {
                    self.contacts.ui.date_digit = self.contacts.ui.date_digit.saturating_sub(1);
                    return true;
                }
                Key::Right => {
                    self.contacts.ui.date_digit = (self.contacts.ui.date_digit + 1).min(7);
                    return true;
                }
                Key::Digit(d) if d < 10 => {
                    let value = &self.contacts.draft[6];
                    let mut bytes = if phone_core::calendar::Date::parse(value).is_some() {
                        value.as_bytes().to_vec()
                    } else {
                        b"1900-01-01".to_vec()
                    };
                    bytes[phone_core::calendar::Date::DIGIT_OFFSETS[self.contacts.ui.date_digit]] =
                        b'0' + d;
                    let next = String::from_utf8(bytes).unwrap();
                    if phone_core::calendar::Date::parse(&next).is_some() {
                        self.contacts.draft[6] = next;
                        self.contacts.ui.date_digit = (self.contacts.ui.date_digit + 1).min(7);
                    }
                    return true;
                }
                _ => {}
            }
        }
        match page {
            Page::ContactForm => match key {
                Key::Delete if matches!(self.index, 0 | 7 | 8) => {
                    self.contacts.draft[self.index] = match self.index {
                        0 => String::new(),
                        _ => "无".into(),
                    }
                }
                Key::Center if matches!(self.index, 0 | 7 | 8) => {
                    if !self.contacts.sync() {
                        return true;
                    }
                    match self.index {
                        0 => self.open_media(media::Purpose::ContactPicture),
                        7 => {
                            self.contacts.page = Page::ContactRingtone;
                            self.index = 0;
                        }
                        _ => {
                            self.contacts.page = Page::ContactGroupChoice;
                            self.index = 0;
                        }
                    }
                }
                Key::Up | Key::Down => {
                    if self.contacts.sync() {
                        let next = if key == Key::Up {
                            (self.index + 8) % 9
                        } else {
                            (self.index + 1) % 9
                        };
                        self.contact_load_field(next);
                    }
                }
                Key::SoftLeft => {
                    if self.contacts.sync() {
                        self.contacts.ui.return_index = self.index;
                        self.contacts.page = Page::ContactFormMenu;
                        self.index = 0;
                    }
                }
                Key::SoftRight => {
                    if (1..=5).contains(&self.index)
                        && (!self.contacts.ui.input.units.is_empty()
                            || self.contacts.ui.input.prediction.active())
                    {
                        self.contacts.ui.input.backspace();
                    } else {
                        self.contact_return();
                    }
                }
                _ => {}
            },
            Page::ContactFormMenu => match key {
                Key::Center => self.contact_save(),
                Key::SoftRight => {
                    self.contacts.page = Page::ContactForm;
                    self.index = self.contacts.ui.return_index;
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
                        self.contacts.ui.return_index = self.index;
                        self.contacts.ui.root_index = self.index;
                        self.contacts.page = Page::ContactsMenu;
                        self.index = 0;
                    }
                    Key::Center if !ids.is_empty() => {
                        self.contacts.ui.root_index = self.index;
                        self.contacts.selected = ids[self.index.min(ids.len() - 1)];
                        if self.contacts.pick_recipient {
                            if let Some((_, n)) = self.contact_numbers().first().cloned() {
                                self.contact_compose(n);
                            }
                        } else {
                            self.contacts.page = Page::ContactDetail;
                            self.index = 0;
                        }
                    }
                    Key::SoftRight => {
                        if self.contacts.pick_recipient {
                            self.contacts.pick_recipient = false;
                            self.go(Screen::Compose);
                        } else if self.contacts.ui.group_filter.take().is_some() {
                            self.contacts.page = Page::ContactGroups;
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
                        self.contacts.page = if page == Page::ContactsMenu {
                            Page::Contacts
                        } else {
                            Page::ContactDetail
                        };
                        self.index = self.contacts.ui.return_index;
                    }
                    Key::Center if self.contact_menu_item(self.index).enabled => {
                        match self.contact_menu_item(self.index).action {
                            Action::Groups => {
                                self.contacts.page = Page::ContactGroups;
                                self.index = 0;
                            }
                            Action::New => self.contact_form(None),
                            Action::Edit => self.contact_form(Some(self.contacts.selected)),
                            Action::Delete => {
                                if page == Page::ContactsMenu {
                                    let ids = self.contact_ids();
                                    self.contacts.selected =
                                        ids[self.contacts.ui.return_index.min(ids.len() - 1)];
                                }
                                self.contacts.page = Page::DeleteContact;
                                self.index = 0;
                            }
                            Action::Share | Action::SpeedDial | Action::Settings => {}
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
                    self.contacts.ui.return_index = self.index;
                    self.contacts.page = Page::ContactMenu;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.contacts.page = Page::Contacts;
                    self.index = self.contacts.ui.root_index;
                }
                Key::Center if self.index % 2 == 1 => {
                    if let Some((_, number)) = self.contact_numbers().get(self.index / 2).cloned() {
                        self.contact_compose(number);
                    }
                }
                Key::Center | Key::Call => {
                    if let Some((_, number)) = self.contact_numbers().get(self.index / 2).cloned() {
                        self.open_dialer(number);
                    }
                }
                _ => {}
            },
            Page::DeleteContact => match key {
                Key::Center => {
                    if self.contacts.selected < self.contacts.records.len() {
                        self.contacts.records.remove(self.contacts.selected);
                        self.records_dirty = true;
                    }
                    self.contacts.page = Page::Contacts;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.contacts.page = Page::ContactDetail;
                    self.index = 0;
                }
                _ => {}
            },
            _ => {}
        }
        true
    }
    pub fn selected_contact_name(&self) -> String {
        self.contacts
            .records
            .get(self.contacts.selected)
            .map(|c| contact_name(c))
            .unwrap_or_default()
    }
}

impl State {
    fn open(&mut self, editing: Option<usize>) {
        self.editing = editing;
        self.page = Page::ContactForm;
        self.draft = editing
            .and_then(|i| self.records.get(i).cloned())
            .unwrap_or_else(|| vec![String::new(); CONTACT_FIELDS.len()]);
        if editing.is_none() {
            self.draft[6] = "1900-01-01".into();
            self.draft[7] = "预设".into();
            self.draft[8] = "无".into();
        }
        self.load_field(1);
    }

    pub fn load_field(&mut self, field: usize) {
        self.ui.field = field;
        let value = self.draft.get(field).cloned().unwrap_or_default();
        let mut input = Editor::new(&value, 40);
        input.numeric = (3..=6).contains(&field);
        if field == 6 {
            self.ui.date_digit = 0;
        }
        if matches!(field, 1 | 2) {
            input
                .prediction
                .set_language(Some(phone_core::predict::Language::Chinese));
        }
        self.ui.input = input;
    }
    fn sync(&mut self) -> bool {
        if !(1..=5).contains(&self.ui.field) {
            return true;
        }
        if !self.ui.input.accept_prediction() {
            return false;
        }
        self.ui.input.commit();
        let field = self.ui.field;
        if let Some(value) = self.draft.get_mut(field) {
            *value = self.ui.input.text();
        }
        true
    }
    fn save(&mut self) -> Result<bool, &'static str> {
        if !self.sync() {
            return Ok(false);
        }
        if self
            .draft
            .iter()
            .skip(1)
            .take(5)
            .all(|s| s.trim().is_empty())
        {
            return Err("请输入姓名或号码。");
        }
        let date = &self.draft[6];
        if !date.is_empty() && phone_core::calendar::Date::parse(date).is_none() {
            return Err("日期格式：YYYY-MM-DD");
        }
        let fields = self.draft.clone();
        if let Some(i) = self.editing {
            self.records[i] = fields;
        } else {
            self.records.push(fields);
        }
        self.editing = None;
        Ok(true)
    }
}
