use super::{Candidate, Language, LearningData, dictionary};
use std::{collections::HashMap, fmt::Debug, sync::Mutex};
/// Replaceable prediction boundary. Text is UTF-8; digits are validated by Prediction.
/// Query must return promptly. No network, filesystem or native firmware execution.
pub trait Engine: Debug + Send + Sync {
    fn query(&self, language: Language, digits: &str, context: &str) -> Vec<Candidate>;
    fn learn(&self, _language: Language, _digits: &str, _context: &str, _candidate: &Candidate) {}
    fn strict(&self) -> bool {
        false
    }
    fn associations(&self, _language: Language, _context: &str) -> Vec<Candidate> {
        vec![]
    }
}
#[derive(Debug, Default)]
struct State {
    data: LearningData,
    revision: u64,
    saved: u64,
}
/// Independent dictionary predictor, not the recovered Nokia/Tegic engine.
#[derive(Debug, Default)]
pub struct Builtin {
    state: Mutex<State>,
}
impl Builtin {
    pub fn from_data(data: LearningData) -> Result<Self, String> {
        data.validate()?;
        Ok(Self {
            state: Mutex::new(State {
                data,
                ..Default::default()
            }),
        })
    }
    pub fn pending_save(&self) -> Option<(LearningData, u64)> {
        let state = self.state.lock().unwrap();
        (state.revision != state.saved).then(|| (state.data.clone(), state.revision))
    }
    pub fn saved(&self, revision: u64) {
        self.state.lock().unwrap().saved = revision;
    }
}
impl Engine for Builtin {
    fn query(&self, language: Language, digits: &str, context: &str) -> Vec<Candidate> {
        let state = self.state.lock().unwrap();
        let mut weights: HashMap<&str, (u32, u32)> = HashMap::new();
        for e in &state.data.entries {
            if e.language != language || !e.digits.starts_with(digits) {
                continue;
            }
            let w = weights.entry(&e.text).or_default();
            w.1 = w.1.saturating_add(e.count);
            if !context.is_empty() && e.context == context {
                w.0 = w.0.saturating_add(e.count);
            }
        }
        dictionary::query(language, digits, &weights)
    }
    fn learn(&self, language: Language, digits: &str, context: &str, candidate: &Candidate) {
        let mut state = self.state.lock().unwrap();
        if state.data.record(language, digits, context, candidate) {
            state.revision = state.revision.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_external_learning_is_rejected_and_old_save_keeps_new_changes_dirty() {
        let engine = Builtin::default();
        let word = Candidate {
            text: "home".into(),
            spelling: "home".into(),
        };
        engine.learn(Language::English, "0", "", &word);
        engine.learn(Language::English, "4663", &"x".repeat(17), &word);
        assert!(engine.pending_save().is_none());
        engine.learn(Language::English, "4663", "", &word);
        let (_, first) = engine.pending_save().unwrap();
        engine.learn(Language::English, "4663", "", &word);
        engine.saved(first);
        let (data, second) = engine.pending_save().unwrap();
        assert_eq!(data.entries[0].count, 2);
        assert!(data.validate().is_ok());
        engine.saved(second);
        assert!(engine.pending_save().is_none());
    }
}
