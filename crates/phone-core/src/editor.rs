//! UTF-16 offsets follow the recovered widget. Host edits preserve Unicode scalar boundaries.
use crate::predict::{Language, Prediction};
/// Scalar-safe UTF-16 stops and horizontal pixel positions supplied by the renderer.
#[derive(Clone, Debug, Default)]
pub struct VisualRow {
    pub stops: Vec<(usize, i32)>,
}
#[derive(Clone, Debug)]
pub struct Editor {
    pub units: Vec<u16>,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    pub limit: usize,
    pending: Option<(u8, usize, usize, u64)>,
    pub numeric: bool,
    pub uppercase: bool,
    pub titlecase: bool,
    pub prediction: Prediction,
    preferred_x: Option<i32>,
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
            titlecase: false,
            prediction: Prediction::default(),
            preferred_x: None,
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
    pub fn accept_prediction(&mut self) -> bool {
        if !self.prediction.active() {
            return true;
        }
        let Some(candidate) = self.prediction.current().cloned() else {
            return false;
        };
        if !self.insert_plain(&candidate.text) {
            return false;
        }
        self.prediction.learn(&candidate.text);
        self.prediction.clear();
        true
    }
    pub fn cycle_mode(&mut self) {
        if !self.accept_prediction() {
            return;
        }
        self.commit();
        match self.prediction.language {
            Some(Language::Chinese) => self.prediction.set_language(Some(Language::Stroke)),
            Some(Language::Stroke) => {
                self.prediction.set_language(None);
                self.titlecase = true;
                self.numeric = false;
                self.uppercase = false;
            }
            Some(Language::English) => self.prediction.set_language(None),
            None if self.numeric => {
                self.numeric = false;
                self.prediction.set_language(Some(Language::Chinese));
            }
            None if self.titlecase => self.titlecase = false,
            None if self.uppercase => {
                self.uppercase = false;
                self.numeric = true;
            }
            None => self.uppercase = true,
        }
    }
    pub fn mode_label(&self) -> &'static str {
        match self.prediction.language {
            Some(Language::Chinese) => "拼",
            Some(Language::Stroke) => "笔",
            Some(Language::English) => "EN9",
            None if self.numeric => "123",
            None if self.titlecase => "Abc",
            None if self.uppercase => "ABC",
            None => "abc",
        }
    }
    pub fn insert(&mut self, text: &str) -> bool {
        // Check the whole transaction before committing a candidate.
        let pending = self
            .prediction
            .current()
            .map_or(0, |c| c.text.encode_utf16().count());
        let selected = self.selection.map_or(0, |(a, b)| a.abs_diff(b));
        if self.units.len() - selected + pending + text.encode_utf16().count() > self.limit {
            return false;
        }
        if !self.accept_prediction() {
            return false;
        }
        self.insert_plain(text)
    }
    fn insert_plain(&mut self, text: &str) -> bool {
        self.preferred_x = None;
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
        self.preferred_x = None;
        if !self.accept_prediction() {
            return;
        }
        self.commit();
        self.selection = Some((0, self.units.len()));
    }
    pub fn clear(&mut self) {
        self.preferred_x = None;
        self.prediction.clear();
        self.units.clear();
        self.cursor = 0;
        self.selection = None;
        self.commit();
    }
    pub fn backspace(&mut self) {
        self.preferred_x = None;
        if self.prediction.active() {
            self.prediction.pop();
            return;
        }
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
        self.preferred_x = None;
        if !self.accept_prediction() {
            return;
        }
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
        if !self.accept_prediction() {
            return;
        }
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
    pub fn vertical_visual(&mut self, delta: i32, rows: &[VisualRow]) {
        if rows.is_empty() || !self.accept_prediction() {
            return;
        }
        self.commit();
        self.selection = None;
        let row = rows
            .partition_point(|r| r.stops.first().is_some_and(|s| s.0 <= self.cursor))
            .saturating_sub(1);
        let x = rows[row]
            .stops
            .iter()
            .find(|s| s.0 == self.cursor)
            .map_or(0, |s| s.1);
        let goal = *self.preferred_x.get_or_insert(x);
        let next = (row as i32 + delta).clamp(0, rows.len() as i32 - 1) as usize;
        if let Some(&(offset, _)) = rows[next].stops.iter().min_by_key(|s| (s.1 - goal).abs()) {
            self.cursor = offset.min(self.units.len());
            if self.cursor > 0
                && self.cursor < self.units.len()
                && (0xdc00..=0xdfff).contains(&self.units[self.cursor])
            {
                self.cursor -= 1;
            }
        }
    }
    /// Composition is visible but not yet committed or counted as message text.
    pub fn preview(&self) -> (Vec<u16>, Option<std::ops::Range<usize>>) {
        let mut units = self.units.clone();
        let Some(candidate) = self
            .prediction
            .current()
            .filter(|_| self.prediction.active())
        else {
            return (units, None);
        };
        let (start, end) = self
            .selection
            .map_or((self.cursor, self.cursor), |(a, b)| (a.min(b), a.max(b)));
        let composing = if self.prediction.language == Some(Language::Chinese) {
            candidate
                .spelling
                .chars()
                .filter(|c| !c.is_whitespace())
                .take(self.prediction.digits.len())
                .collect::<String>()
        } else if self.prediction.language == Some(Language::Stroke) {
            self.prediction
                .digits
                .chars()
                .map(|c| match c {
                    '1' => '一',
                    '2' => '丨',
                    '3' => '丿',
                    '4' => '丶',
                    '5' => '乛',
                    _ => c,
                })
                .collect()
        } else {
            candidate.text.clone()
        };
        let word: Vec<u16> = composing.encode_utf16().collect();
        let range = start..start + word.len();
        units.splice(start..end, word);
        (units, Some(range))
    }
    pub fn digit(&mut self, key: u8, now: u64) {
        self.preferred_x = None;
        if key > 9 {
            return;
        }
        self.tick(now);
        if self.prediction.language == Some(Language::Stroke) {
            self.commit();
            self.prediction.push(key);
            return;
        }
        if self.prediction.language.is_some() {
            match key {
                2..=9 => {
                    self.commit();
                    self.prediction.push(key);
                }
                0 => {
                    if self.prediction.language == Some(Language::English)
                        || !self.prediction.active()
                    {
                        if self.insert(" ") {
                            self.pending = Some((0, 0, self.cursor - 1, now));
                        }
                    } else {
                        self.accept_prediction();
                    }
                }
                1 if self.insert(if self.prediction.language == Some(Language::Chinese) {
                    "，"
                } else {
                    "."
                }) =>
                {
                    self.pending = Some((1, 0, self.cursor - 1, now));
                }
                _ => {}
            }
            return;
        }
        if self.numeric {
            self.insert(&key.to_string());
            return;
        }
        let chars: Vec<_> = TAP[key as usize].chars().collect();
        if let Some((old, index, at, _)) = self.pending
            && old == key
        {
            let next = (index + 1) % chars.len();
            let c = if self.uppercase
                || (self.titlecase
                    && (at == 0
                        || matches!(self.units.get(at.wrapping_sub(1)), Some(10 | 46 | 33 | 63))))
            {
                chars[next].to_ascii_uppercase()
            } else {
                chars[next]
            };
            self.units[at] = c as u16;
            self.cursor = at + 1;
            self.pending = Some((key, next, at, now));
            return;
        }
        let c = if self.uppercase
            || (self.titlecase
                && (self.cursor == 0
                    || matches!(
                        self.units.get(self.cursor.wrapping_sub(1)),
                        Some(10 | 46 | 33 | 63)
                    )))
        {
            chars[0].to_ascii_uppercase()
        } else {
            chars[0]
        };
        if self.insert(&c.to_string()) {
            self.pending = Some((key, 0, self.cursor - 1, now));
        }
    }
    pub fn long_digit(&mut self, key: u8) {
        if self.prediction.language.is_some() && key == 0 && self.pending.is_none() {
            self.insert("0");
            return;
        }
        if self.prediction.language.is_some() && (2..=9).contains(&key) {
            let original = self.prediction.clone();
            if self.prediction.digits.ends_with((b'0' + key) as char) {
                self.prediction.pop();
            }
            if !self.insert(&key.to_string()) {
                self.prediction = original;
            }
            return;
        }
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
    fn predictive_commit_selection_limit_and_long_digits() {
        let mut e = Editor::new("ab", 3);
        e.select_all();
        e.prediction.set_language(Some(Language::Chinese));
        for d in [6, 4, 4, 2, 6] {
            e.digit(d, 0);
        }
        assert_eq!(e.text(), "ab");
        assert!(e.accept_prediction());
        assert_eq!(e.text(), "你好");
        for d in [6, 4, 4, 2, 6] {
            e.digit(d, 0);
        }
        assert!(!e.accept_prediction());
        assert!(e.prediction.active());
        assert_eq!(e.text(), "你好");
        e.clear();
        e.digit(6, 0);
        e.long_digit(6);
        assert_eq!(e.text(), "6");
        e.digit(0, 1);
        e.long_digit(0);
        assert_eq!(e.text(), "60");
        e.digit(1, 2);
        e.long_digit(1);
        assert_eq!(e.text(), "601");
    }
    #[test]
    fn prediction_edit_and_mode_switch_are_transactional() {
        let mut e = Editor::default();
        e.prediction.set_language(Some(Language::English));
        for d in [4, 3, 5, 5, 6] {
            e.digit(d, 0);
        }
        e.digit(0, 1);
        assert_eq!(e.text(), "hello ");
        e.digit(4, 2);
        e.backspace();
        assert_eq!(e.text(), "hello ");
        assert!(!e.prediction.active());
        e.digit(4, 3);
        e.cycle_mode();
        assert!(e.prediction.language.is_none());
        assert!(!e.prediction.active());
    }
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

#[cfg(test)]
mod visual_tests {
    use super::*;
    #[test]
    fn proportional_vertical_motion_keeps_x_across_short_lines() {
        let mut e = Editor::new("中ab\ni\n中文文", 100);
        e.cursor = 2;
        let rows = vec![
            VisualRow {
                stops: vec![(0, 0), (1, 18), (2, 26), (3, 34)],
            },
            VisualRow {
                stops: vec![(4, 0), (5, 4)],
            },
            VisualRow {
                stops: vec![(6, 0), (7, 18), (8, 36), (9, 54)],
            },
        ];
        e.vertical_visual(1, &rows);
        assert_eq!(e.cursor, 5);
        e.vertical_visual(1, &rows);
        assert_eq!(e.cursor, 7); // x=26 is nearer 18; UTF-16 column 2 would incorrectly give 36.
        e.vertical_visual(-1, &rows);
        e.vertical_visual(-1, &rows);
        assert_eq!(e.cursor, 2); // Short middle row did not overwrite the desired x.
    }
    #[test]
    fn preview_replaces_selection_without_committing() {
        let mut e = Editor::new("A中😀B", 100);
        e.selection = Some((1, 4));
        e.prediction.set_language(Some(Language::Chinese));
        for d in [6, 4, 4, 2, 6] {
            e.digit(d, 0);
        }
        let (units, span) = e.preview();
        assert_eq!(String::from_utf16_lossy(&units), "AnihaoB");
        assert_eq!(span, Some(1..6));
        assert_eq!(e.text(), "A中😀B");
        assert!(e.accept_prediction());
        assert_eq!(e.text(), "A你好B");
    }
    #[test]
    fn native_hash_modes_and_titlecase_multitap() {
        let mut e = Editor::default();
        e.prediction.set_language(Some(Language::Chinese));
        for expected in ["笔", "Abc", "abc", "ABC", "123", "拼"] {
            e.cycle_mode();
            assert_eq!(e.mode_label(), expected);
        }
        e.cycle_mode();
        e.cycle_mode();
        e.digit(2, 0);
        e.digit(2, 100);
        e.digit(2, 200);
        assert_eq!(e.text(), "C");
        e.digit(2, 1000);
        assert_eq!(e.text(), "Ca");
    }
    #[test]
    fn stroke_replacement_is_usable_and_commits_a_character() {
        let mut e = Editor::default();
        e.prediction.set_language(Some(Language::Stroke));
        for d in [3, 2, 3, 5, 5, 3, 4] {
            e.digit(d, 0);
        }
        let i = e
            .prediction
            .candidates
            .iter()
            .position(|c| c.text == "你")
            .unwrap();
        e.prediction.selected = i;
        assert!(e.accept_prediction());
        assert_eq!(e.text(), "你");
    }
}
