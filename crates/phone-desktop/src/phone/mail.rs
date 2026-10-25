//! Local mailbox reader/actions. Layout and folder policy are not recovered conversations.
use super::*;
use std::{collections::BTreeMap, sync::OnceLock};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MailPage {
    Read,
    Options,
    Delete,
    Result,
}
pub struct PendingQueue {
    pub change: phone_core::sms::QueueChange,
    pub was_dirty: bool,
}
pub struct MailUi {
    pub item: usize,
    pub folder: Screen,
    pub delete_return: Screen,
    pub scroll: usize,
    pub lines: usize,
    pub pending: Option<PendingQueue>,
}
impl Default for MailUi {
    fn default() -> Self {
        Self {
            item: 0,
            folder: Screen::Outbox,
            delete_return: Screen::Outbox,
            scroll: 0,
            lines: 0,
            pending: None,
        }
    }
}
pub fn operation_label(code: u8) -> Option<&'static str> {
    static LABELS: OnceLock<BTreeMap<u8, String>> = OnceLock::new();
    LABELS
        .get_or_init(|| {
            serde_json::from_str(include_str!(
                "../../../../assets/ui/sms-operation-labels.json"
            ))
            .expect("verified SMS labels")
        })
        .get(&code)
        .map(String::as_str)
}
#[derive(Clone, Copy)]
pub struct MailContent<'a> {
    pub peer: &'a str,
    pub body: &'a str,
    pub unread: bool,
}
#[derive(Clone, Copy)]
pub enum MailAction {
    Retry,
    Edit,
    Reply,
    Forward,
    Delete,
}
impl MailAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Retry => "重新发送",
            Self::Edit => "编辑为新信息",
            Self::Reply => "回复",
            Self::Forward => "转发",
            Self::Delete => "删除",
        }
    }
}
impl Phone {
    pub fn receive_message(&mut self, from: String, body: String) -> Result<usize, &'static str> {
        if self.mail_ui.pending.is_some() {
            return Err("Mailbox save in progress; retry after completion");
        }
        let selected = if self.screen == Screen::Messages && self.index > 0 {
            self.mailbox
                .conversations()
                .get(self.index - 1)
                .map(|c| c.peer.clone())
        } else {
            None
        };
        let index = self.mailbox.receive(from, body)?;
        if let Some(peer) = selected
            && let Some(i) = self
                .mailbox
                .conversations()
                .iter()
                .position(|c| c.peer == peer)
        {
            self.index = i + 1;
        }
        self.dirty = true;
        if self.screen == Screen::Conversation
            && self.chat.peer.trim() == self.mailbox.inbox[index].from.trim()
            && self.overlay.is_none()
            && self.chat.popup == conversation::Popup::None
            && self.chat.focus.is_none()
        {
            self.mailbox.inbox[index].read = true;
        }
        Ok(index)
    }
    pub fn mail_count(&self, folder: Screen) -> usize {
        match folder {
            Screen::Inbox => self.mailbox.inbox.len(),
            Screen::Drafts => self.mailbox.drafts.len(),
            _ => self.mailbox.outbox.len(),
        }
    }
    pub fn mail_content(&self, folder: Screen, index: usize) -> Option<MailContent<'_>> {
        if folder == Screen::Inbox {
            self.mailbox.inbox.get(index).map(|m| MailContent {
                peer: &m.from,
                body: &m.body,
                unread: !m.read,
            })
        } else {
            let messages = if folder == Screen::Drafts {
                &self.mailbox.drafts
            } else {
                &self.mailbox.outbox
            };
            messages.get(index).map(|m| MailContent {
                peer: &m.to,
                body: &m.body,
                unread: false,
            })
        }
    }
    pub fn mail_message(&self) -> Option<MailContent<'_>> {
        self.mail_content(self.mail_ui.folder, self.mail_ui.item)
    }
    pub fn mail_actions(&self) -> &'static [MailAction] {
        if self.mail_ui.folder == Screen::Inbox {
            &[MailAction::Reply, MailAction::Forward, MailAction::Delete]
        } else {
            &[
                MailAction::Retry,
                MailAction::Edit,
                MailAction::Forward,
                MailAction::Delete,
            ]
        }
    }
    pub(super) fn mail_return_folder(&mut self) {
        let count = self.mail_count(self.mail_ui.folder);
        self.go(self.mail_ui.folder);
        self.index = self.mail_ui.item.min(count.saturating_sub(1));
    }
    pub(super) fn mail_delete_prompt(&mut self) {
        self.mail_ui.delete_return = self.screen;
        self.go(Screen::Mail(MailPage::Delete));
    }
    pub(super) fn mail_open(&mut self, page: MailPage) {
        self.mail_ui.item = self.index;
        self.mail_ui.folder = self.screen;
        self.mail_ui.scroll = 0;
        self.mail_ui.lines = 0;
        if self.screen == Screen::Inbox
            && page == MailPage::Read
            && let Some(message) = self.mailbox.inbox.get_mut(self.index)
            && !message.read
        {
            message.read = true;
            self.dirty = true;
        }
        self.go(Screen::Mail(page));
    }
    pub(super) fn mail_key(&mut self, page: MailPage, key: Key) {
        if self.mail_message().is_none() {
            self.mail_return_folder();
            return;
        }
        use MailPage::*;
        match page {
            Read => match key {
                Key::Up => self.mail_ui.scroll = self.mail_ui.scroll.saturating_sub(1),
                Key::Down => {
                    self.mail_ui.scroll =
                        (self.mail_ui.scroll + 1).min(self.mail_ui.lines.saturating_sub(7))
                }
                Key::SoftLeft | Key::Center => self.go(Screen::Mail(Options)),
                Key::SoftRight => self.mail_return_folder(),
                Key::Delete => self.mail_delete_prompt(),
                _ => {}
            },
            Options => match key {
                Key::Up => {
                    self.index =
                        (self.index + self.mail_actions().len() - 1) % self.mail_actions().len()
                }
                Key::Down => self.index = (self.index + 1) % self.mail_actions().len(),
                Key::SoftRight => self.go(Screen::Mail(Read)),
                Key::Center => match self.mail_actions()[self.index] {
                    MailAction::Retry => self.go(Screen::Mail(Result)),
                    action @ (MailAction::Edit | MailAction::Reply | MailAction::Forward) => {
                        let content = self.mail_message().unwrap();
                        let message = Message {
                            to: if matches!(action, MailAction::Forward) {
                                String::new()
                            } else {
                                content.peer.into()
                            },
                            body: if matches!(action, MailAction::Reply) {
                                String::new()
                            } else {
                                content.body.into()
                            },
                            ..Default::default()
                        };
                        self.compose = Compose::from_message(&message, None);
                        self.compose.recipient = matches!(action, MailAction::Forward);
                        self.go(Screen::Compose);
                    }
                    MailAction::Delete => self.mail_delete_prompt(),
                },
                _ => {}
            },
            Delete => match key {
                Key::SoftRight => {
                    let back = self.mail_ui.delete_return;
                    if matches!(back, Screen::Drafts | Screen::Outbox | Screen::Inbox) {
                        self.mail_return_folder();
                    } else {
                        self.go(back);
                        if back == Screen::Mail(Options) {
                            self.index = self.mail_actions().len() - 1;
                        }
                    }
                }
                Key::Center => {
                    match self.mail_ui.folder {
                        Screen::Drafts => {
                            self.mailbox.drafts.remove(self.mail_ui.item);
                        }
                        Screen::Inbox => {
                            self.mailbox.inbox.remove(self.mail_ui.item);
                        }
                        _ => {
                            self.mailbox.outbox.remove(self.mail_ui.item);
                        }
                    }
                    self.dirty = true;
                    self.mail_return_folder();
                }
                _ => {}
            },
            Result if self.chat.route => match key {
                Key::Center | Key::SoftRight => self.open_thread(self.chat.peer.clone()),
                _ => {}
            },
            Result => match key {
                Key::Center => {
                    self.mail_ui.scroll = 0;
                    self.go(Screen::Mail(Read));
                }
                Key::SoftRight => self.mail_return_folder(),
                Key::SoftLeft => {} // No service; never duplicate the queued item or invent delivery.
                _ => {}
            },
        }
    }
}
