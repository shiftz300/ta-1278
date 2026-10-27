//! Shared input state for the bounded original engine and optional replacement providers.
mod dictionary;
mod engine;
pub mod firmware;
mod learning;
pub mod native;
pub use engine::{Builtin, Engine};
pub use learning::LearningData;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
pub fn default_engine() -> Arc<dyn Engine> {
    static ENGINE: std::sync::OnceLock<Arc<firmware::Firmware>> = std::sync::OnceLock::new();
    ENGINE
        .get_or_init(|| Arc::new(firmware::Firmware::default()))
        .clone()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Clone, Debug)]
pub struct Prediction {
    pub language: Option<Language>,
    pub digits: String,
    pub candidates: Vec<Candidate>,
    pub selected: usize,
    pub spellings: Vec<String>,
    pub spelling_selected: usize,
    pub spelling_focus: bool,
    alternatives: Vec<Candidate>,
    engine: Arc<dyn Engine>,
    context: String,
    learning_enabled: bool,
    association: bool,
}
impl Default for Prediction {
    fn default() -> Self {
        Self {
            language: None,
            digits: String::new(),
            candidates: vec![],
            selected: 0,
            spellings: vec![],
            spelling_selected: 0,
            spelling_focus: false,
            alternatives: vec![],
            engine: default_engine(),
            context: String::new(),
            learning_enabled: true,
            association: false,
        }
    }
}
impl Prediction {
    pub fn set_language(&mut self, language: Option<Language>) {
        self.clear();
        self.language = language;
    }
    pub fn clear(&mut self) {
        self.association = false;
        self.digits.clear();
        self.candidates.clear();
        self.selected = 0;
        self.spellings.clear();
        self.alternatives.clear();
        self.spelling_selected = 0;
        self.spelling_focus = false;
    }
    pub fn active(&self) -> bool {
        !self.digits.is_empty() || (self.association && !self.candidates.is_empty())
    }
    pub fn associating(&self) -> bool {
        self.association
    }
    pub fn show_associations(&mut self, context: &str) {
        self.clear();
        if !self.learning_enabled {
            return;
        }
        self.set_context(context);
        if let Some(language) = self.language {
            self.candidates = self.engine.associations(language, &self.context);
            self.candidates
                .retain(|c| !c.text.is_empty() && c.text.encode_utf16().count() <= 64);
            self.candidates.truncate(256);
            self.association = !self.candidates.is_empty();
        }
    }
    pub fn push(&mut self, digit: u8) {
        if (if self.language == Some(Language::Stroke) {
            (1..=5).contains(&digit)
        } else {
            (2..=9).contains(&digit)
        }) && self.digits.len() < 36
        {
            let previous = self.clone();
            self.association = false;
            self.digits.push((b'0' + digit) as char);
            self.update();
            if self.engine.strict() && self.candidates.is_empty() {
                *self = previous;
            }
        }
    }
    pub fn pop(&mut self) {
        if self.association {
            self.clear();
            return;
        }
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
                .take(256)
                .cloned()
                .collect();
            self.selected = 0;
        }
    }
    pub fn current(&self) -> Option<&Candidate> {
        self.candidates.get(self.selected)
    }
    pub fn set_engine(&mut self, engine: Arc<dyn Engine>) {
        if !Arc::ptr_eq(&self.engine, &engine) {
            self.engine = engine;
            self.update();
        }
    }
    pub fn set_context(&mut self, before_cursor: &str) {
        if !self.learning_enabled {
            self.context.clear();
            return;
        }
        self.context = before_cursor
            .chars()
            .rev()
            .take(16)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
    }
    pub fn learn(&mut self, text: &str) {
        if !self.learning_enabled {
            return;
        }
        if let Some(language) = self.language
            && let Some(candidate) = self.candidates.iter().find(|c| c.text == text)
        {
            self.engine
                .learn(language, &self.digits, &self.context, candidate);
        }
    }
    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
        if !enabled {
            self.context.clear();
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
        self.candidates = self.engine.query(language, &self.digits, &self.context);
        // Providers cannot break the shared candidate UI with unbounded output.
        self.candidates.retain(|c| {
            !c.text.is_empty()
                && c.text.encode_utf16().count() <= 64
                && c.spelling.chars().count() <= 96
        });
        self.candidates.truncate(256);
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
        if self.candidates.is_empty() && !self.engine.strict() {
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
        p.set_engine(Arc::new(Builtin::default()));
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
        p.set_engine(Arc::new(Builtin::default()));
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

#[cfg(test)]
mod engine_tests {
    use super::*;
    #[test]
    fn shared_learning_survives_new_editor_and_serialization() {
        let engine = Arc::new(Builtin::default());
        let mut first = Prediction::default();
        first.set_engine(engine.clone());
        first.set_language(Some(Language::English));
        for digit in [4, 6, 6, 3] {
            first.push(digit);
        }
        first.learn("home");
        let (data, revision) = engine.pending_save().unwrap();
        let bytes = serde_json::to_vec(&data).unwrap();
        engine.saved(revision);
        assert!(engine.pending_save().is_none());
        let restored =
            Arc::new(Builtin::from_data(serde_json::from_slice(&bytes).unwrap()).unwrap());
        let mut second = Prediction::default();
        second.set_engine(restored);
        second.set_language(Some(Language::English));
        for digit in [4, 6, 6, 3] {
            second.push(digit);
        }
        assert_eq!(second.current().unwrap().text, "home");
        first.learn("good");
        assert!(engine.pending_save().is_some());
    }
    #[test]
    fn replaceable_engine_sees_context_without_changing_editor_or_service_api() {
        #[derive(Debug)]
        struct Fixed;
        impl Engine for Fixed {
            fn query(&self, _: Language, digits: &str, context: &str) -> Vec<Candidate> {
                assert_eq!((digits, context), ("4", "你好 "));
                vec![Candidate {
                    text: "home".into(),
                    spelling: "home".into(),
                }]
            }
        }
        let mut e = crate::editor::Editor::new("你好 ", 100);
        e.prediction.set_engine(Arc::new(Fixed));
        e.prediction.set_language(Some(Language::English));
        e.digit(4, 0);
        assert!(e.accept_prediction());
        assert_eq!(e.text(), "你好 home");
    }
    #[test]
    fn bad_learning_version_is_rejected_and_context_can_change_ranking() {
        let data = LearningData {
            version: 2,
            ..Default::default()
        };
        assert!(Builtin::from_data(data).is_err());
        let mut p = Prediction::default();
        p.set_engine(Arc::new(Builtin::default()));
        p.set_language(Some(Language::English));
        p.set_context("go ");
        for d in [4, 6, 6, 3] {
            p.push(d);
        }
        p.learn("home");
        p.clear();
        p.set_context("very ");
        for d in [4, 6, 6, 3] {
            p.push(d);
        }
        p.learn("good");
        p.learn("good");
        p.clear();
        p.set_context("go ");
        for d in [4, 6, 6, 3] {
            p.push(d);
        }
        assert_eq!(p.current().unwrap().text, "home");
    }
}

#[cfg(test)]
mod privacy_tests {
    use super::*;
    #[test]
    fn private_editor_neither_passes_context_nor_records_candidates() {
        #[derive(Debug)]
        struct Private;
        impl Engine for Private {
            fn query(&self, _: Language, _: &str, context: &str) -> Vec<Candidate> {
                assert!(context.is_empty());
                vec![Candidate {
                    text: "home".into(),
                    spelling: "home".into(),
                }]
            }
            fn learn(&self, _: Language, _: &str, _: &str, _: &Candidate) {
                panic!("Private input reached learning");
            }
        }
        let mut e = crate::editor::Editor::new("secret ", 100);
        e.prediction.set_engine(Arc::new(Private));
        e.prediction.set_learning_enabled(false);
        e.prediction.set_language(Some(Language::English));
        e.digit(4, 0);
        assert!(e.accept_prediction());
        assert_eq!(e.text(), "secret home");
    }
}
