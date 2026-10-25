//! A projection of the local mailbox; no second copy of message bodies or modem state.
use super::Mailbox;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Incoming(usize),
    Outgoing(usize),
}
#[derive(Clone, Debug)]
pub struct Conversation {
    pub peer: String,
    pub items: Vec<Item>,
    pub drafts: Vec<usize>,
    pub unread: usize,
    pub time: u64,
}
pub struct Content<'a> {
    pub body: &'a str,
    pub time: u64,
    pub incoming: bool,
    pub unsent: bool,
}
impl Mailbox {
    pub fn content(&self, item: Item) -> Option<Content<'_>> {
        match item {
            Item::Incoming(i) => self.inbox.get(i).map(|m| Content {
                body: &m.body,
                time: m.time,
                incoming: true,
                unsent: false,
            }),
            Item::Outgoing(i) => self.outbox.get(i).map(|m| Content {
                body: &m.body,
                time: m.time,
                incoming: false,
                unsent: !m.sent,
            }),
        }
    }
    pub fn conversations(&self) -> Vec<Conversation> {
        let mut groups = BTreeMap::<String, Conversation>::new();
        let mut add =
            |peer: &str, item: Option<Item>, draft: Option<usize>, time: u64, unread: bool| {
                let c = groups
                    .entry(peer.trim().into())
                    .or_insert_with(|| Conversation {
                        peer: peer.trim().into(),
                        items: vec![],
                        drafts: vec![],
                        unread: 0,
                        time: 0,
                    });
                c.items.extend(item);
                c.drafts.extend(draft);
                c.time = c.time.max(time);
                c.unread += usize::from(unread);
            };
        // Stable fallback for older records without timestamps. No invented dates.
        for (i, m) in self.outbox.iter().enumerate() {
            add(&m.to, Some(Item::Outgoing(i)), None, m.time, false);
        }
        for (i, m) in self.inbox.iter().enumerate() {
            add(&m.from, Some(Item::Incoming(i)), None, m.time, !m.read);
        }
        for (i, m) in self.drafts.iter().enumerate() {
            add(&m.to, None, Some(i), m.time, false);
        }
        let mut groups: Vec<_> = groups.into_values().collect();
        for c in &mut groups {
            c.items.sort_by_key(|&i| self.content(i).unwrap().time);
        }
        groups.sort_by_key(|c| std::cmp::Reverse(c.time));
        groups
    }
    pub fn thread(&self, peer: &str) -> Option<Conversation> {
        self.conversations()
            .into_iter()
            .find(|c| c.peer == peer.trim())
    }
    pub fn read_thread(&mut self, peer: &str) -> bool {
        let mut changed = false;
        for m in &mut self.inbox {
            if m.from.trim() == peer.trim() && !m.read {
                m.read = true;
                changed = true;
            }
        }
        changed
    }
    pub fn delete_item(&mut self, item: Item) {
        match item {
            Item::Incoming(i) if i < self.inbox.len() => {
                self.inbox.remove(i);
            }
            Item::Outgoing(i) if i < self.outbox.len() => {
                self.outbox.remove(i);
            }
            _ => {}
        }
    }
}
