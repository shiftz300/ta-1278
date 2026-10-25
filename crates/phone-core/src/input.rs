use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Center,
    SoftLeft,
    SoftRight,
    Call,
    End,
    Digit(u8),
    Star,
    Hash,
    Delete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Down,
    Up,
    Long,
    Repeat,
    Cancel,
}
#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub key: Key,
    pub phase: Phase,
}
struct Held {
    sources: Vec<u32>,
    start: u64,
    repeated: u64,
    long: bool,
}
#[derive(Default)]
pub struct Input {
    held: HashMap<Key, Held>,
}
impl Input {
    pub fn press(&mut self, source: u32, key: Key, now: u64) -> Option<Event> {
        if let Some(h) = self.held.get_mut(&key) {
            if !h.sources.contains(&source) {
                h.sources.push(source);
            }
            None
        } else {
            self.held.insert(
                key,
                Held {
                    sources: vec![source],
                    start: now,
                    repeated: now,
                    long: false,
                },
            );
            Some(Event {
                key,
                phase: Phase::Down,
            })
        }
    }
    pub fn release(&mut self, source: u32) -> Vec<Event> {
        let keys: Vec<_> = self
            .held
            .iter_mut()
            .filter_map(|(&k, h)| {
                h.sources.retain(|s| *s != source);
                h.sources.is_empty().then_some(k)
            })
            .collect();
        keys.into_iter()
            .map(|key| {
                self.held.remove(&key);
                Event {
                    key,
                    phase: Phase::Up,
                }
            })
            .collect()
    }
    pub fn cancel(&mut self) -> Vec<Event> {
        self.held
            .drain()
            .map(|(key, _)| Event {
                key,
                phase: Phase::Cancel,
            })
            .collect()
    }
    pub fn tick(&mut self, now: u64) -> Vec<Event> {
        let mut out = Vec::new();
        for (&key, h) in &mut self.held {
            if !h.long && now.saturating_sub(h.start) >= 500 {
                h.long = true;
                h.repeated = now;
                out.push(Event {
                    key,
                    phase: Phase::Long,
                });
            } else if h.long && now.saturating_sub(h.repeated) >= 120 {
                h.repeated = now;
                out.push(Event {
                    key,
                    phase: Phase::Repeat,
                });
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sources_share_key_without_early_release() {
        let mut i = Input::default();
        assert!(i.press(1, Key::Center, 0).is_some());
        assert!(i.press(2, Key::Center, 2).is_none());
        assert!(i.release(1).is_empty());
        assert_eq!(i.release(2)[0].phase, Phase::Up);
    }
    #[test]
    fn long_once_and_cancel_never_releases() {
        let mut i = Input::default();
        i.press(1, Key::Left, 0);
        assert!(i.tick(499).is_empty());
        assert_eq!(i.tick(500)[0].phase, Phase::Long);
        assert!(i.tick(501).is_empty());
        assert_eq!(i.cancel()[0].phase, Phase::Cancel);
        assert!(i.release(1).is_empty());
    }
}
