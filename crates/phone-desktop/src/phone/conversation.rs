//! Conversation navigation over the existing mailbox and transactional composer.
#[cfg(test)]
mod tests;
use super::*;
use phone_core::sms::conversation::Item;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Popup {
    #[default]
    None,
    Root,
    Bubble,
    Number,
    Delete,
}
#[derive(Default)]
pub struct ChatUi {
    pub peer: String,
    pub focus: Option<usize>,
    pub number_focus: bool,
    pub popup: Popup,
    pub choice: usize,
    pub scroll: i32,
    pub max_scroll: i32,
    pub route: bool,
    pub reveal: bool,
    pub contact_return: bool,
}
/// Bounded local number detection; not the original firmware's full link parser.
pub fn number(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_digit() && c != '+')
        .find(|s| {
            (3..=40).contains(&s.len())
                && s.chars()
                    .enumerate()
                    .all(|(i, c)| c.is_ascii_digit() || (i == 0 && c == '+'))
        })
        .map(str::to_owned)
}
impl Phone {
    pub fn chat_item(&self) -> Option<Item> {
        self.mailbox
            .thread(&self.chat.peer)?
            .items
            .get(self.chat.focus?)
            .copied()
    }
    pub fn open_thread(&mut self, peer: String) {
        let draft = self
            .mailbox
            .thread(&peer)
            .and_then(|c| c.drafts.last().copied());
        self.compose = if let Some(i) = draft {
            Compose::from_message(&self.mailbox.drafts[i], Some(i))
        } else {
            let mut c = Compose::from_message(
                &Message {
                    to: peer.clone(),
                    ..Default::default()
                },
                None,
            );
            c.time = self.timestamp();
            c
        };
        self.chat = ChatUi {
            peer,
            route: true,
            ..Default::default()
        };
        self.dirty |= self.mailbox.read_thread(&self.chat.peer);
        self.go(Screen::Conversation);
    }
    pub(super) fn messages_key(&mut self, key: Key) {
        if self.chat.popup == Popup::Root {
            match key {
                Key::Up | Key::Down => self.chat.choice = 1 - self.chat.choice,
                Key::SoftRight | Key::End => self.chat.popup = Popup::None,
                Key::Center => {
                    let folder = if self.chat.choice == 0 {
                        Screen::Drafts
                    } else {
                        Screen::Outbox
                    };
                    self.chat.popup = Popup::None;
                    self.go(folder);
                }
                _ => {}
            }
            return;
        }
        let threads = self.mailbox.conversations();
        match key {
            Key::Up => self.index = self.index.saturating_sub(1),
            Key::Down => self.index = (self.index + 1).min(threads.len()),
            Key::SoftLeft => {
                self.chat.popup = Popup::Root;
                self.chat.choice = 0;
            }
            Key::Center if self.index == 0 => {
                self.compose = self.new_compose();
                self.chat = ChatUi {
                    route: true,
                    ..Default::default()
                };
                self.go(Screen::Compose);
            }
            Key::Center => {
                if let Some(c) = threads.get(self.index - 1) {
                    if c.peer.is_empty() {
                        if let Some(&i) = c.drafts.last() {
                            self.compose = Compose::from_message(&self.mailbox.drafts[i], Some(i));
                            self.chat.route = true;
                            self.go(Screen::Compose);
                        }
                    } else {
                        self.open_thread(c.peer.clone());
                    }
                }
            }
            Key::SoftRight => self.leave_app(),
            _ => {}
        }
    }
    pub(super) fn chat_event(&mut self, key: Key) -> bool {
        if self.chat.popup != Popup::None {
            self.chat_popup_key(key);
            return true;
        }
        if let Some(focus) = self.chat.focus {
            let items = self
                .mailbox
                .thread(&self.chat.peer)
                .map(|c| c.items)
                .unwrap_or_default();
            if matches!(key, Key::Up | Key::Down) {
                self.chat.reveal = true;
            }
            match key {
                Key::Left => self.chat.scroll = (self.chat.scroll - 48).max(0),
                Key::Right => self.chat.scroll = (self.chat.scroll + 48).min(self.chat.max_scroll),
                Key::Up => {
                    let has_number = items
                        .get(focus)
                        .and_then(|&i| self.mailbox.content(i))
                        .and_then(|m| number(m.body))
                        .is_some();
                    if !self.chat.number_focus && has_number {
                        self.chat.number_focus = true;
                    } else {
                        self.chat.focus = Some(focus.saturating_sub(1));
                        self.chat.number_focus = false;
                    }
                }
                Key::Down => {
                    if self.chat.number_focus {
                        self.chat.number_focus = false;
                    } else if focus + 1 < items.len() {
                        self.chat.focus = Some(focus + 1);
                    } else {
                        self.chat.focus = None;
                    }
                }
                Key::SoftRight => {
                    self.chat.focus = None;
                    self.chat.number_focus = false;
                }
                Key::Center if self.chat.number_focus => self.save_chat_number(),
                Key::Center => self.chat.focus = None,
                Key::SoftLeft => {
                    self.chat.popup = if self.chat.number_focus {
                        Popup::Number
                    } else {
                        Popup::Bubble
                    };
                    self.chat.choice = 0;
                }
                Key::Delete => self.chat.popup = Popup::Delete,
                Key::Digit(_) | Key::Hash | Key::Star => {
                    self.chat.focus = None;
                    return false;
                }
                _ => {}
            }
            return true;
        }
        if key == Key::Up
            && self
                .editor_lines
                .first()
                .is_some_and(|r| r.stops.iter().any(|(i, _)| *i == self.compose.body.cursor))
        {
            let count = self
                .mailbox
                .thread(&self.chat.peer)
                .map_or(0, |c| c.items.len());
            if count > 0 {
                self.compose.body.commit();
                self.chat.focus = Some(count - 1);
                self.chat.number_focus = false;
                self.chat.reveal = true;
                return true;
            }
        }
        false
    }
    fn save_chat_number(&mut self) {
        let n = self
            .chat_item()
            .and_then(|i| self.mailbox.content(i))
            .and_then(|c| number(c.body));
        if let Some(n) = n {
            // Keep the unfinished reply durable before leaving its editor.
            if !self.compose.body.units.is_empty() {
                self.save_draft();
            }
            self.chat.contact_return = true;
            self.organizer_form(organizer::OrganizerPage::ContactForm, None);
            self.organizer.draft[3] = n;
            self.go(Screen::Organizer(organizer::OrganizerPage::ContactForm));
        }
    }
    fn chat_popup_key(&mut self, key: Key) {
        let popup = self.chat.popup;
        match key {
            Key::SoftRight | Key::End => self.chat.popup = Popup::None,
            Key::Up | Key::Down if popup != Popup::Delete => {
                self.chat.choice = 1 - self.chat.choice
            }
            Key::Center => {
                let Some(item) = self.chat_item() else {
                    self.chat.popup = Popup::None;
                    return;
                };
                match popup {
                    Popup::Delete => {
                        self.mailbox.delete_item(item);
                        self.dirty = true;
                        self.chat.focus = None;
                        self.chat.number_focus = false;
                        self.chat.popup = Popup::None;
                    }
                    Popup::Bubble if self.chat.choice == 0 => self.chat.popup = Popup::Delete,
                    Popup::Bubble => {
                        let body = self.mailbox.content(item).unwrap().body.to_owned();
                        if !self.compose.body.units.is_empty() {
                            self.save_draft();
                        }
                        self.compose = self.new_compose();
                        self.compose.body.insert(&body);
                        self.chat.popup = Popup::None;
                        self.go(Screen::Compose);
                    }
                    Popup::Number if self.chat.choice == 1 => {
                        let n = self.mailbox.content(item).and_then(|m| number(m.body));
                        if let Some(n) = n {
                            if !self.compose.body.units.is_empty() {
                                self.save_draft();
                            }
                            self.open_thread(n);
                        }
                    }
                    Popup::Number => {
                        let number = self.mailbox.content(item).and_then(|m| number(m.body));
                        if let Some(number) = number {
                            self.chat.popup = Popup::None;
                            self.open_dialer(number);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
