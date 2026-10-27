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
impl Shell {
    pub fn chat_item(&self) -> Option<Item> {
        self.messaging
            .mailbox
            .thread(&self.messaging.chat.peer)?
            .items
            .get(self.messaging.chat.focus?)
            .copied()
    }
    pub fn open_thread(&mut self, peer: String) {
        let draft = self
            .messaging
            .mailbox
            .thread(&peer)
            .and_then(|c| c.drafts.last().copied());
        self.messaging.compose = if let Some(i) = draft {
            Compose::from_message(&self.messaging.mailbox.drafts[i], Some(i))
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
        self.messaging.chat = ChatUi {
            peer,
            route: true,
            ..Default::default()
        };
        self.messaging.dirty |= self
            .messaging
            .mailbox
            .read_thread(&self.messaging.chat.peer);
        self.go(Screen::Conversation);
    }
    pub(super) fn messages_key(&mut self, key: Key) {
        if self.messaging.chat.popup == Popup::Root {
            match key {
                Key::Up | Key::Down => self.messaging.chat.choice = 1 - self.messaging.chat.choice,
                Key::SoftRight | Key::End => self.messaging.chat.popup = Popup::None,
                Key::Center => {
                    let folder = if self.messaging.chat.choice == 0 {
                        Screen::Drafts
                    } else {
                        Screen::Outbox
                    };
                    self.messaging.chat.popup = Popup::None;
                    self.go(folder);
                }
                _ => {}
            }
            return;
        }
        let threads = self.messaging.mailbox.conversations();
        match key {
            Key::Up => self.index = self.index.saturating_sub(1),
            Key::Down => self.index = (self.index + 1).min(threads.len()),
            Key::SoftLeft => {
                self.messaging.chat.popup = Popup::Root;
                self.messaging.chat.choice = 0;
            }
            Key::Center if self.index == 0 => {
                self.messaging.compose = self.new_compose();
                self.messaging.chat = ChatUi {
                    route: true,
                    ..Default::default()
                };
                self.go(Screen::Compose);
            }
            Key::Center => {
                if let Some(c) = threads.get(self.index - 1) {
                    if c.peer.is_empty() {
                        if let Some(&i) = c.drafts.last() {
                            self.messaging.compose =
                                Compose::from_message(&self.messaging.mailbox.drafts[i], Some(i));
                            self.messaging.chat.route = true;
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
        if self.messaging.chat.popup != Popup::None {
            self.chat_popup_key(key);
            return true;
        }
        if let Some(focus) = self.messaging.chat.focus {
            let items = self
                .messaging
                .mailbox
                .thread(&self.messaging.chat.peer)
                .map(|c| c.items)
                .unwrap_or_default();
            if matches!(key, Key::Up | Key::Down) {
                self.messaging.chat.reveal = true;
            }
            match key {
                Key::Left => self.messaging.chat.scroll = (self.messaging.chat.scroll - 48).max(0),
                Key::Right => {
                    self.messaging.chat.scroll =
                        (self.messaging.chat.scroll + 48).min(self.messaging.chat.max_scroll)
                }
                Key::Up => {
                    let has_number = items
                        .get(focus)
                        .and_then(|&i| self.messaging.mailbox.content(i))
                        .and_then(|m| number(m.body))
                        .is_some();
                    if !self.messaging.chat.number_focus && has_number {
                        self.messaging.chat.number_focus = true;
                    } else {
                        self.messaging.chat.focus = Some(focus.saturating_sub(1));
                        self.messaging.chat.number_focus = false;
                    }
                }
                Key::Down => {
                    if self.messaging.chat.number_focus {
                        self.messaging.chat.number_focus = false;
                    } else if focus + 1 < items.len() {
                        self.messaging.chat.focus = Some(focus + 1);
                    } else {
                        self.messaging.chat.focus = None;
                    }
                }
                Key::SoftRight => {
                    self.messaging.chat.focus = None;
                    self.messaging.chat.number_focus = false;
                }
                Key::Center if self.messaging.chat.number_focus => self.save_chat_number(),
                Key::Center => self.messaging.chat.focus = None,
                Key::SoftLeft => {
                    self.messaging.chat.popup = if self.messaging.chat.number_focus {
                        Popup::Number
                    } else {
                        Popup::Bubble
                    };
                    self.messaging.chat.choice = 0;
                }
                Key::Delete => self.messaging.chat.popup = Popup::Delete,
                Key::Digit(_) | Key::Hash | Key::Star => {
                    self.messaging.chat.focus = None;
                    return false;
                }
                _ => {}
            }
            return true;
        }
        if key == Key::Up
            && self.editor_lines.first().is_some_and(|r| {
                r.stops
                    .iter()
                    .any(|(i, _)| *i == self.messaging.compose.body.cursor)
            })
        {
            let count = self
                .messaging
                .mailbox
                .thread(&self.messaging.chat.peer)
                .map_or(0, |c| c.items.len());
            if count > 0 {
                self.messaging.compose.body.commit();
                self.messaging.chat.focus = Some(count - 1);
                self.messaging.chat.number_focus = false;
                self.messaging.chat.reveal = true;
                return true;
            }
        }
        false
    }
    fn save_chat_number(&mut self) {
        let n = self
            .chat_item()
            .and_then(|i| self.messaging.mailbox.content(i))
            .and_then(|c| number(c.body));
        if let Some(n) = n {
            // Keep the unfinished reply durable before leaving its editor.
            if !self.messaging.compose.body.units.is_empty() {
                self.save_draft();
            }
            self.messaging.chat.contact_return = true;
            self.contact_form(None);
            self.contacts.draft[3] = n;
            self.go(Screen::Contacts);
        }
    }
    fn chat_popup_key(&mut self, key: Key) {
        let popup = self.messaging.chat.popup;
        match key {
            Key::SoftRight | Key::End => self.messaging.chat.popup = Popup::None,
            Key::Up | Key::Down if popup != Popup::Delete => {
                self.messaging.chat.choice = 1 - self.messaging.chat.choice
            }
            Key::Center => {
                let Some(item) = self.chat_item() else {
                    self.messaging.chat.popup = Popup::None;
                    return;
                };
                match popup {
                    Popup::Delete => {
                        self.messaging.mailbox.delete_item(item);
                        self.messaging.dirty = true;
                        self.messaging.chat.focus = None;
                        self.messaging.chat.number_focus = false;
                        self.messaging.chat.popup = Popup::None;
                    }
                    Popup::Bubble if self.messaging.chat.choice == 0 => {
                        self.messaging.chat.popup = Popup::Delete
                    }
                    Popup::Bubble => {
                        let body = self
                            .messaging
                            .mailbox
                            .content(item)
                            .unwrap()
                            .body
                            .to_owned();
                        if !self.messaging.compose.body.units.is_empty() {
                            self.save_draft();
                        }
                        self.messaging.compose = self.new_compose();
                        self.messaging.compose.body.insert(&body);
                        self.messaging.chat.popup = Popup::None;
                        self.go(Screen::Compose);
                    }
                    Popup::Number if self.messaging.chat.choice == 1 => {
                        let n = self
                            .messaging
                            .mailbox
                            .content(item)
                            .and_then(|m| number(m.body));
                        if let Some(n) = n {
                            if !self.messaging.compose.body.units.is_empty() {
                                self.save_draft();
                            }
                            self.open_thread(n);
                        }
                    }
                    Popup::Number => {
                        let number = self
                            .messaging
                            .mailbox
                            .content(item)
                            .and_then(|m| number(m.body));
                        if let Some(number) = number {
                            self.messaging.chat.popup = Popup::None;
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
