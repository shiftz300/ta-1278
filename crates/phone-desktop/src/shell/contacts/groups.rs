//! Local group management and contact choice lists.
use super::Page;
use crate::shell::*;
impl Shell {
    pub(in crate::shell) fn contact_groups_key(&mut self, key: Key) -> bool {
        let page = self.contacts.page;
        if page == Page::ContactGroupDelete {
            match key {
                Key::Center => {
                    if let Some(i) = self
                        .contacts
                        .ui
                        .selected_group
                        .take()
                        .filter(|i| *i < self.contacts.groups.len())
                    {
                        let name = self.contacts.groups.remove(i);
                        for row in &mut self.contacts.records {
                            if row[8] == name {
                                row[8] = "无".into();
                            }
                        }
                        self.records_dirty = true;
                    }
                    self.contacts.page = Page::ContactGroups;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.contacts.page = Page::ContactGroups;
                    self.index = 0;
                }
                _ => {}
            }
            return true;
        }
        if page == Page::ContactGroupMenu {
            match key {
                Key::Up => self.index = (self.index + 2) % 3,
                Key::Down => self.index = (self.index + 1) % 3,
                Key::SoftRight => {
                    self.contacts.page = Page::ContactGroups;
                    self.index = 0;
                }
                Key::Center => match self.index {
                    0 => {
                        self.contacts.page = Page::ContactGroups;
                        self.index = 0;
                        self.open_local_text(TextTarget::ContactGroup, String::new(), false);
                    }
                    1 => {
                        if let Some(i) = self.contacts.ui.selected_group {
                            let name = self.contacts.groups[i].clone();
                            self.open_local_text(TextTarget::ContactGroupRename, name, false);
                        }
                    }
                    2 if self.contacts.ui.selected_group.is_some() => {
                        self.contacts.page = Page::ContactGroupDelete
                    }
                    _ => {}
                },
                _ => {}
            }
            return true;
        }
        if matches!(
            page,
            Page::ContactGroups | Page::ContactGroupChoice | Page::ContactRingtone
        ) {
            let labels = self.contact_choices();
            let n = labels.len();
            match key {
                Key::Up | Key::Down => {
                    vertical(&mut self.index, n, key, Boundary::Wrap);
                }
                Key::SoftLeft if page == Page::ContactGroups => {
                    self.contacts.ui.selected_group = self.index.checked_sub(1);
                    self.contacts.page = Page::ContactGroupMenu;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.contacts.page = if page == Page::ContactGroups {
                        Page::Contacts
                    } else {
                        Page::ContactForm
                    };
                    self.index = if page == Page::ContactRingtone {
                        7
                    } else if page == Page::ContactGroupChoice {
                        8
                    } else {
                        0
                    };
                }
                Key::Center if n > 0 => {
                    if page == Page::ContactGroups {
                        self.contacts.ui.group_filter = if self.index == 0 {
                            None
                        } else {
                            Some(labels[self.index].clone())
                        };
                        self.contacts.page = Page::Contacts;
                        self.index = 0;
                    } else if page == Page::ContactGroupChoice {
                        self.contacts.draft[8] = labels[self.index].clone();
                        self.contacts.page = Page::ContactForm;
                        self.index = 8;
                    } else if self.index == crate::audio::native::file_index() {
                        self.open_media(media::Purpose::ContactRingtone);
                        self.contacts.page = Page::ContactForm;
                        self.media.ui.return_index = 7;
                    } else {
                        self.contacts.draft[7] = labels[self.index].clone();
                        self.audio_preview = Some(labels[self.index].clone());
                        self.contacts.page = Page::ContactForm;
                        self.index = 7;
                    }
                }
                _ => {}
            }
            return true;
        }
        false
    }
    pub fn contact_choices(&self) -> Vec<String> {
        if self.contacts.page == Page::ContactRingtone {
            return crate::audio::native::tone_labels("预设");
        }
        std::iter::once(if self.contacts.page == Page::ContactGroups {
            "全部联系人".to_owned()
        } else {
            "无".to_owned()
        })
        .chain(self.contacts.groups.iter().cloned())
        .collect()
    }
    pub(in crate::shell) fn contact_group_name(&mut self, value: String) {
        let value = value.trim();
        if value.is_empty()
            || value.chars().count() > 128
            || value == "无"
            || value == "全部联系人"
            || self.contacts.groups.iter().any(|g| g == value)
        {
            self.notice("组名称为空或已存在。");
            return;
        }
        if self.contacts.groups.len() >= 64 {
            self.notice("通话组已满。");
            return;
        }
        self.contacts.groups.push(value.to_owned());
        self.records_dirty = true;
    }
    pub(in crate::shell) fn rename_contact_group(&mut self, value: String) {
        let value = value.trim();
        if value.is_empty()
            || value.chars().count() > 128
            || value == "无"
            || value == "全部联系人"
            || self.contacts.groups.iter().any(|s| s == value)
        {
            self.notice("组名称为空或已存在。");
            return;
        }
        if let Some(i) = self
            .contacts
            .ui
            .selected_group
            .filter(|i| *i < self.contacts.groups.len())
        {
            let old = std::mem::replace(&mut self.contacts.groups[i], value.into());
            for row in &mut self.contacts.records {
                if row[8] == old {
                    row[8] = value.into();
                }
            }
            self.records_dirty = true;
        }
        self.contacts.page = Page::ContactGroups;
        self.index = 0;
    }
}
