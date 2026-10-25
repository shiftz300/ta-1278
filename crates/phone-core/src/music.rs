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
    held: Option<(Key, bool)>,
    seed: u32,
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
            held: None,
            seed: 1278,
        }
    }
}
impl Music {
    pub fn duration(&self) -> f32 {
        self.tracks.get(self.index).map_or(0., |t| t.1)
    }
    fn step(&mut self, delta: i32, natural: bool) {
        let n = self.tracks.len();
        if n == 0 {
            self.playing = false;
            return;
        }
        if natural && self.repeat == Repeat::One {
            self.position = 0.;
            return;
        }
        if natural && !self.shuffle && self.repeat == Repeat::Off && self.index + 1 == n {
            self.playing = false;
            self.position = self.duration();
            return;
        }
        if self.shuffle && n > 1 {
            self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
            self.index = (self.index + 1 + self.seed as usize % (n - 1)) % n;
        } else {
            self.index = (self.index as i32 + delta).rem_euclid(n as i32) as usize;
        }
        self.position = 0.;
    }
    pub fn event(&mut self, e: Event) {
        match (e.key, e.phase) {
            (Key::Center, Phase::Down) => {
                if !self.tracks.is_empty() {
                    if self.position >= self.duration() {
                        self.position = 0.;
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
        if self.playing && self.held.is_none() {
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
