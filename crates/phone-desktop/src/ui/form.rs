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
        self.draw_sized(c, y, 68, false)
    }
    /// Schedule fields share the same controls with shorter choice rows.
    pub fn draw_sized(&self, c: &mut Canvas, y: i32, height: i32, compact: bool) -> [i32; 4] {
        let choice = matches!(self.kind, FieldKind::Choice);
        if choice && self.focused {
            c.rect(0, y, 240, height, c.accent(3));
        }
        c.fit(
            6,
            y + if choice || compact { 8 } else { 4 },
            self.label,
            if choice && !compact { 20 } else { 16 },
            if self.focused && !choice {
                c.accent(3)
            } else if choice && !compact {
                WHITE
            } else {
                MUTED
            },
            228,
        );
        c.fit(
            6,
            y + if choice {
                37
            } else if compact {
                33
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
            if (choice && !compact) || self.value.is_empty() {
                MUTED
            } else {
                WHITE
            },
            228,
        );
        if !choice {
            c.rect(
                6,
                y + if compact {
                    height - 10
                } else if matches!(self.kind, FieldKind::Date) {
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

pub fn date(c: &mut Canvas, value: &str, y: i32, selected: usize) {
    c.text(6, y, value, 16, WHITE);
    if value.len() == 10 && value.is_ascii() {
        let position = phone_core::calendar::Date::DIGIT_OFFSETS[selected.min(7)];
        selected_digit(c, value, position, 6, y);
    }
}
pub fn datetime(c: &mut Canvas, date: &str, time: &str, y: i32, digit: Option<usize>) {
    c.text(6, y, date, 16, WHITE);
    c.text(124, y, time, 16, WHITE);
    if let Some(digit) = digit.filter(|d| *d < 12) {
        let (value, offset, left) = if digit < 8 {
            (date, phone_core::calendar::Date::DIGIT_OFFSETS[digit], 6)
        } else {
            (time, [0, 1, 3, 4][digit - 8], 124)
        };
        selected_digit(c, value, offset, left, y);
    }
}

fn selected_digit(c: &mut Canvas, value: &str, offset: usize, left: i32, y: i32) {
    if value.is_ascii() && offset < value.len() {
        let x = left + c.width(&value[..offset], 16);
        let glyph = &value[offset..offset + 1];
        let width = c.width(glyph, 16);
        c.rect(x, y + 3, width, 19, WHITE);
        c.text(x, y, glyph, 16, [0, 0, 0, 255]);
    }
}

/// Compact inline decimal child, measured from the native converter form.
/// Uses the same editor/caret and softkey system as the other forms.
pub fn numeric(
    c: &mut Canvas,
    label: &str,
    editor: &phone_core::editor::Editor,
    focused: bool,
    y: i32,
) -> Vec<phone_core::editor::VisualRow> {
    c.text(
        6,
        y + 8,
        label,
        16,
        if focused { c.accent(3) } else { MUTED },
    );
    if focused {
        c.text(185, y + 8, "✎123", 16, c.accent(3));
    }
    let lines = if focused {
        super::editor::Edit::new([6, y + 33, 228, 26])
            .sized(16, 26)
            .draw(c, editor)
    } else {
        c.fit(6, y + 33, &editor.text(), 16, WHITE, 228);
        vec![]
    };
    c.rect(6, y + 58, 228, 3, if focused { c.accent(3) } else { MUTED });
    lines
}
