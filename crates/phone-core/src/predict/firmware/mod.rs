//! Original T9 routines interpreted in bounded, isolated guest memory.
mod arm;
mod session;
use super::{Candidate, Engine, Language};
use serde::{Deserialize, Serialize};
use session::{CN_UDB, EN_UDB, Session};
use std::sync::Mutex;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub version: u32,
    pub english: Vec<u8>,
    pub chinese: Vec<u8>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            version: 1,
            english: vec![],
            chinese: vec![],
        }
    }
}
impl Data {
    fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || (!self.english.is_empty() && self.english.len() != EN_UDB)
            || (!self.chinese.is_empty() && self.chinese.len() != CN_UDB)
        {
            return Err("Invalid native input data; file retained".into());
        }
        Ok(())
    }
}
#[derive(Debug)]
struct State {
    data: Data,
    bases: [Session; 3],
    revision: u64,
    saved: u64,
    error: Option<String>,
}
#[derive(Debug)]
pub struct Firmware {
    state: Mutex<State>,
}
fn slot(l: Language) -> usize {
    match l {
        Language::English => 0,
        Language::Chinese => 1,
        Language::Stroke => 2,
    }
}
impl Firmware {
    pub fn from_data(mut data: Data) -> Result<Self, String> {
        data.validate()?;
        let bases = [
            Session::new(Language::English, &data.english)?,
            Session::new(Language::Chinese, &data.chinese)?,
            Session::new(Language::Stroke, &data.chinese)?,
        ];
        if (!data.english.is_empty() && bases[0].dictionary() != data.english)
            || (!data.chinese.is_empty() && bases[1].dictionary() != data.chinese)
        {
            return Err("Native input dictionary checksum failed; file retained".into());
        }
        // Initialize only our new, synthetic dictionaries. No handset user data is imported.
        data.english = bases[0].dictionary();
        data.chinese = bases[1].dictionary();
        Ok(Self {
            state: Mutex::new(State {
                data,
                bases,
                revision: 0,
                saved: 0,
                error: None,
            }),
        })
    }
    /// Copy learning data and its revision for an atomic host-owned state save.
    pub fn snapshot(&self) -> (Data, u64) {
        let state = self.state.lock().unwrap();
        (state.data.clone(), state.revision)
    }
    pub fn pending_save(&self) -> Option<(Data, u64)> {
        let s = self.state.lock().unwrap();
        (s.revision != s.saved).then(|| (s.data.clone(), s.revision))
    }
    pub fn saved(&self, revision: u64) {
        self.state.lock().unwrap().saved = revision;
    }
    pub fn take_error(&self) -> Option<String> {
        self.state.lock().unwrap().error.take()
    }
    fn run(
        &self,
        language: Language,
        digits: &str,
        context: &str,
        association: bool,
    ) -> Vec<Candidate> {
        let mut state = self.state.lock().unwrap();
        let result = (|| {
            let mut s = state.bases[slot(language)].clone();
            s.context(context)?;
            if !s.digits(digits)? {
                return Ok(vec![]);
            }
            s.candidates(association)
        })();
        match result {
            Ok(v) => v,
            Err(e) => {
                state.error = Some(e);
                vec![]
            }
        }
    }
}
impl Default for Firmware {
    fn default() -> Self {
        Self::from_data(Data::default()).expect("Verified bundled T9 engine")
    }
}
impl Engine for Firmware {
    fn query(&self, language: Language, digits: &str, context: &str) -> Vec<Candidate> {
        self.run(language, digits, context, false)
    }
    fn strict(&self) -> bool {
        true
    }
    fn associations(&self, language: Language, context: &str) -> Vec<Candidate> {
        if language == Language::English || context.is_empty() {
            vec![]
        } else {
            self.run(language, "", context, true)
        }
    }
    fn learn(&self, language: Language, digits: &str, context: &str, candidate: &Candidate) {
        let mut state = self.state.lock().unwrap();
        let result = (|| {
            let mut s = state.bases[slot(language)].clone();
            s.context(context)?;
            if !s.digits(digits)? {
                return Ok(());
            }
            s.choose(candidate)?;
            let udb = s.dictionary();
            let old = if language == Language::English {
                &state.data.english
            } else {
                &state.data.chinese
            };
            if &udb != old {
                if language == Language::English {
                    state.bases[0] = Session::new(Language::English, &udb)?;
                    state.data.english = udb;
                } else {
                    let cn = Session::new(Language::Chinese, &udb)?;
                    let st = Session::new(Language::Stroke, &udb)?;
                    state.bases[1] = cn;
                    state.bases[2] = st;
                    state.data.chinese = udb;
                }
                state.revision = state.revision.saturating_add(1);
            }
            Ok(())
        })();
        if let Err(e) = result {
            state.error = Some(e);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_independent_interpreter_fixtures() {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/prediction.json")).unwrap();
        let e = Firmware::default();
        for case in fixtures["cases"].as_array().unwrap() {
            let lang = match case["language"].as_str().unwrap() {
                "English" => Language::English,
                "Chinese" => Language::Chinese,
                "Stroke" => Language::Stroke,
                _ => unreachable!(),
            };
            let digits = case["digits"].as_str().unwrap();
            let expected: Vec<&str> = case["first_candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect();
            let actual = e.query(lang, digits, "");
            assert_eq!(e.take_error(), None, "{lang:?} {digits}");
            if expected.is_empty() {
                assert!(actual.is_empty(), "{lang:?} {digits}");
            } else {
                assert_eq!(
                    actual
                        .iter()
                        .take(expected.len())
                        .map(|c| c.text.as_str())
                        .collect::<Vec<_>>(),
                    expected,
                    "{lang:?} {digits}"
                );
            }
        }
    }
    #[test]
    fn chinese_late_candidate_learns_and_reloads_without_host_reordering() {
        let e = Firmware::default();
        let candidate = e
            .query(Language::Chinese, "64", "")
            .into_iter()
            .find(|c| c.text == "酿")
            .unwrap();
        e.learn(Language::Chinese, "64", "", &candidate);
        assert_eq!(e.take_error(), None);
        let (data, _) = e
            .pending_save()
            .expect("Original AUDB changes for a late selection");
        assert!(data.chinese[10..].iter().any(|b| *b != 0));
        let json = serde_json::to_vec(&data).unwrap();
        let other = Firmware::from_data(serde_json::from_slice(&json).unwrap()).unwrap();
        let a = e.query(Language::Chinese, "64", "");
        let b = other.query(Language::Chinese, "64", "");
        assert_eq!(
            a.iter().map(|c| (&c.text, &c.spelling)).collect::<Vec<_>>(),
            b.iter().map(|c| (&c.text, &c.spelling)).collect::<Vec<_>>()
        );
        assert_eq!(e.take_error(), None);
        assert_eq!(other.take_error(), None);
        let mut broken = data;
        broken.chinese[10] ^= 1;
        assert!(Firmware::from_data(broken).is_err());
    }
    #[test]
    fn original_candidate_order_pinyin_pages_strokes_and_context() {
        let engine = Firmware::default();
        let en = engine.query(Language::English, "4663", "");
        assert_eq!(
            en.iter()
                .take(4)
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>(),
            ["good", "home", "gone", "hood"]
        );
        let cn = engine.query(Language::Chinese, "64", "");
        assert_eq!(
            cn.iter()
                .take(8)
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>(),
            ["你", "呢", "尼", "泥", "逆", "拟", "腻", "妮"]
        );
        assert!(cn.iter().any(|c| c.spelling == "mi" && c.text == "米"));
        assert!(engine.query(Language::Chinese, "644", "").is_empty());
        assert_eq!(engine.query(Language::Stroke, "12", "")[0].text, "十");
        assert_eq!(engine.query(Language::Chinese, "426", "你")[0].text, "好");
        assert_eq!(engine.associations(Language::Chinese, "你")[0].text, "们");
        assert_eq!(engine.take_error(), None);
    }
    #[test]
    fn native_user_data_roundtrips_and_stale_save_cannot_clear_new_changes() {
        let e = Firmware::default();
        let c = e.query(Language::English, "4663", "")[1].clone();
        e.learn(Language::English, "4663", "", &c);
        let (data, first) = e
            .pending_save()
            .expect("Native word acceptance updates its user dictionary");
        let bytes = serde_json::to_vec(&data).unwrap();
        let reloaded = Firmware::from_data(serde_json::from_slice(&bytes).unwrap()).unwrap();
        assert_eq!(reloaded.state.lock().unwrap().data.english, data.english);
        e.learn(Language::English, "4663", "", &c);
        e.saved(first);
        assert!(e.pending_save().is_some());
        assert!(e.take_error().is_none());
    }
}
