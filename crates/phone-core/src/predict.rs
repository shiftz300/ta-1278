//! Offline nine-key prediction, newly implemented for the replica (not the Nokia T9 engine).
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Chinese,
    Stroke,
    English,
}
#[derive(Clone, Debug)]
pub struct Candidate {
    pub text: String,
    pub spelling: String,
}
struct Entry {
    text: &'static str,
    spelling: &'static str,
    code: String,
    weight: u64,
}
pub fn encode(text: &str) -> Option<String> {
    let mut code = String::new();
    for c in text.chars() {
        let d = match c.to_ascii_lowercase() {
            'a'..='c' => '2',
            'd'..='f' => '3',
            'g'..='i' => '4',
            'j'..='l' => '5',
            'm'..='o' => '6',
            'p'..='s' => '7',
            't'..='v' | 'ü' => '8',
            'w'..='z' => '9',
            ' ' | '\'' => continue,
            _ => return None,
        };
        code.push(d);
    }
    (!code.is_empty()).then_some(code)
}
fn dictionary(language: Language) -> &'static [Entry] {
    static CN: OnceLock<Vec<Entry>> = OnceLock::new();
    static EN: OnceLock<Vec<Entry>> = OnceLock::new();
    static ST: OnceLock<Vec<Entry>> = OnceLock::new();
    static CN_SOURCE: OnceLock<String> = OnceLock::new();
    static ST_SOURCE: OnceLock<String> = OnceLock::new();
    static EN_SOURCE: OnceLock<String> = OnceLock::new();
    mod packed {
        include!(concat!(env!("OUT_DIR"), "/packed.rs"));
    }
    let (slot, source, packed) = match language {
        Language::Chinese => (&CN, &CN_SOURCE, packed::PINYIN),
        Language::Stroke => (&ST, &ST_SOURCE, packed::STROKE),
        Language::English => (&EN, &EN_SOURCE, packed::ENGLISH),
    };
    slot.get_or_init(|| {
        let data = source.get_or_init(|| {
            String::from_utf8(crate::embedded::decode(packed).expect("Embedded dictionary"))
                .expect("UTF-8 dictionary")
        });
        let mut entries = Vec::new();
        let weights: HashMap<&str, u64> = if language == Language::Stroke {
            let mut weights = HashMap::new();
            for e in dictionary(Language::Chinese) {
                let w = weights.entry(e.text).or_insert(0);
                *w = (*w).max(e.weight);
            }
            weights
        } else {
            HashMap::new()
        };
        for line in data.lines() {
            let (text, spelling, weight) = if language == Language::Chinese {
                let mut p = line.split('\t');
                let Some(text) = p.next() else { continue };
                let Some(spelling) = p.next() else { continue };
                let Some(weight) = p.next() else { continue };
                (text, spelling, weight)
            } else if language == Language::Stroke {
                let mut fields = line.split('\t');
                let Some(text) = fields.next() else { continue };
                let Some(strokes) = fields.next() else {
                    continue;
                };
                (text, strokes, "1")
            } else {
                let Some((text, weight)) = line.split_once(' ') else {
                    continue;
                };
                if !text.bytes().all(|b| b.is_ascii_lowercase()) {
                    continue;
                }
                (text, text, weight)
            };
            let code = if language == Language::Stroke {
                spelling
                    .chars()
                    .map(|c| match c {
                        'h' => Some('1'),
                        's' => Some('2'),
                        'p' => Some('3'),
                        'n' => Some('4'),
                        'z' => Some('5'),
                        _ => None,
                    })
                    .collect::<Option<String>>()
            } else {
                encode(spelling)
            };
            let Some(code) = code else {
                continue;
            };
            let Ok(mut weight) = weight.trim().parse::<u64>() else {
                continue;
            };
            if language == Language::Stroke {
                weight = weights.get(text).copied().unwrap_or(weight);
            }
            if code.len() <= 36 {
                entries.push(Entry {
                    text,
                    spelling,
                    code,
                    weight,
                });
            }
        }
        entries.sort_by(|a, b| a.code.cmp(&b.code));
        entries
    })
}
#[derive(Clone, Debug, Default)]
pub struct Prediction {
    pub language: Option<Language>,
    pub digits: String,
    pub candidates: Vec<Candidate>,
    pub selected: usize,
    pub spellings: Vec<String>,
    pub spelling_selected: usize,
    pub spelling_focus: bool,
    alternatives: Vec<Candidate>,
    learned: HashMap<(Language, String), u32>,
}
impl Prediction {
    pub fn set_language(&mut self, language: Option<Language>) {
        self.clear();
        self.language = language;
    }
    pub fn clear(&mut self) {
        self.digits.clear();
        self.candidates.clear();
        self.selected = 0;
        self.spellings.clear();
        self.alternatives.clear();
        self.spelling_selected = 0;
        self.spelling_focus = false;
    }
    pub fn active(&self) -> bool {
        !self.digits.is_empty()
    }
    pub fn push(&mut self, digit: u8) {
        if (if self.language == Some(Language::Stroke) {
            (1..=5).contains(&digit)
        } else {
            (2..=9).contains(&digit)
        }) && self.digits.len() < 36
        {
            self.digits.push((b'0' + digit) as char);
            self.update();
        }
    }
    pub fn pop(&mut self) {
        self.digits.pop();
        self.update();
    }
    pub fn cycle(&mut self, delta: i32) {
        if self.spelling_focus && !self.spellings.is_empty() {
            self.spelling_selected = (self.spelling_selected as i32 + delta)
                .rem_euclid(self.spellings.len() as i32)
                as usize;
            self.filter_spelling();
            return;
        }
        if !self.candidates.is_empty() {
            self.selected =
                (self.selected as i32 + delta).rem_euclid(self.candidates.len() as i32) as usize;
        }
    }
    fn filter_spelling(&mut self) {
        if let Some(spelling) = self.spellings.get(self.spelling_selected) {
            self.candidates = self
                .alternatives
                .iter()
                .filter(|c| &c.spelling == spelling)
                .take(24)
                .cloned()
                .collect();
            self.selected = 0;
        }
    }
    pub fn current(&self) -> Option<&Candidate> {
        self.candidates.get(self.selected)
    }
    pub fn learn(&mut self, text: &str) {
        if let Some(language) = self.language {
            let n = self.learned.entry((language, text.into())).or_default();
            *n = n.saturating_add(1);
        }
    }
    fn update(&mut self) {
        self.selected = 0;
        self.candidates.clear();
        self.spellings.clear();
        self.alternatives.clear();
        self.spelling_selected = 0;
        self.spelling_focus = false;
        let Some(language) = self.language else {
            return;
        };
        if self.digits.is_empty() {
            return;
        }
        let dict = dictionary(language);
        let start = dict.partition_point(|e| e.code.as_str() < self.digits.as_str());
        let mut matching: Vec<_> = dict[start..]
            .iter()
            .take_while(|e| e.code.starts_with(&self.digits))
            .collect();
        matching.sort_by_cached_key(|e| {
            let learned = self
                .learned
                .get(&(language, e.text.into()))
                .copied()
                .unwrap_or(0);
            (
                e.code != self.digits,
                std::cmp::Reverse(learned),
                std::cmp::Reverse(e.weight),
                e.code.len(),
                e.text,
            )
        });
        let mut seen = HashSet::new();
        for e in matching {
            if seen.insert(e.text) {
                self.candidates.push(Candidate {
                    text: e.text.into(),
                    spelling: e.spelling.into(),
                });
                if self.candidates.len()
                    == if language == Language::Chinese {
                        256
                    } else {
                        24
                    }
                {
                    break;
                }
            }
        }
        if language == Language::Chinese && !self.candidates.is_empty() {
            let mut spellings = HashSet::new();
            for c in &self.candidates {
                if spellings.insert(c.spelling.clone()) {
                    self.spellings.push(c.spelling.clone());
                }
            }
            self.alternatives = std::mem::take(&mut self.candidates);
            self.filter_spelling();
            self.spelling_focus = true;
        }
        // Unknown words can always be entered as digits; direct typing/multi-tap is also available.
        if self.candidates.is_empty() {
            self.candidates.push(Candidate {
                text: self.digits.clone(),
                spelling: "digits · no match".into(),
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chinese_phrase_and_english_ambiguity() {
        let mut p = Prediction::default();
        p.set_language(Some(Language::Chinese));
        for d in [6, 4, 4, 2, 6] {
            p.push(d)
        }
        assert_eq!(p.current().unwrap().text, "你好");
        assert_eq!(p.current().unwrap().spelling, "ni hao");
        p.set_language(Some(Language::English));
        for d in [4, 6, 6, 3] {
            p.push(d)
        }
        assert!(p.candidates.iter().any(|c| c.text == "good"));
        assert!(p.candidates.iter().any(|c| c.text == "home"));
    }
    #[test]
    fn cancel_delete_learning_and_no_match() {
        let mut p = Prediction::default();
        p.set_language(Some(Language::English));
        for d in [4, 6, 6, 3] {
            p.push(d)
        }
        p.learn("home");
        p.pop();
        p.push(3);
        assert_eq!(p.current().unwrap().text, "home");
        p.clear();
        assert!(p.candidates.is_empty());
        for _ in 0..36 {
            p.push(9)
        }
        assert_eq!(p.current().unwrap().text, "9".repeat(36));
        p.push(9);
        assert_eq!(p.digits.len(), 36);
    }
}
