//! Local SMS lifecycle and folder actions. No modem access.
use super::*;
impl Phone {
    pub fn message(&self) -> Message {
        Message {
            to: self.compose.to.text(),
            body: self.compose.body.text(),
        }
    }
    pub(super) fn queue_message(&mut self) {
        let m = self.message();
        match self.mailbox.queue(self.compose.draft, m) {
            Ok(()) => {
                self.dirty = true;
                self.go(Screen::Messages);
                self.notice("Queued locally. No SMS service: message was NOT sent.");
            }
            Err(e) => self.notice(e),
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
        if self.pending_compose_close {
            self.pending_compose_close = false;
            self.go(Screen::Messages);
        }
    }
    pub fn storage_failed(&mut self) {
        // A later background retry must not close an editor the user has resumed editing.
        self.pending_compose_close = false;
    }
    pub(super) fn save_draft(&mut self) {
        let message = self.message();
        if message.to.is_empty() && message.body.is_empty() {
            return;
        }
        let i = self.mailbox.save(self.compose.draft, message);
        self.compose.draft = Some(i);
        self.dirty = true;
    }
    pub(super) fn close_compose(&mut self) {
        if self.compose.to.units.is_empty() && self.compose.body.units.is_empty() {
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
        let count = if drafts {
            self.mailbox.drafts.len()
        } else {
            self.mailbox.outbox.len()
        };
        match key {
            Key::Up => self.index = self.index.saturating_sub(1),
            Key::Down => self.index = (self.index + 1).min(count.saturating_sub(1)),
            Key::SoftRight => self.go(Screen::Messages),
            Key::Delete if count > 0 => {
                if drafts {
                    self.mailbox.drafts.remove(self.index);
                } else {
                    self.mailbox.outbox.remove(self.index);
                }
                self.index = self.index.min(count.saturating_sub(2));
                self.dirty = true;
            }
            Key::Center | Key::SoftLeft if count > 0 => {
                if drafts {
                    let m = &self.mailbox.drafts[self.index];
                    let mut c = Compose::default();
                    c.to.insert(&m.to);
                    c.body.insert(&m.body);
                    c.draft = Some(self.index);
                    c.recipient = false;
                    self.compose = c;
                    self.go(Screen::Compose);
                } else {
                    let m = &self.mailbox.outbox[self.index];
                    self.notice(format!("NOT SENT\nTo: {}\n{}", m.to, m.body));
                }
            }
            _ => {}
        }
    }
}
