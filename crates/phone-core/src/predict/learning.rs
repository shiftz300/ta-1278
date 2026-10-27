use super::{Candidate, Language};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Learned {
    pub language: Language,
    pub digits: String,
    pub text: String,
    pub spelling: String,
    pub context: String,
    pub count: u32,
}
impl Learned {
    fn valid(&self) -> bool {
        !(self.digits.is_empty()
            || self.digits.len() > 36
            || !self.digits.bytes().all(|d| match self.language {
                Language::Stroke => (b'1'..=b'5').contains(&d),
                _ => (b'2'..=b'9').contains(&d),
            })
            || self.text.is_empty()
            || self.text.encode_utf16().count() > 64
            || self.spelling.chars().count() > 96
            || self.context.chars().count() > 16
            || self.count == 0)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LearningData {
    pub version: u32,
    pub entries: Vec<Learned>,
}
impl Default for LearningData {
    fn default() -> Self {
        Self {
            version: 1,
            entries: vec![],
        }
    }
}
impl LearningData {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.entries.len() > 2048 || self.entries.iter().any(|e| !e.valid())
        {
            return Err("Invalid input learning data; file retained".into());
        }
        Ok(())
    }
    pub(super) fn record(
        &mut self,
        language: Language,
        digits: &str,
        context: &str,
        candidate: &Candidate,
    ) -> bool {
        let new = Learned {
            language,
            digits: digits.into(),
            text: candidate.text.clone(),
            spelling: candidate.spelling.clone(),
            context: context.into(),
            count: 1,
        };
        if !new.valid() {
            return false;
        }
        if let Some(i) = self.entries.iter().position(|e| {
            e.language == language
                && e.digits == digits
                && e.text == candidate.text
                && e.context == context
        }) {
            let mut e = self.entries.remove(i);
            e.count = e.count.saturating_add(1);
            self.entries.push(e);
        } else {
            self.entries.push(new);
            if self.entries.len() > 2048 {
                self.entries.remove(0);
            }
        }
        true
    }
}
