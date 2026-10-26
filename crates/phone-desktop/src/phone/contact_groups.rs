//! Local group management and contact choice lists.
use super::*;
use organizer::OrganizerPage as Page;
impl Phone {
    pub(super) fn contact_groups_key(&mut self, key: Key) -> bool {
        let page = self.organizer.page;
        if page == Page::ContactGroupDelete {
            match key {
                Key::Center => {
                    if let Some(i) = self
                        .organizer
                        .contact
                        .selected_group
                        .take()
                        .filter(|i| *i < self.local_data.groups.len())
                    {
                        let name = self.local_data.groups.remove(i);
                        for row in &mut self.local_data.contacts {
                            if row[8] == name {
                                row[8] = "无".into();
                            }
                        }
                        self.local_dirty = true;
                    }
                    self.organizer.page = Page::ContactGroups;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.organizer.page = Page::ContactGroups;
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
                    self.organizer.page = Page::ContactGroups;
                    self.index = 0;
                }
                Key::Center => match self.index {
                    0 => {
                        self.organizer.page = Page::ContactGroups;
                        self.index = 0;
                        self.open_local_text(TextTarget::ContactGroup, String::new(), false);
                    }
                    1 => {
                        if let Some(i) = self.organizer.contact.selected_group {
                            let name = self.local_data.groups[i].clone();
                            self.open_local_text(TextTarget::ContactGroupRename, name, false);
                        }
                    }
                    2 if self.organizer.contact.selected_group.is_some() => {
                        self.organizer.page = Page::ContactGroupDelete
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
                Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
                Key::Down if n > 0 => self.index = (self.index + 1) % n,
                Key::SoftLeft if page == Page::ContactGroups => {
                    self.organizer.contact.selected_group = self.index.checked_sub(1);
                    self.organizer.page = Page::ContactGroupMenu;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.organizer.page = if page == Page::ContactGroups {
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
                        self.organizer.contact.group_filter = if self.index == 0 {
                            None
                        } else {
                            Some(labels[self.index].clone())
                        };
                        self.organizer.page = Page::Contacts;
                        self.index = 0;
                    } else if page == Page::ContactGroupChoice {
                        self.organizer.draft[8] = labels[self.index].clone();
                        self.organizer.page = Page::ContactForm;
                        self.index = 8;
                    } else if self.index == crate::native_audio::file_index() {
                        self.open_media(media_browser::Purpose::ContactRingtone);
                        self.organizer.page = Page::ContactForm;
                        self.media_ui.return_index = 7;
                    } else {
                        self.organizer.draft[7] = labels[self.index].clone();
                        self.audio_preview = Some(labels[self.index].clone());
                        self.organizer.page = Page::ContactForm;
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
        if self.organizer.page == Page::ContactRingtone {
            return crate::native_audio::tone_labels("预设");
        }
        std::iter::once(if self.organizer.page == Page::ContactGroups {
            "全部联系人".to_owned()
        } else {
            "无".to_owned()
        })
        .chain(self.local_data.groups.iter().cloned())
        .collect()
    }
    pub(super) fn contact_group_name(&mut self, value: String) {
        let value = value.trim();
        if value.is_empty()
            || value.chars().count() > 128
            || value == "无"
            || value == "全部联系人"
            || self.local_data.groups.iter().any(|g| g == value)
        {
            self.notice("组名称为空或已存在。");
            return;
        }
        if self.local_data.groups.len() >= 64 {
            self.notice("通话组已满。");
            return;
        }
        self.local_data.groups.push(value.to_owned());
        self.local_dirty = true;
    }
    pub(super) fn rename_contact_group(&mut self, value: String) {
        let value = value.trim();
        if value.is_empty()
            || value.chars().count() > 128
            || value == "无"
            || value == "全部联系人"
            || self.local_data.groups.iter().any(|s| s == value)
        {
            self.notice("组名称为空或已存在。");
            return;
        }
        if let Some(i) = self
            .organizer
            .contact
            .selected_group
            .filter(|i| *i < self.local_data.groups.len())
        {
            let old = std::mem::replace(&mut self.local_data.groups[i], value.into());
            for row in &mut self.local_data.contacts {
                if row[8] == old {
                    row[8] = value.into();
                }
            }
            self.local_dirty = true;
        }
        self.organizer.page = Page::ContactGroups;
        self.index = 0;
    }
}
