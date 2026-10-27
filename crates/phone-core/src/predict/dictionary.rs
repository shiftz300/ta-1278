use super::{Candidate, Language, encode};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};
struct Entry {
    text: &'static str,
    spelling: &'static str,
    code: String,
    weight: u64,
}
fn dictionary(language: Language) -> &'static [Entry] {
    static CN: OnceLock<Vec<Entry>> = OnceLock::new();
    static EN: OnceLock<Vec<Entry>> = OnceLock::new();
    static ST: OnceLock<Vec<Entry>> = OnceLock::new();
    static CN_SOURCE: OnceLock<String> = OnceLock::new();
    static ST_SOURCE: OnceLock<String> = OnceLock::new();
    static EN_SOURCE: OnceLock<String> = OnceLock::new();
    #[allow(dead_code)]
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

pub(super) fn query(
    language: Language,
    digits: &str,
    weights: &HashMap<&str, (u32, u32)>,
) -> Vec<Candidate> {
    let dict = dictionary(language);
    let start = dict.partition_point(|e| e.code.as_str() < digits);
    let mut matching: Vec<_> = dict[start..]
        .iter()
        .take_while(|e| e.code.starts_with(digits))
        .collect();
    matching.sort_by_cached_key(|e| {
        let (context_weight, learned) = weights.get(e.text).copied().unwrap_or_default();
        (
            e.code != digits,
            std::cmp::Reverse(context_weight),
            std::cmp::Reverse(learned),
            std::cmp::Reverse(e.weight),
            e.code.len(),
            e.text,
        )
    });
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    for e in matching {
        if seen.insert(e.text) {
            candidates.push(Candidate {
                text: e.text.into(),
                spelling: e.spelling.into(),
            });
            if candidates.len()
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
    candidates
}
