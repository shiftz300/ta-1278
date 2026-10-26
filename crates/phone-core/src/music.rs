use crate::input::{Event, Key, Phase};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Repeat {
    Off,
    All,
    One,
}
pub struct Music {
    pub index: usize,
    pub playing: bool,
    pub position: f32,
    pub volume: u8,
    pub repeat: Repeat,
    pub shuffle: bool,
    pub tracks: Vec<(String, f32)>,
    pub queue: Vec<usize>,
    pub external_clock: bool,
    pub generation: u64,
    pub seek_revision: u64,
    held: Option<(Key, bool)>,
    seed: u32,
    shuffled: Vec<usize>,
    shuffled_source: Vec<usize>,
}
impl Default for Music {
    fn default() -> Self {
        Self {
            index: 0,
            playing: false,
            position: 0.,
            volume: 6,
            repeat: Repeat::Off,
            shuffle: false,
            tracks: vec![
                ("Demo track 1".into(), 90.),
                ("Demo track 2".into(), 120.),
                ("Demo track 3".into(), 75.),
            ],
            external_clock: false,
            queue: vec![],
            generation: 0,
            seek_revision: 0,
            held: None,
            seed: 1278,
            shuffled: vec![],
            shuffled_source: vec![],
        }
    }
}
impl Music {
    pub fn duration(&self) -> f32 {
        self.tracks.get(self.index).map_or(0., |t| t.1)
    }
    pub fn play(&mut self, index: usize) {
        if index >= self.tracks.len() {
            return;
        }
        self.index = index;
        self.shuffled.clear();
        self.position = 0.;
        self.playing = true;
        self.generation = self.generation.wrapping_add(1);
    }
    pub fn ended(&mut self) {
        self.step(1, true);
    }
    pub fn order(&self) -> Vec<usize> {
        if self.queue.is_empty() {
            (0..self.tracks.len()).collect()
        } else {
            self.queue
                .iter()
                .copied()
                .filter(|i| *i < self.tracks.len())
                .collect()
        }
    }
    fn step(&mut self, delta: i32, natural: bool) {
        self.generation = self.generation.wrapping_add(1);
        let mut order = self.order();
        let n = order.len();
        if n == 0 {
            self.playing = false;
            return;
        }
        if natural && self.repeat == Repeat::One {
            self.position = 0.;
            return;
        }
        if self.shuffle {
            if self.shuffled_source != order || self.shuffled.is_empty() {
                self.shuffled_source = order.clone();
                // Permutation generation is host policy. Native end mode 1
                // (shuffle + repeat off) still stops when its queue wraps.
                let current = order.iter().position(|i| *i == self.index).unwrap_or(0);
                order.swap(0, current);
                for i in (2..n).rev() {
                    self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    order.swap(i, 1 + self.seed as usize % i);
                }
                self.shuffled = order;
            }
            order = self.shuffled.clone();
        } else {
            self.shuffled.clear();
        }
        let current = order.iter().position(|i| *i == self.index).unwrap_or(0);
        if natural && self.repeat == Repeat::Off && current + 1 == n {
            self.playing = false;
            self.position = self.duration();
            return;
        }
        self.index = order[(current as i32 + delta).rem_euclid(n as i32) as usize];
        self.position = 0.;
    }
    pub fn event(&mut self, e: Event) {
        match (e.key, e.phase) {
            (Key::Center, Phase::Down) => {
                if !self.tracks.is_empty() {
                    if self.position >= self.duration() {
                        self.position = 0.;
                        self.generation = self.generation.wrapping_add(1);
                    }
                    self.playing = !self.playing;
                }
            }
            (Key::Up, Phase::Down | Phase::Repeat) => self.volume = (self.volume + 1).min(10),
            (Key::Down, Phase::Down | Phase::Repeat) => self.volume = self.volume.saturating_sub(1),
            (Key::Left | Key::Right, Phase::Down) => self.held = Some((e.key, false)),
            (Key::Left | Key::Right, Phase::Long | Phase::Repeat) => {
                if self.held.is_some_and(|(k, _)| k == e.key) {
                    self.held = Some((e.key, true));
                    let d = if e.key == Key::Left { -5. } else { 5. };
                    self.position = (self.position + d).clamp(0., self.duration());
                    self.seek_revision = self.seek_revision.wrapping_add(1);
                }
            }
            (Key::Left | Key::Right, Phase::Up) => {
                if self.held == Some((e.key, false)) {
                    self.step(if e.key == Key::Left { -1 } else { 1 }, false)
                }
                self.held = None;
            }
            (_, Phase::Cancel) => self.held = None,
            _ => {}
        }
    }
    pub fn tick(&mut self, seconds: f32) {
        if !self.external_clock && self.playing && self.held.is_none() {
            self.position += seconds;
            if self.position >= self.duration() {
                self.step(1, true);
            }
        }
    }
    pub fn cycle_repeat(&mut self) {
        self.repeat = match self.repeat {
            Repeat::Off => Repeat::All,
            Repeat::All => Repeat::One,
            Repeat::One => Repeat::Off,
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn playlist_order_is_used_for_end_skip_repeat_and_shuffle() {
        let mut m = Music {
            queue: vec![2, 0],
            ..Music::default()
        };
        m.play(2);
        m.ended();
        assert_eq!(m.index, 0);
        m.ended();
        assert!(!m.playing);
        assert_eq!(m.index, 0);
        m.repeat = Repeat::All;
        m.playing = true;
        m.ended();
        assert_eq!(m.index, 2);
        m.shuffle = true;
        for _ in 0..20 {
            m.ended();
            assert!(matches!(m.index, 0 | 2));
        }
        m.queue = vec![999];
        m.ended();
        assert!(!m.playing);
    }
    #[test]
    fn short_release_and_long_seek() {
        let mut m = Music::default();
        m.event(Event {
            key: Key::Right,
            phase: Phase::Down,
        });
        assert_eq!(m.index, 0);
        m.event(Event {
            key: Key::Right,
            phase: Phase::Up,
        });
        assert_eq!(m.index, 1);
        m.event(Event {
            key: Key::Right,
            phase: Phase::Down,
        });
        m.event(Event {
            key: Key::Right,
            phase: Phase::Long,
        });
        m.event(Event {
            key: Key::Right,
            phase: Phase::Up,
        });
        assert_eq!(m.index, 1);
        assert_eq!(m.position, 5.);
    }
    #[test]
    fn eof_repeat_and_empty() {
        let mut m = Music {
            index: 2,
            playing: true,
            ..Music::default()
        };
        m.tick(100.);
        assert!(!m.playing);
        m.repeat = Repeat::All;
        m.playing = true;
        m.tick(1.);
        assert_eq!(m.index, 0);
        m.repeat = Repeat::One;
        m.tick(100.);
        assert_eq!(m.index, 0);
        m.tracks.clear();
        m.tick(100.);
        assert!(!m.playing);
    }
}

#[cfg(test)]
mod native_end_tests {
    use super::*;
    #[test]
    fn shuffle_without_repeat_visits_queue_once_then_stops_and_all_wraps() {
        let mut m = Music {
            shuffle: true,
            ..Music::default()
        };
        m.play(1);
        let mut seen = vec![m.index];
        for _ in 0..2 {
            m.ended();
            assert!(m.playing);
            assert!(!seen.contains(&m.index));
            seen.push(m.index);
        }
        m.ended();
        assert!(!m.playing);
        assert_eq!(m.position, m.duration());
        m.repeat = Repeat::All;
        m.playing = true;
        m.ended();
        assert_eq!(m.index, seen[0]);
        m.repeat = Repeat::One;
        m.ended();
        assert_eq!(m.index, seen[0]);
        m.queue = vec![2];
        m.repeat = Repeat::Off;
        m.play(2);
        m.ended();
        assert!(!m.playing);
    }
}
