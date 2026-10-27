//! Inline numeric conversion fields. The fields never swap when focus changes.
use super::*;
use phone_core::converter::{INPUT_LIMIT, PAIRS};
use tools::ToolPage;

pub struct ConversionUi {
    pub pair: usize,
    pub focus: usize,
    pub fields: [Editor; 2],
}
fn numeric(text: &str) -> Editor {
    let mut e = Editor::new(text, INPUT_LIMIT);
    e.numeric = true;
    e
}
impl Default for ConversionUi {
    fn default() -> Self {
        Self {
            pair: 0,
            focus: 0,
            fields: [numeric(""), numeric("")],
        }
    }
}
impl Shell {
    pub fn conversion_active(&self) -> bool {
        matches!(self.screen, Screen::Tools(_))
            && matches!(self.tools.page, ToolPage::Conversion(_))
            && self.overlay.is_none()
            && self.active_reminder().is_none()
            && self.messaging.ui.pending.is_none()
    }
    pub fn conversion_pair(&self) -> &'static phone_core::converter::Conversion {
        let category = match self.tools.page {
            ToolPage::Conversion(n) => n.min(PAIRS.len() - 1),
            _ => 0,
        };
        &PAIRS[category][self.tools.conversion.pair.min(PAIRS[category].len() - 1)]
    }
    fn recalculate(&mut self) {
        let state = &self.tools.conversion;
        let result = self
            .conversion_pair()
            .convert(&state.fields[state.focus].text(), state.focus == 1);
        let other = state.focus ^ 1;
        self.tools.conversion.fields[other] = numeric(result.as_deref().unwrap_or(""));
    }
    pub(super) fn conversion_text(&mut self, text: &str) {
        let e = &mut self.tools.conversion.fields[self.tools.conversion.focus];
        let mut proposed = e.text();
        let (a, b) = e
            .selection
            .map(|(a, b)| (a.min(b), a.max(b)))
            .unwrap_or((e.cursor, e.cursor));
        // Numeric fields contain ASCII only; UTF-16 positions equal byte positions.
        proposed.replace_range(a..b, text);
        let body = proposed.strip_prefix('-').unwrap_or(&proposed);
        if !body.chars().all(|c| c.is_ascii_digit() || c == '.') || body.matches('.').count() > 1 {
            return;
        }
        if e.insert(text) {
            self.recalculate();
        } else {
            self.status = "此数字太长–不得超过12位。".into();
        }
    }
    pub(super) fn conversion_key(&mut self, key: Key) {
        if let ToolPage::ConversionPairs(category) = self.tools.page {
            let n = PAIRS.get(category).map_or(0, |p| p.len());
            match key {
                Key::Up | Key::Down => {
                    vertical(&mut self.index, n, key, Boundary::Wrap);
                }
                Key::Center if self.index < n => {
                    self.tools.conversion = ConversionUi {
                        pair: self.index,
                        ..Default::default()
                    };
                    self.tools.page = ToolPage::Conversion(category);
                    self.index = 0;
                }
                Key::SoftRight => {
                    self.tools.page = ToolPage::Converter;
                    self.index = category;
                }
                _ => {}
            }
            return;
        }
        match key {
            Key::Digit(d) if d < 10 => self.conversion_text(&d.to_string()),
            Key::Hash => self.conversion_text("."),
            Key::Star => {
                let state = &mut self.tools.conversion;
                let e = &mut state.fields[state.focus];
                let text = e.text();
                let next = if let Some(text) = text.strip_prefix('-') {
                    text.to_owned()
                } else {
                    format!("-{text}")
                };
                if next.len() <= INPUT_LIMIT {
                    *e = numeric(&next);
                    self.recalculate();
                }
            }
            Key::Up | Key::Down => {
                self.tools.conversion.focus ^= 1;
            }
            Key::Left => {
                self.tools.conversion.fields[self.tools.conversion.focus].move_horizontal(-1)
            }
            Key::Right => {
                self.tools.conversion.fields[self.tools.conversion.focus].move_horizontal(1)
            }
            Key::Delete | Key::SoftRight
                if !self.tools.conversion.fields[self.tools.conversion.focus]
                    .units
                    .is_empty() =>
            {
                self.tools.conversion.fields[self.tools.conversion.focus].backspace();
                self.recalculate();
            }
            Key::SoftRight => {
                if let ToolPage::Conversion(n) = self.tools.page {
                    self.tools.page = ToolPage::ConversionPairs(n);
                    self.index = self.tools.conversion.pair;
                }
            }
            Key::SoftLeft => {
                self.tools.conversion.fields = [numeric(""), numeric("")];
            }
            _ => {}
        }
    }
}
