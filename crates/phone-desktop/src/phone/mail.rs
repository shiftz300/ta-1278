//! Local mailbox reader/actions. Layout and folder policy are not recovered conversations.
mod state;
use super::*;
use crate::ui::menu::{self, Event as MenuEvent, Item as MenuItem};
pub use state::State;
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
    const fn item(self) -> MenuItem<'static, Self> {
        MenuItem::new(self, self.label())
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Retry => "重新发送",
            Self::Edit => "编辑为新信息",
            Self::Reply => "回复",
            Self::Forward => "转发",
            Self::Delete => "删除",
        }
    }
}
pub fn delivery_text(state: &phone_core::services::SubmissionState) -> (&'static str, &str) {
    use phone_core::services::SubmissionState::*;
    match state {
        Queued => ("待发送", "信息已保存在本机"),
        Accepted => ("发送中", "服务已接受请求"),
        Sent => ("已发送", "尚未收到送达回执"),
        Delivered => ("已送达", "已收到送达回执"),
        Failed { reason } => ("发送失败", reason),
    }
}
impl Phone {
    pub fn mail_delivery(&self) -> (&'static str, &str) {
        let message = self.messaging.mailbox.outbox.get(self.messaging.ui.item);
        match message
            .and_then(|m| m.submission.as_ref())
            .map(|s| &s.state)
        {
            Some(state) => delivery_text(state),
            None if message.is_some_and(|m| m.sent) => ("已发送", ""),
            None => ("未发送", "短信服务未连接"),
        }
    }
    #[cfg(any(feature = "desktop", test))]
    pub fn receive_message(&mut self, from: String, body: String) -> Result<usize, &'static str> {
        if self.messaging.ui.pending.is_some() {
            return Err("Mailbox save in progress; retry after completion");
        }
        let selected = if self.screen == Screen::Messages && self.index > 0 {
            self.messaging
                .mailbox
                .conversations()
                .get(self.index - 1)
                .map(|c| c.peer.clone())
        } else {
            None
        };
        let index = self.messaging.mailbox.receive(from, body)?;
        self.received_message(index, selected);
        Ok(index)
    }
    pub(crate) fn received_message(&mut self, index: usize, selected: Option<String>) {
        if let Some(peer) = selected
            && let Some(i) = self
                .messaging
                .mailbox
                .conversations()
                .iter()
                .position(|c| c.peer == peer)
        {
            self.index = i + 1;
        }
        self.messaging.dirty = true;
        if self.screen == Screen::Conversation
            && self.messaging.chat.peer.trim() == self.messaging.mailbox.inbox[index].from.trim()
            && self.overlay.is_none()
            && self.messaging.chat.popup == conversation::Popup::None
            && self.messaging.chat.focus.is_none()
        {
            self.messaging.mailbox.inbox[index].read = true;
        }
    }
    pub fn mail_count(&self, folder: Screen) -> usize {
        match folder {
            Screen::Inbox => self.messaging.mailbox.inbox.len(),
            Screen::Drafts => self.messaging.mailbox.drafts.len(),
            _ => self.messaging.mailbox.outbox.len(),
        }
    }
    pub fn mail_content(&self, folder: Screen, index: usize) -> Option<MailContent<'_>> {
        if folder == Screen::Inbox {
            self.messaging
                .mailbox
                .inbox
                .get(index)
                .map(|m| MailContent {
                    peer: &m.from,
                    body: &m.body,
                    unread: !m.read,
                })
        } else {
            let messages = if folder == Screen::Drafts {
                &self.messaging.mailbox.drafts
            } else {
                &self.messaging.mailbox.outbox
            };
            messages.get(index).map(|m| MailContent {
                peer: &m.to,
                body: &m.body,
                unread: false,
            })
        }
    }
    pub fn mail_message(&self) -> Option<MailContent<'_>> {
        self.mail_content(self.messaging.ui.folder, self.messaging.ui.item)
    }
    pub fn mail_menu(&self) -> &'static [MenuItem<'static, MailAction>] {
        const INBOX: &[MenuItem<'static, MailAction>] = &[
            MailAction::Reply.item(),
            MailAction::Forward.item(),
            MailAction::Delete.item(),
        ];
        const OUTBOX: &[MenuItem<'static, MailAction>] = &[
            MailAction::Retry.item(),
            MailAction::Edit.item(),
            MailAction::Forward.item(),
            MailAction::Delete.item(),
        ];
        if self.messaging.ui.folder == Screen::Inbox {
            INBOX
        } else {
            OUTBOX
        }
    }
    pub(super) fn mail_return_folder(&mut self) {
        let count = self.mail_count(self.messaging.ui.folder);
        self.go(self.messaging.ui.folder);
        self.index = self.messaging.ui.item.min(count.saturating_sub(1));
    }
    pub(super) fn mail_delete_prompt(&mut self) {
        self.messaging.ui.delete_return = self.screen;
        self.go(Screen::Mail(MailPage::Delete));
    }
    pub(super) fn mail_open(&mut self, page: MailPage) {
        self.messaging.ui.item = self.index;
        self.messaging.ui.folder = self.screen;
        self.messaging.ui.scroll = 0;
        self.messaging.ui.lines = 0;
        if self.screen == Screen::Inbox
            && page == MailPage::Read
            && let Some(message) = self.messaging.mailbox.inbox.get_mut(self.index)
            && !message.read
        {
            message.read = true;
            self.messaging.dirty = true;
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
                Key::Up => self.messaging.ui.scroll = self.messaging.ui.scroll.saturating_sub(1),
                Key::Down => {
                    self.messaging.ui.scroll = (self.messaging.ui.scroll + 1)
                        .min(self.messaging.ui.lines.saturating_sub(7))
                }
                Key::SoftLeft | Key::Center => self.go(Screen::Mail(Options)),
                Key::SoftRight => self.mail_return_folder(),
                Key::Delete => self.mail_delete_prompt(),
                _ => {}
            },
            Options => match menu::event(self.mail_menu(), &mut self.index, key) {
                MenuEvent::Back => self.go(Screen::Mail(Read)),
                MenuEvent::Activate(action) => match action {
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
                        self.messaging.compose = Compose::from_message(&message, None);
                        self.messaging.compose.recipient = matches!(action, MailAction::Forward);
                        self.go(Screen::Compose);
                    }
                    MailAction::Delete => self.mail_delete_prompt(),
                },
                MenuEvent::Focus | MenuEvent::Ignored => {}
            },
            Delete => match key {
                Key::SoftRight => {
                    let back = self.messaging.ui.delete_return;
                    if matches!(back, Screen::Drafts | Screen::Outbox | Screen::Inbox) {
                        self.mail_return_folder();
                    } else {
                        self.go(back);
                        if back == Screen::Mail(Options) {
                            self.index = self.mail_menu().len() - 1;
                        }
                    }
                }
                Key::Center => {
                    match self.messaging.ui.folder {
                        Screen::Drafts => {
                            self.messaging.mailbox.drafts.remove(self.messaging.ui.item);
                        }
                        Screen::Inbox => {
                            self.messaging.mailbox.inbox.remove(self.messaging.ui.item);
                        }
                        _ => {
                            self.messaging.mailbox.outbox.remove(self.messaging.ui.item);
                        }
                    }
                    self.messaging.dirty = true;
                    self.mail_return_folder();
                }
                _ => {}
            },
            Result if self.messaging.chat.route => match key {
                Key::Center | Key::SoftRight => self.open_thread(self.messaging.chat.peer.clone()),
                _ => {}
            },
            Result => match key {
                Key::Center => {
                    self.messaging.ui.scroll = 0;
                    self.go(Screen::Mail(Read));
                }
                Key::SoftRight => self.mail_return_folder(),
                Key::SoftLeft => {} // No service; never duplicate the queued item or invent delivery.
                _ => {}
            },
        }
    }
}
