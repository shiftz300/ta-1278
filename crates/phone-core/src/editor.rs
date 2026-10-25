//! UTF-16 offsets follow the recovered widget. Host edits preserve Unicode scalar boundaries.
#[derive(Clone, Debug)]
pub struct Editor {
    pub units: Vec<u16>,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    pub limit: usize,
    pending: Option<(u8, usize, usize, u64)>,
    pub numeric: bool,
    pub uppercase: bool,
}
// First recovered Latin multi-tap table. Selection of this table is a replica policy.
pub const TAP: [&str; 10] = [
    " 0\n",
    ".,'?!\"1-()@/:_",
    "abc2äæåàáâãç",
    "def3èéêëð",
    "ghi4ìíîï",
    "jkl5£",
    "mno6öøòóôõñ",
    "pqrs7ß$",
    "tuv8ùúûü",
    "wxyz9ýþ",
];
impl Default for Editor {
    fn default() -> Self {
        Self::new("", 4096)
    }
}
impl Editor {
    pub fn new(text: &str, limit: usize) -> Self {
        let units: Vec<_> = text.encode_utf16().collect();
        let cursor = units.len();
        Self {
            units,
            cursor,
            selection: None,
            limit,
            pending: None,
            numeric: false,
            uppercase: false,
        }
    }

    pub fn text(&self) -> String {
        String::from_utf16_lossy(&self.units)
    }
    pub fn commit(&mut self) {
        self.pending = None;
    }
    pub fn tick(&mut self, now: u64) {
        if self
            .pending
            .is_some_and(|(_, _, _, t)| now.saturating_sub(t) >= 750)
        {
            self.commit();
        }
    }
    fn erase_selection(&mut self) {
        if let Some((a, b)) = self.selection.take() {
            let (a, b) = (a.min(b), a.max(b));
            self.units.drain(a..b);
            self.cursor = a;
        }
    }
    pub fn insert(&mut self, text: &str) -> bool {
        self.commit();
        let incoming: Vec<_> = text.encode_utf16().collect();
        let selected = self.selection.map_or(0, |(a, b)| a.abs_diff(b));
        if self.units.len() - selected + incoming.len() > self.limit {
            return false;
        }
        self.erase_selection();
        let n = incoming.len();
        self.units.splice(self.cursor..self.cursor, incoming);
        self.cursor += n;
        true
    }
    pub fn select_all(&mut self) {
        self.commit();
        self.selection = Some((0, self.units.len()));
    }
    pub fn clear(&mut self) {
        self.units.clear();
        self.cursor = 0;
        self.selection = None;
        self.commit();
    }
    pub fn backspace(&mut self) {
        self.commit();
        if self.selection.is_some() {
            self.erase_selection();
            return;
        }
        if self.cursor > 0 {
            let end = self.cursor;
            self.move_horizontal(-1);
            self.units.drain(self.cursor..end);
        }
    }
    pub fn move_horizontal(&mut self, delta: i32) {
        self.commit();
        self.selection = None;
        if delta < 0 && self.cursor > 0 {
            self.cursor -= 1;
            if self.cursor > 0 && (0xdc00..=0xdfff).contains(&self.units[self.cursor]) {
                self.cursor -= 1;
            }
        }
        if delta > 0 && self.cursor < self.units.len() {
            if (0xd800..=0xdbff).contains(&self.units[self.cursor]) {
                self.cursor += 1;
            }
            self.cursor = (self.cursor + 1).min(self.units.len());
        }
    }
    pub fn vertical(&mut self, delta: i32, lines: &[usize]) {
        self.commit();
        self.selection = None;
        if lines.is_empty() {
            return;
        }
        let row = lines
            .partition_point(|&s| s <= self.cursor)
            .saturating_sub(1);
        let column = self.cursor - lines[row];
        let next = (row as i32 + delta).clamp(0, lines.len() as i32 - 1) as usize;
        let end = lines.get(next + 1).map_or(self.units.len(), |n| {
            if *n > 0 && self.units[*n - 1] == 10 {
                n - 1
            } else {
                *n
            }
        });
        self.cursor = (lines[next] + column).min(end);
        if self.cursor > 0
            && self.cursor < self.units.len()
            && (0xdc00..=0xdfff).contains(&self.units[self.cursor])
        {
            self.cursor -= 1;
        }
    }
    pub fn digit(&mut self, key: u8, now: u64) {
        if key > 9 {
            return;
        }
        self.tick(now);
        if self.numeric {
            self.insert(&key.to_string());
            return;
        }
        let chars: Vec<_> = TAP[key as usize].chars().collect();
        if let Some((old, index, at, _)) = self.pending
            && old == key
        {
            let next = (index + 1) % chars.len();
            let c = if self.uppercase {
                chars[next].to_ascii_uppercase()
            } else {
                chars[next]
            };
            self.units[at] = c as u16;
            self.cursor = at + 1;
            self.pending = Some((key, next, at, now));
            return;
        }
        let c = if self.uppercase {
            chars[0].to_ascii_uppercase()
        } else {
            chars[0]
        };
        if self.insert(&c.to_string()) {
            self.pending = Some((key, 0, self.cursor - 1, now));
        }
    }
    pub fn long_digit(&mut self, key: u8) {
        if let Some((old, _, at, _)) = self.pending
            && old == key
        {
            self.units[at] = u16::from(b'0' + key);
            self.cursor = at + 1;
            self.commit();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multitap_middle_timeout_selection() {
        let mut e = Editor::new("xy", 20);
        e.cursor = 1;
        e.digit(2, 0);
        e.digit(2, 100);
        assert_eq!(e.text(), "xby");
        e.digit(2, 900);
        assert_eq!(e.text(), "xbay");
        e.select_all();
        e.digit(3, 1000);
        assert_eq!(e.text(), "d");
    }
    #[test]
    fn utf16_and_limit() {
        let mut e = Editor::new("中😀", 3);
        assert_eq!(e.cursor, 3);
        assert!(!e.insert("a"));
        e.backspace();
        assert_eq!(e.text(), "中");
        e.select_all();
        assert!(e.insert("ab"));
    }
    #[test]
    fn long_numeric_and_navigation_commit() {
        let mut e = Editor::default();
        e.digit(2, 0);
        e.long_digit(2);
        assert_eq!(e.text(), "2");
        e.move_horizontal(-1);
        e.digit(3, 50);
        assert_eq!(e.text(), "d2");
    }
}
