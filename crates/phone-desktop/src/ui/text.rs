//! Text metric profiles shared by edit controls and their layout measurement.
use crate::canvas::{Canvas, Color, Line};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextMetrics {
    Standard,
    Message,
}
pub const MESSAGE: TextMetrics = TextMetrics::Message;
impl TextMetrics {
    fn char_width(self, c: &mut Canvas, ch: char, size: u16) -> i32 {
        let mut bytes = [0; 4];
        c.width(ch.encode_utf8(&mut bytes), sms_size(ch, size))
    }
    pub fn width(self, c: &mut Canvas, text: &str, size: u16) -> i32 {
        if self == Self::Standard {
            return c.width(text, size);
        }
        text.chars().map(|ch| self.char_width(c, ch, size)).sum()
    }
    pub fn draw(self, c: &mut Canvas, mut x: i32, y: i32, text: &str, size: u16, color: Color) {
        if self == Self::Standard {
            c.text(x, y, text, size, color);
            return;
        }
        for ch in text.chars() {
            let glyph_size = sms_size(ch, size);
            let top = y + if size == 20 && glyph_size == size {
                2
            } else {
                0
            };
            let mut bytes = [0; 4];
            c.text(x, top, ch.encode_utf8(&mut bytes), glyph_size, color);
            x += self.char_width(c, ch, size);
            if x > 240 {
                break;
            }
        }
    }
    pub fn wrap(self, c: &mut Canvas, text: &str, size: u16, width: i32) -> Vec<Line> {
        if self == Self::Standard {
            return c.wrap(text, size, width);
        }
        let mut rows = vec![];
        let mut row = String::new();
        let (mut start, mut offset, mut advance) = (0, 0, 0);
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch.is_ascii_alphanumeric() && !row.ends_with(|c: char| c.is_ascii_alphanumeric()) {
                let word_width = self.char_width(c, ch, size)
                    + chars
                        .clone()
                        .take_while(|c| c.is_ascii_alphanumeric())
                        .map(|ch| self.char_width(c, ch, size))
                        .sum::<i32>();
                if !row.is_empty() && advance + word_width > width {
                    rows.push(Line {
                        text: std::mem::take(&mut row),
                        start,
                        end: offset,
                    });
                    advance = 0;
                    start = offset;
                }
            }
            let next = self.char_width(c, ch, size);
            if ch == '\n' || (!row.is_empty() && advance + next > width) {
                rows.push(Line {
                    text: std::mem::take(&mut row),
                    start,
                    end: offset,
                });
                advance = 0;
                start = offset;
            }
            offset += ch.len_utf16();
            if ch == '\n' {
                start = offset;
            } else {
                row.push(ch);
                advance += next;
            }
        }
        rows.push(Line {
            text: row,
            start,
            end: offset,
        });
        rows
    }
}

fn sms_size(ch: char, size: u16) -> u16 {
    if size == 20
        && matches!(ch as u32, 0x2e80..=0x9fff | 0xf900..=0xfaff | 0xfe30..=0xfe4f | 0xff00..=0xffef)
    {
        24
    } else {
        size
    }
}
