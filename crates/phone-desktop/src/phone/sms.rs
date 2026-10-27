//! Local SMS lifecycle and folder actions. No modem access.
use super::*;
impl Compose {
    pub(super) fn from_message(message: &Message, draft: Option<usize>) -> Self {
        let mut to = Editor::new(&message.to, 40.max(message.to.encode_utf16().count()));
        to.numeric = true;
        let mut body = Editor::new(&message.body, 4096.max(message.body.encode_utf16().count()));
        body.prediction
            .set_language(Some(phone_core::predict::Language::Chinese));
        Self {
            to,
            body,
            draft,
            time: message.time,
            recipient: false,
        }
    }
}
impl Phone {
    pub fn message(&self) -> Message {
        Message {
            to: self.compose.to.text(),
            body: self.compose.body.text(),
            time: self.compose.time,
            ..Default::default()
        }
    }
    pub(super) fn queue_message(&mut self) {
        if self.mail_ui.pending.is_some() {
            return;
        }
        let m = self.message();
        if m.to.is_empty() {
            self.notice(mail::operation_label(4).unwrap());
            return;
        }
        match self.mailbox.stage_service_queue(
            self.compose.draft,
            m,
            self.service_backend.as_deref(),
        ) {
            Ok(change) => {
                self.mail_ui.pending = Some(mail::PendingQueue {
                    change,
                    was_dirty: self.dirty,
                });
                self.pending_compose_close = false;
                self.dirty = true;
            }
            Err(e) => self.notice(if e == "Message is empty" {
                "请输入信息内容"
            } else {
                "请输入有效的电话号码"
            }),
        }
    }
    pub(super) fn save_and_close(&mut self) {
        self.compose.to.commit();
        self.compose.body.commit();
        self.save_draft();
        if self.dirty {
            self.pending_compose_close = true;
        } else {
            self.go(Screen::Messages);
        }
    }
    pub fn storage_saved(&mut self) {
        self.dirty = false;
        if let Some(pending) = self.mail_ui.pending.take() {
            self.mail_ui.item = pending.change.index;
            self.mail_ui.folder = Screen::Outbox;
            self.compose.draft = None;
            if self.chat.route {
                self.chat.peer = self.compose.to.text();
            }
            self.go(Screen::Mail(mail::MailPage::Result));
            return;
        }
        if self.pending_compose_close {
            self.pending_compose_close = false;
            self.go(Screen::Messages);
        }
    }
    pub fn storage_failed(&mut self) {
        // A later background retry must not close an editor the user has resumed editing.
        self.pending_compose_close = false;
        if let Some(pending) = self.mail_ui.pending.take() {
            match self.mailbox.rollback_queue(pending.change) {
                Ok(()) => self.dirty = pending.was_dirty,
                Err(e) => {
                    self.dirty = true;
                    self.notice(e);
                }
            }
        }
    }
    pub fn save_draft(&mut self) {
        let message = self.message();
        if self.screen == Screen::Conversation && message.body.is_empty() {
            if let Some(i) = self
                .compose
                .draft
                .take()
                .filter(|i| *i < self.mailbox.drafts.len())
            {
                self.mailbox.drafts.remove(i);
                self.dirty = true;
            }
            return;
        }
        if message.to.is_empty() && message.body.is_empty() {
            return;
        }
        let i = self.mailbox.save(self.compose.draft, message);
        self.compose.draft = Some(i);
        self.dirty = true;
    }
    pub(super) fn close_compose(&mut self) {
        if self.screen == Screen::Conversation && self.compose.body.units.is_empty() {
            self.save_draft();
            self.go(Screen::Messages);
        } else if self.compose.to.units.is_empty() && self.compose.body.units.is_empty() {
            self.go(Screen::Messages);
        } else {
            self.compose.to.commit();
            self.compose.body.commit();
            self.overlay = Some(Overlay::Menu {
                kind: MenuKind::ExitSms,
                index: 0,
            });
        }
    }
    pub(super) fn mail_list_key(&mut self, key: Key) {
        let drafts = self.screen == Screen::Drafts;
        let folder = self.screen;
        let count = self.mail_count(folder);
        match key {
            Key::Up => self.index = self.index.saturating_sub(1),
            Key::Down => self.index = (self.index + 1).min(count.saturating_sub(1)),
            Key::SoftRight => {
                let peer = self
                    .mail_content(folder, self.index)
                    .map(|m| m.peer.to_owned());
                self.go(Screen::Messages);
                self.index = peer
                    .and_then(|peer| {
                        self.mailbox
                            .conversations()
                            .iter()
                            .position(|c| c.peer == peer)
                    })
                    .map_or(0, |i| i + 1);
            }
            Key::Delete if count > 0 => {
                self.mail_ui.item = self.index;
                self.mail_ui.folder = self.screen;
                self.mail_delete_prompt();
            }
            Key::Center | Key::SoftLeft if count > 0 => {
                if drafts {
                    let m = &self.mailbox.drafts[self.index];
                    self.compose = Compose::from_message(m, Some(self.index));
                    self.go(Screen::Compose);
                } else {
                    self.mail_open(mail::MailPage::Read);
                }
            }
            _ => {}
        }
    }
}
