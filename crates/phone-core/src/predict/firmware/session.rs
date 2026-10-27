//! Calls into the isolated original engine; all addresses refer to synthetic guest memory.
use super::arm::{Arm, LOAD, RAM, REQUEST, Result};
use crate::predict::{Candidate, Language};
use std::sync::{Arc, OnceLock};
const FIELD: u32 = RAM + 0x1000;
const TEXT: u32 = RAM + 0x10000;
const AUX: u32 = RAM + 0x12000;
const CONTEXT: u32 = RAM + 0x14000;
const UDB: u32 = RAM + 0x16000;
const KEY: u32 = 0x880cf39c;
pub const EN_UDB: usize = 4096;
pub const CN_UDB: usize = 110;
#[derive(Clone, Debug)]
pub struct Session {
    arm: Arm,
    language: Language,
}
struct Resources {
    code: Arc<[u8]>,
    english: Arc<[u8]>,
    chinese: Arc<[u8]>,
    tables: Arc<[u8]>,
}
fn resources() -> &'static Result<Resources> {
    static DATA: OnceLock<Result<Resources>> = OnceLock::new();
    DATA.get_or_init(|| {
        #[allow(dead_code)]
        mod packed {
            include!(concat!(env!("OUT_DIR"), "/packed.rs"));
        }
        Ok(Resources {
            code: crate::embedded::decode(packed::T9_CODE)?.into(),
            english: crate::embedded::decode(packed::T9_ENGLISH)?.into(),
            chinese: crate::embedded::decode(packed::T9_CHINESE)?.into(),
            tables: crate::embedded::decode(packed::T9_TABLES)?.into(),
        })
    })
}
impl Session {
    pub fn new(language: Language, udb: &[u8]) -> Result<Self> {
        let resources = resources().as_ref().map_err(Clone::clone)?;
        let chinese = language != Language::English;
        let mut s = Self {
            arm: Arm::new(
                resources.code.clone(),
                if chinese {
                    resources.chinese.clone()
                } else {
                    resources.english.clone()
                },
                resources.tables.clone(),
                chinese,
            ),
            language,
        };
        if chinese {
            s.ok(0x880d7718, &[FIELD, TEXT, AUX, 2048, 0, REQUEST, 4, LOAD])?;
            s.ok(0x880d7ba4, &[FIELD, 0x204, FIELD, 0])?;
            s.ok(
                if language == Language::Stroke {
                    0x880d73d8
                } else {
                    0x880d6e94
                },
                &[FIELD],
            )?;
            s.ok(0x880d6ff8, &[FIELD, 4])?;
        } else {
            s.ok(0x880e4e44, &[FIELD, TEXT, AUX, 2048, 0, REQUEST, LOAD])?;
            s.ok(0x880e54a8, &[FIELD, 0x109, FIELD, 0])?;
            for f in [0x880e56c8, 0x880e55b8] {
                s.ok(f, &[FIELD])?;
            }
        }
        let len = if chinese { CN_UDB } else { EN_UDB };
        if !udb.is_empty() {
            if udb.len() != len {
                return Err("Invalid native input dictionary size".into());
            }
            s.arm.ram[(UDB - RAM) as usize..(UDB - RAM) as usize + len].copy_from_slice(udb);
        }
        s.ok(
            if chinese { 0x880d7884 } else { 0x880e2220 },
            &[
                FIELD,
                UDB,
                len as u32,
                2,
                if chinese { 0x204 } else { 0x109 },
            ],
        )?;
        Ok(s)
    }
    fn ok(&mut self, pc: u32, args: &[u32]) -> Result<()> {
        let status = self.arm.call(pc, args)?;
        if status != 0 {
            return Err(format!("T9 status {status} at {pc:08x}"));
        }
        Ok(())
    }
    pub fn dictionary(&self) -> Vec<u8> {
        let len = if self.language == Language::English {
            EN_UDB
        } else {
            CN_UDB
        };
        self.arm.ram[(UDB - RAM) as usize..(UDB - RAM) as usize + len].to_vec()
    }
    pub fn context(&mut self, text: &str) -> Result<()> {
        // Cloned candidate pages share this complete-query instruction budget.
        self.arm.reset_budget();
        let units: Vec<u16> = text
            .chars()
            .rev()
            .take(16)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<String>()
            .encode_utf16()
            .collect();
        if !units.is_empty() {
            for (i, u) in units.iter().enumerate() {
                self.arm.write(CONTEXT + i as u32 * 2, *u as u32, 2)?;
            }
            self.ok(0x880cc5dc, &[FIELD, CONTEXT, units.len() as u32])?;
        }
        Ok(())
    }
    pub fn digits(&mut self, digits: &str) -> Result<bool> {
        if digits.len() > 36
            || !digits.bytes().all(|d| {
                if self.language == Language::Stroke {
                    (b'1'..=b'5').contains(&d)
                } else {
                    (b'2'..=b'9').contains(&d)
                }
            })
        {
            return Ok(false);
        }
        for digit in digits.bytes() {
            let status = self.arm.call(KEY, &[FIELD, u32::from(digit - b'0')])?;
            if status == 7 {
                return Ok(false);
            }
            if status != 0 {
                return Err(format!("T9 key rejected: {status}"));
            }
        }
        Ok(true)
    }
    fn spellings(&self) -> Result<Vec<String>> {
        if self.language != Language::Chinese {
            return Ok(vec![]);
        }
        let mut bytes = vec![];
        for i in 0..64 {
            let b = self.arm.read(FIELD + 0x326 + i, 1)? as u8;
            if b == 0 {
                break;
            }
            bytes.push(b);
        }
        let text = String::from_utf8(bytes).map_err(|_| "Invalid T9 spelling bytes")?;
        Ok(text.split_ascii_whitespace().map(str::to_owned).collect())
    }
    fn page(&self) -> Result<Vec<String>> {
        if self.language == Language::English {
            let len = self.arm.read(FIELD + 10, 2)?;
            let word = self.arm.read(FIELD + 14, 2)? + self.arm.read(FIELD + 16, 2)?;
            if word > len || len > 2048 {
                return Err("Invalid T9 English text bounds".into());
            }
            let units = (len - word..len)
                .map(|i| self.arm.read(TEXT + i * 2, 2).map(|u| u as u16))
                .collect::<Result<Vec<_>>>()?;
            let text = String::from_utf16(&units).map_err(|_| "Invalid T9 English text")?;
            return Ok(if text.is_empty() { vec![] } else { vec![text] });
        }
        let n = self.arm.read(FIELD + 0x31e, 2)?;
        if n > 4 {
            return Err("Invalid T9 Chinese page size".into());
        }
        (0..n)
            .map(|i| {
                char::from_u32(self.arm.read(FIELD + 0x30a + i * 2, 2)?)
                    .filter(|c| !c.is_control())
                    .map(|c| c.to_string())
                    .ok_or_else(|| "Invalid T9 Chinese character".into())
            })
            .collect()
    }
    pub fn candidates(&self, association: bool) -> Result<Vec<Candidate>> {
        let spellings = if association {
            vec![]
        } else {
            self.spellings()?
        };
        let groups = if spellings.is_empty() {
            vec![String::new()]
        } else {
            spellings
        };
        let mut result = vec![];
        let group_limit = (256 / groups.len() / 4).max(1) * 4;
        for (i, spelling) in groups.iter().enumerate() {
            let mut s = self.clone();
            if !spelling.is_empty() {
                s.ok(0x880d7330, &[FIELD, i as u32 + 1])?;
            }
            let mut seen = std::collections::HashSet::new();
            let group_start = result.len();
            // Bound work independently of malformed engine output or cyclic page cursors.
            for _ in 0..64 {
                let page = s.page()?;
                if page.is_empty() || !seen.insert(page.clone()) {
                    break;
                }
                result.extend(page.into_iter().map(|text| Candidate {
                    text,
                    spelling: spelling.clone(),
                }));
                if result.len() - group_start >= group_limit {
                    break;
                }
                if result.len() >= 256 {
                    result.truncate(256);
                    return Ok(result);
                }
                let status = s.arm.call(KEY, &[FIELD, 17])?;
                if status == 5 || status == 7 {
                    break;
                }
                if status != 0 {
                    return Err(format!("T9 candidate page failed: {status}"));
                }
            }
        }
        Ok(result)
    }
    pub fn choose(&mut self, candidate: &Candidate) -> Result<()> {
        if self.language == Language::English {
            for _ in 0..64 {
                if self.page()?.first() == Some(&candidate.text) {
                    return self.ok(KEY, &[FIELD, 19]);
                }
                if self.arm.call(KEY, &[FIELD, 17])? != 0 {
                    break;
                }
            }
            return Err("T9 English selection is no longer available".into());
        }
        if !candidate.spelling.is_empty() {
            let index = self
                .spellings()?
                .iter()
                .position(|s| s == &candidate.spelling)
                .ok_or("T9 selected spelling is no longer available")?;
            self.ok(0x880d7330, &[FIELD, index as u32 + 1])?;
        }
        for _ in 0..64 {
            if self.page()?.contains(&candidate.text) {
                for ch in candidate.text.chars() {
                    self.ok(0x880cf34c, &[FIELD, ch as u32])?;
                }
                return Ok(());
            }
            if self.arm.call(KEY, &[FIELD, 17])? != 0 {
                break;
            }
        }
        Err("T9 Chinese selection is no longer available".into())
    }
}
