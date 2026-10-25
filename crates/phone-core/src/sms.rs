use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub to: String,
    pub body: String,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Mailbox {
    pub drafts: Vec<Message>,
    pub outbox: Vec<Message>,
}
/// Minimal undo information while the host is writing messages.json.
/// Not serialized; the UI must block concurrent mailbox mutations until completion.
pub struct QueueChange {
    pub index: usize,
    queued: Message,
    removed_draft: Option<(usize, Message)>,
}
impl Mailbox {
    pub fn save(&mut self, index: Option<usize>, message: Message) -> usize {
        if let Some(i) = index.filter(|i| *i < self.drafts.len()) {
            self.drafts[i] = message;
            i
        } else {
            self.drafts.push(message);
            self.drafts.len() - 1
        }
    }
    /// Local queue only. It deliberately cannot report delivery or invoke a modem.
    pub fn queue(&mut self, index: Option<usize>, message: Message) -> Result<(), &'static str> {
        self.stage_queue(index, message).map(|_| ())
    }
    pub fn stage_queue(
        &mut self,
        index: Option<usize>,
        message: Message,
    ) -> Result<QueueChange, &'static str> {
        if message.to.is_empty()
            || !message
                .to
                .chars()
                .enumerate()
                .all(|(i, c)| c.is_ascii_digit() || (i == 0 && c == '+'))
            || message.to == "+"
        {
            return Err("Enter a phone number");
        }
        if message.body.is_empty() {
            return Err("Message is empty");
        }
        let change = QueueChange {
            index: self.outbox.len(),
            queued: message.clone(),
            removed_draft: index
                .filter(|i| *i < self.drafts.len())
                .map(|i| (i, self.drafts.remove(i))),
        };
        self.outbox.push(message);
        Ok(change)
    }
    pub fn rollback_queue(&mut self, change: QueueChange) -> Result<(), &'static str> {
        // Refuse to undo a different/newer message; a receipt is not a blanket deletion.
        if self.outbox.len() != change.index + 1
            || self.outbox.get(change.index) != Some(&change.queued)
            || change
                .removed_draft
                .as_ref()
                .is_some_and(|(i, _)| *i > self.drafts.len())
        {
            return Err("Mailbox changed during save; in-memory data retained");
        }
        self.outbox.pop();
        if let Some((i, message)) = change.removed_draft {
            self.drafts.insert(i, message);
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq)]
pub struct Count {
    pub remaining: usize,
    pub segments: usize,
    pub ucs2: bool,
}
/// Reconstructed from 883a492c/883a4ade, 880b0754, 88351b12, 88352232/96.
/// Explicit replica configuration: alphabet_management=1, sms_encode=0.
pub fn count(text: &str) -> Count {
    let one = "ª∞º`";
    let two = "çÁÍÓÚáíóúêÔôÊÀÃÕÂãõâ\x1bŞşıĞğİ";
    let ext = "^{}\\[~]|€";
    let default = "@£$¥èéùìòÇ\nØø\rÅåΔ_ΦΓΛΩΠΨΣΘΞ\x1bÆæßÉ !\"#¤%&'()*+,-./0123456789:;<=>?¡ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÑÜ§¿abcdefghijklmnopqrstuvwxyzäöñüà";
    let mut n = 0;
    let mut r1 = 0;
    let mut r2 = 0;
    let mut ex = 0;
    let mut ucs2 = false;
    let mut langs = [0; 5];
    for u in text.encode_utf16() {
        n += 1;
        let c = char::from_u32(u as u32).unwrap_or('\u{fffd}');
        if u > 127
            && !one.contains(c)
            && !two.contains(c)
            && !ext.contains(c)
            && !default.contains(c)
        {
            ucs2 = true;
        } else if two.contains(c) {
            r2 += 1;
        } else if one.contains(c) {
            r1 += 1;
        } else if ext.contains(c) {
            ex += 1;
        }
        if "ŞşĞğİı".contains(c) {
            langs[0] += 1;
        } else if "ÀÂêÔôÊÃãÕõâº∞ª`".contains(c) {
            langs[2] += 1;
        } else if "ÚúÍíÓóÁá".contains(c) {
            langs[4] += 1;
        }
    }
    ucs2 |= langs[0] > 0 && (langs[2] > 0 || langs[4] > 0);
    let used = if ucs2 { n } else { n + ex + r2 };
    let (single, multi) = if ucs2 {
        (70, 67)
    } else if r2 > 0 {
        (155, 146)
    } else if r1 > 0 {
        (155, 149)
    } else {
        (160, 153)
    };
    let (ps, pm) = if !ucs2 && r2 > 0 && langs[0] == 0 && langs[2] == 0 {
        (155, 149)
    } else {
        (single, multi)
    };
    Count {
        remaining: if used <= single {
            single - used
        } else {
            (multi - used % multi) % multi
        },
        segments: if used <= ps { 1 } else { (used - 1) / pm + 1 },
        ucs2,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_boundaries_and_language_mix() {
        assert_eq!(
            count(&"a".repeat(161)),
            Count {
                remaining: 145,
                segments: 2,
                ucs2: false
            }
        );
        assert_eq!(
            count(&"中".repeat(71)),
            Count {
                remaining: 63,
                segments: 2,
                ucs2: true
            }
        );
        assert!(count("ŞÁ").ucs2);
        assert!(!count("Á").ucs2);
        assert_eq!(count("^€").remaining, 156);
        assert_eq!(count("😀").remaining, 68);
    }
    #[test]
    fn drafts_replace_and_queue_never_sends() {
        let mut m = Mailbox::default();
        let i = m.save(
            None,
            Message {
                to: "123".into(),
                body: "a".into(),
            },
        );
        m.save(
            Some(i),
            Message {
                to: "123".into(),
                body: "b".into(),
            },
        );
        assert_eq!(m.drafts.len(), 1);
        assert!(m.queue(Some(i), m.drafts[i].clone()).is_ok());
        assert!(m.drafts.is_empty());
        assert_eq!(m.outbox[0].body, "b");
        assert!(m.queue(None, Message::default()).is_err());
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    #[test]
    fn rollback_restores_middle_draft_and_refuses_a_changed_queue() {
        let mut m = Mailbox::default();
        for body in ["first", "middle", "last"] {
            m.save(
                None,
                Message {
                    to: "123".into(),
                    body: body.into(),
                },
            );
        }
        let before = m.drafts.clone();
        let change = m
            .stage_queue(
                Some(1),
                Message {
                    to: "123".into(),
                    body: "edited middle".into(),
                },
            )
            .unwrap();
        m.rollback_queue(change).unwrap();
        assert_eq!(m.drafts, before);
        assert!(m.outbox.is_empty());
        let change = m.stage_queue(Some(1), before[1].clone()).unwrap();
        m.outbox.push(Message {
            to: "456".into(),
            body: "newer".into(),
        });
        assert!(m.rollback_queue(change).is_err());
        assert_eq!(m.outbox.len(), 2);
        assert_eq!(m.drafts.len(), 2);
    }
}
