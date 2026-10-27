//! Local dialer and attempted-call history. No transport or simulated connection.
use super::*;
use serde::{Deserialize, Serialize};

pub const LIMIT: usize = 100;
#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    pub number: String,
    pub time: u64,
}
#[derive(Default)]
pub struct Calls {
    pub number: Editor,
    pub selected: Option<usize>,
    origin: Option<(Screen, usize)>,
    pub confirm_delete: bool,
}
pub fn valid(number: &str) -> bool {
    !number.is_empty() && number.bytes().any(|c| c.is_ascii_digit()) && editable(number)
}
fn editable(number: &str) -> bool {
    number.len() <= 40
        && number
            .bytes()
            .enumerate()
            .all(|(i, c)| c.is_ascii_digit() || matches!(c, b'*' | b'#') || (i == 0 && c == b'+'))
}
impl Phone {
    pub fn dial_number(&self) -> String {
        match self.calls.selected {
            Some(i) => self
                .local_data
                .contacts
                .get(i)
                .and_then(|c| c.iter().skip(3).take(3).find(|n| valid(n)))
                .cloned()
                .unwrap_or_default(),
            None => self.calls.number.text(),
        }
    }
    pub fn open_dialer(&mut self, number: String) {
        if self.screen != Screen::Tools(tools::ToolPage::Dial) {
            self.calls.origin = Some((self.screen, self.index));
        }
        self.calls.selected = None;
        self.calls.number = Editor::new(&number, 40);
        self.calls.number.numeric = true;
        self.tools.page = tools::ToolPage::Dial;
        self.go(Screen::Tools(tools::ToolPage::Dial));
    }
    pub(super) fn leave_dialer(&mut self) {
        let (screen, index) = self.calls.origin.take().unwrap_or((Screen::Home, 0));
        if let Screen::Tools(page) = screen {
            self.tools.page = page;
        }
        self.go(screen);
        self.index = index;
    }
    pub(super) fn dial_text(&mut self, text: &str) {
        // Reject a whole invalid edit, including insertion of + inside a number.
        let mut candidate = self.calls.number.clone();
        if candidate.insert(text) && editable(&candidate.text()) {
            self.calls.number = candidate;
            self.calls.selected = None;
        }
    }
    pub(super) fn calls_key(&mut self, key: Key) {
        use tools::ToolPage::{Calls, Dial};
        if self.tools.page == Dial {
            match key {
                Key::Up | Key::Down if !self.local_data.contacts.is_empty() => {
                    self.calls.selected = match (key, self.calls.selected) {
                        (Key::Up, Some(0)) | (Key::Up, None) => None,
                        (Key::Up, Some(i)) => Some(i - 1),
                        (Key::Down, Some(i)) => {
                            Some((i + 1).min(self.local_data.contacts.len() - 1))
                        }
                        _ => Some(0),
                    };
                }
                Key::Digit(d) if d < 10 => self.dial_text(&d.to_string()),
                Key::Star => self.dial_text("*"),
                Key::Hash => self.dial_text("#"),
                Key::Left => self.calls.number.move_horizontal(-1),
                Key::Right => self.calls.number.move_horizontal(1),
                Key::Delete => {
                    self.calls.number.backspace();
                    self.calls.selected = None;
                }
                Key::SoftRight if !self.calls.number.units.is_empty() => {
                    self.calls.number.backspace();
                    self.calls.selected = None;
                }
                Key::SoftRight => self.leave_dialer(),
                Key::SoftLeft => {
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::Dial,
                        index: 0,
                    })
                }
                Key::Call | Key::Center => {
                    let number = self.dial_number();
                    if !valid(&number) {
                        self.notice("请输入有效的电话号码");
                        return;
                    }
                    self.local_data.calls.insert(
                        0,
                        Record {
                            number,
                            time: self.timestamp() / 1000,
                        },
                    );
                    self.local_data.calls.truncate(LIMIT);
                    self.local_dirty = true;
                    self.notice("未连接通话服务，无法拨出。");
                }
                _ => {}
            }
            return;
        }
        if self.tools.page != Calls {
            return;
        }
        if self.calls.confirm_delete {
            match key {
                Key::Center => {
                    if self.index < self.local_data.calls.len() {
                        self.local_data.calls.remove(self.index);
                        self.index = self
                            .index
                            .min(self.local_data.calls.len().saturating_sub(1));
                        self.local_dirty = true;
                    }
                    self.calls.confirm_delete = false;
                }
                Key::SoftRight => self.calls.confirm_delete = false,
                _ => {}
            }
            return;
        }
        match key {
            Key::Up | Key::Down => {
                vertical(
                    &mut self.index,
                    self.local_data.calls.len(),
                    key,
                    Boundary::Clamp,
                );
            }
            Key::Center | Key::Call => {
                if let Some(record) = self.local_data.calls.get(self.index) {
                    self.open_dialer(record.number.clone());
                } else if key == Key::Call {
                    self.open_dialer(String::new());
                }
            }
            Key::SoftLeft => self.open_dialer(String::new()),
            Key::Delete if !self.local_data.calls.is_empty() => self.calls.confirm_delete = true,
            Key::SoftRight => self.leave_app(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
