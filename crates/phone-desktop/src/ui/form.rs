//! Label/value fields and avatars shared by phone forms and contact headers.
use crate::canvas::{Canvas, Color, MUTED, WHITE};
#[derive(Clone, Copy)]
pub enum FieldKind {
    Input,
    Date,
    Choice,
}
pub struct Field<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub placeholder: &'a str,
    pub focused: bool,
    pub kind: FieldKind,
}
impl Field<'_> {
    pub fn draw(&self, c: &mut Canvas, y: i32) -> [i32; 4] {
        let choice = matches!(self.kind, FieldKind::Choice);
        if choice && self.focused {
            c.rect(0, y, 240, 68, c.accent(3));
        }
        c.text(
            6,
            y + if choice { 8 } else { 4 },
            self.label,
            if choice { 20 } else { 16 },
            if self.focused && !choice {
                c.accent(3)
            } else if choice {
                WHITE
            } else {
                MUTED
            },
        );
        c.text(
            6,
            y + if choice {
                37
            } else if matches!(self.kind, FieldKind::Date) {
                25
            } else {
                27
            },
            if self.value.is_empty() {
                self.placeholder
            } else {
                self.value
            },
            if choice {
                18
            } else if matches!(self.kind, FieldKind::Date) {
                16
            } else {
                20
            },
            if choice || self.value.is_empty() {
                MUTED
            } else {
                WHITE
            },
        );
        if !choice {
            c.rect(
                6,
                y + if matches!(self.kind, FieldKind::Date) {
                    53
                } else {
                    51
                },
                228,
                3,
                if self.focused && !matches!(self.kind, FieldKind::Date) {
                    c.accent(3)
                } else {
                    MUTED
                },
            );
        }
        [6, y + 27, 228, 26]
    }
}
pub fn avatar(c: &mut Canvas, x: i32, y: i32, letter: Option<char>, fill: Color) {
    for dy in -20..20 {
        for dx in -20..20 {
            if dx * dx + dy * dy <= 400 {
                c.rect(x + 20 + dx, y + 20 + dy, 1, 1, fill);
            }
        }
    }
    if let Some(letter) = letter {
        let text = letter.to_string();
        let w = c.width(&text, 16);
        c.text(x + 20 - w / 2, y + 10, &text, 16, [0, 0, 0, 255]);
    }
}
pub fn scroll_to(top: i32, height: i32, bottom: i32) -> i32 {
    (top + height - bottom).max(0)
}

pub fn date(c: &mut Canvas, value: &str, y: i32, digit: usize) {
    c.text(6, y, value, 16, WHITE);
    if value.len() == 10 && value.is_ascii() {
        let position = phone_core::calendar::Date::DIGIT_OFFSETS[digit.min(7)];
        let x = 6 + c.width(&value[..position], 16);
        let glyph = &value[position..position + 1];
        let width = c.width(glyph, 16);
        c.rect(x, y + 3, width, 19, WHITE);
        c.text(x, y, glyph, 16, [0, 0, 0, 255]);
    }
}
