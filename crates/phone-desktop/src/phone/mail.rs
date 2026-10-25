//! Local outbox reader/actions. Layout and folder policy are not recovered conversations.
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
impl Phone {
    pub fn mail_message(&self) -> Option<&Message> {
        if self.mail_ui.folder == Screen::Drafts {
            self.mailbox.drafts.get(self.mail_ui.item)
        } else {
            self.mailbox.outbox.get(self.mail_ui.item)
        }
    }
    pub(super) fn mail_return_folder(&mut self) {
        let count = if self.mail_ui.folder == Screen::Drafts {
            self.mailbox.drafts.len()
        } else {
            self.mailbox.outbox.len()
        };
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
                Key::Up => self.index = (self.index + 3) % 4,
                Key::Down => self.index = (self.index + 1) % 4,
                Key::SoftRight => self.go(Screen::Mail(Read)),
                Key::Center => match self.index {
                    // Backend is absent: retry retains this same queued record.
                    0 => self.go(Screen::Mail(Result)),
                    1 | 2 => {
                        let message = self.mail_message().unwrap().clone();
                        let forward = self.index == 2;
                        self.compose = Compose::from_message(&message, None);
                        if forward {
                            self.compose.to.clear();
                        }
                        self.compose.recipient = forward;
                        self.go(Screen::Compose);
                    }
                    _ => self.mail_delete_prompt(),
                },
                _ => {}
            },
            Delete => match key {
                Key::SoftRight => {
                    let back = self.mail_ui.delete_return;
                    if matches!(back, Screen::Drafts | Screen::Outbox) {
                        self.mail_return_folder();
                    } else {
                        self.go(back);
                        if back == Screen::Mail(Options) {
                            self.index = 3;
                        }
                    }
                }
                Key::Center => {
                    if self.mail_ui.folder == Screen::Drafts {
                        self.mailbox.drafts.remove(self.mail_ui.item);
                    } else {
                        self.mailbox.outbox.remove(self.mail_ui.item);
                    }
                    self.dirty = true;
                    self.mail_return_folder();
                }
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
