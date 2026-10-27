//! Inline timer form and running menu sampled on 17.00.17.02.
use super::*;
use tools::ToolPage;

pub struct TimerEdit {
    pub seconds: u64,
    pub digit: usize,
    pub name: Editor,
}
impl Default for TimerEdit {
    fn default() -> Self {
        let mut name = Editor::new("", 20); // Native child-edit descriptor: 0x14 UTF-16 units.
        name.prediction
            .set_language(Some(phone_core::predict::Language::Chinese));
        Self {
            seconds: 60,
            digit: 0,
            name,
        }
    }
}
impl Phone {
    pub fn timer_name_active(&self) -> bool {
        matches!(self.screen, Screen::Tools(_))
            && self.tools.page == ToolPage::Timer
            && self.index == 1
    }
    pub(super) fn open_timer(&mut self) {
        self.tools.timer_edit = TimerEdit::default();
        self.tools.timer_edit.seconds = self.tools.timer_seconds;
        if self.tools.timer.running {
            self.tools.timer_edit.name = Editor::new(&self.tools.timer_name, 20);
            self.tools
                .timer_edit
                .name
                .prediction
                .set_language(Some(phone_core::predict::Language::Chinese));
        }
        self.tools.page = ToolPage::Timer;
        self.index = 0;
    }
    pub(super) fn timer_key(&mut self, key: Key) {
        if self.tools.page == ToolPage::TimerMenu {
            match key {
                Key::Up | Key::Down => self.index ^= 1,
                Key::Center if self.index == 0 => self.open_timer(),
                Key::Center => {
                    self.tools.timer = Default::default();
                    self.tools.timer_feedback = Some(("计时已停止", 2000));
                    self.tools.timer_notice = false;
                    self.tools.page = ToolPage::Other;
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.tools.page = ToolPage::Other;
                    self.index = 0;
                }
                _ => {}
            }
            return;
        }
        match key {
            Key::Up | Key::Down => {
                self.tools.timer_edit.name.commit();
                self.index ^= 1;
            }
            Key::Left if self.index == 0 => {
                self.tools.timer_edit.digit = (self.tools.timer_edit.digit + 5) % 6
            }
            Key::Right if self.index == 0 => {
                self.tools.timer_edit.digit = (self.tools.timer_edit.digit + 1) % 6
            }
            Key::Digit(d) if self.index == 0 => {
                let t = &mut self.tools.timer_edit;
                let mut digits = [
                    (t.seconds / 36000) as u8,
                    (t.seconds / 3600 % 10) as u8,
                    (t.seconds / 600 % 6) as u8,
                    (t.seconds / 60 % 10) as u8,
                    (t.seconds / 10 % 6) as u8,
                    (t.seconds % 10) as u8,
                ];
                if d <= 9 && (!matches!(t.digit, 2 | 4) || d <= 5) {
                    digits[t.digit] = d;
                    t.seconds = digits
                        .iter()
                        .zip([36000, 3600, 600, 60, 10, 1])
                        .map(|(d, w)| *d as u64 * w)
                        .sum();
                    t.digit = (t.digit + 1) % 6;
                }
            }
            Key::Center => {
                let edit = &mut self.tools.timer_edit;
                if !edit.name.accept_prediction() {
                    return;
                }
                edit.name.commit();
                if edit.seconds == 0 || edit.name.units.is_empty() {
                    return;
                }
                self.tools.timer_seconds = edit.seconds;
                self.tools.timer_name = edit.name.text();
                self.tools.timer.start(edit.seconds);
                self.tools.timer_feedback = Some(("计时开始", 2000));
                self.tools.timer_notice = false;
                self.tools.page = ToolPage::Other;
                self.index = 0;
            }
            Key::SoftRight => {
                self.tools.page = ToolPage::Other;
                self.index = 0;
            }
            _ => {}
        }
    }
}
