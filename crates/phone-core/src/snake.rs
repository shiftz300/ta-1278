//! Rules traced from firmware 17.00.17.02; see research archive: docs/evidence/snake-recovery.json.
//! Random update follows the kernel; initial state and scheduling are local policies.
use crate::input::Key;
use std::collections::VecDeque;
const MAPS: &[u8; 1344] = include_bytes!("../../../assets/snake/maps.bin");
const TICKS: &[u8; 18] = include_bytes!("../../../assets/snake/tick-ms.bin");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
#[derive(Clone, Copy, Debug)]
pub struct Bonus {
    pub position: (i32, i32),
    pub countdown: u8,
    ticks: u16,
}
pub struct Snake {
    pub body: VecDeque<(i32, i32)>,
    directions: VecDeque<Direction>,
    pub food: (i32, i32),
    pub bonus: Option<Bonus>,
    pub direction: Direction,
    pending: Direction,
    pub score: u32,
    pub running: bool,
    pub over: bool,
    pub kind: usize,
    pub level: u8,
    elapsed: u64,
    random: Random55,
}
impl Default for Snake {
    fn default() -> Self {
        Self {
            body: VecDeque::new(),
            directions: VecDeque::new(),
            food: (0, 2),
            bonus: None,
            direction: Direction::Right,
            pending: Direction::Right,
            score: 0,
            running: false,
            over: false,
            kind: 0,
            level: 1,
            elapsed: 0,
            random: Random55::default(),
        }
    }
}
impl Snake {
    pub const W: i32 = 12;
    pub const H: i32 = 16;
    pub fn start(&mut self) {
        // 0x8869023e: three nodes, head at (8,6), moving right.
        self.body = VecDeque::from([(8, 6), (7, 6), (6, 6)]);
        self.directions = VecDeque::from([Direction::Right; 3]);
        self.direction = Direction::Right;
        self.pending = Direction::Right;
        self.score = 0;
        self.bonus = None;
        self.over = false;
        self.running = true;
        self.elapsed = 0;
        self.place_food();
    }
    pub fn discard(&mut self) {
        self.running = false;
        self.body.clear();
        self.directions.clear();
        self.bonus = None;
    }
    pub fn segment_direction(&self, i: usize) -> Direction {
        self.directions.get(i).copied().unwrap_or(self.direction)
    }
    pub fn tick_ms(&self) -> u64 {
        let i = self.level.clamp(1, 8) as usize * 2;
        u16::from_le_bytes([TICKS[i], TICKS[i + 1]]) as u64
    }
    pub fn key(&mut self, k: Key) {
        let d = match k {
            Key::Up | Key::Digit(2) => Direction::Up,
            Key::Right | Key::Digit(6) => Direction::Right,
            Key::Down | Key::Digit(8) => Direction::Down,
            Key::Left | Key::Digit(4) => Direction::Left,
            _ => return,
        };
        let reverse = matches!(
            (self.direction, d),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        );
        // 0x88099b88..0x88099bea rejects both directions on the current
        // axis, so a repeated straight key cannot cancel a queued turn.
        if !reverse && d != self.direction {
            self.pending = d;
        }
    }
    pub fn wall(&self, x: i32, y: i32) -> bool {
        (0..Self::W).contains(&x)
            && (0..Self::H).contains(&y)
            && MAPS[self.kind.min(5) * 192 + (y * Self::W + x) as usize] != 0
    }
    fn free_position(&mut self, avoid: Option<(i32, i32)>) -> Option<(i32, i32)> {
        // Native uses TWO draws at 0x886902dc/ea: x %12, y %14 +2.
        let free = |s: &Self, p| !s.body.contains(&p) && !s.wall(p.0, p.1) && Some(p) != avoid;
        for _ in 0..336 {
            let p = (
                (self.random.next() % 12) as i32,
                (self.random.next() % 14) as i32 + 2,
            );
            if free(self, p) {
                return Some(p);
            }
        }
        (2..Self::H)
            .flat_map(|y| (0..Self::W).map(move |x| (x, y)))
            .find(|&p| free(self, p))
    }
    fn place_food(&mut self) {
        if let Some(p) = self.free_position(self.bonus.map(|b| b.position)) {
            self.food = p;
        } else {
            self.running = false;
            self.over = true;
        }
    }
    fn bonus_tick(&mut self) {
        let divisor = 1000 / self.tick_ms() as u16;
        if let Some(b) = &mut self.bonus {
            b.ticks += 1;
            if b.ticks > 1000 {
                b.ticks = 0;
            } else if b.ticks.is_multiple_of(divisor) {
                if b.countdown == 0 {
                    self.bonus = None;
                } else {
                    b.countdown -= 1;
                }
            }
        }
    }
    pub fn tick(&mut self, ms: u64) {
        if !self.running || self.body.is_empty() {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(ms.min(1000));
        let delay = self.tick_ms();
        while self.elapsed >= delay && self.running {
            self.elapsed -= delay;
            self.direction = self.pending;
            let (dx, dy) = match self.direction {
                Direction::Up => (0, -1),
                Direction::Right => (1, 0),
                Direction::Down => (0, 1),
                Direction::Left => (-1, 0),
            };
            let h = self.body[0];
            let next = (
                (h.0 + dx).rem_euclid(Self::W),
                (h.1 + dy).rem_euclid(Self::H),
            );
            // 0x886904bc checks ALL nodes before removing the tail.
            if self.body.contains(&next) || self.wall(next.0, next.1) {
                self.running = false;
                self.over = true;
                break;
            }
            self.body.push_front(next);
            self.directions.push_front(self.direction);
            if next == self.food {
                self.score = self.score.wrapping_add(1) & 0xffff;
                self.place_food();
                if self.score > 0
                    && self.score.is_multiple_of(6)
                    && self.bonus.is_none()
                    && let Some(position) = self.free_position(Some(self.food))
                {
                    self.bonus = Some(Bonus {
                        position,
                        countdown: 6,
                        ticks: 0,
                    });
                }
            } else {
                if let Some(b) = self.bonus
                    && next == b.position
                {
                    self.score = (self.score + b.countdown as u32 * 3) & 0xffff;
                    self.bonus = None;
                }
                self.body.pop_back();
                self.directions.pop_back();
            }
            self.bonus_tick();
        }
    }
}
/// 0x8165ab58: add two uint32 words, store at the second pointer, advance
/// both pointers in a 0xdc-byte ring, return the sum with bit31 cleared.
struct Random55 {
    words: [u32; 55],
    read: usize,
    write: usize,
}
impl Default for Random55 {
    fn default() -> Self {
        // Boot-time seed table and pointer separation are not recovered.
        let mut seed = 1278u32;
        let words = std::array::from_fn(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed
        });
        Self {
            words,
            read: 0,
            write: 31,
        }
    }
}
impl Random55 {
    fn next(&mut self) -> u32 {
        let sum = self.words[self.read].wrapping_add(self.words[self.write]);
        self.words[self.write] = sum;
        self.read = (self.read + 1) % 55;
        self.write = (self.write + 1) % 55;
        sum & 0x7fff_ffff
    }
}
#[cfg(test)]
mod random_tests {
    use super::Random55;
    #[test]
    fn native_ring_update_masks_only_return_value() {
        let mut rng = Random55 {
            words: [0; 55],
            read: 54,
            write: 0,
        };
        rng.words[54] = 0x80000000;
        rng.words[0] = 3;
        assert_eq!(rng.next(), 3);
        assert_eq!(rng.words[0], 0x80000003);
        assert_eq!((rng.read, rng.write), (0, 1));
        rng.words[1] = u32::MAX;
        assert_eq!(rng.next(), 2);
        assert_eq!(rng.words[1], 0x80000002);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_initial_state_speed_and_growth() {
        let mut s = Snake::default();
        s.start();
        assert_eq!(s.body, [(8, 6), (7, 6), (6, 6)]);
        s.food = (9, 6);
        s.key(Key::Left);
        s.tick(224);
        assert_eq!(s.score, 0);
        s.tick(1);
        assert_eq!((s.body[0], s.score, s.body.len()), ((9, 6), 1, 4));
        s.running = false;
        let before = s.body.clone();
        s.tick(999);
        assert_eq!(s.body, before);
        for (i, ms) in [225, 200, 175, 150, 125, 100, 90, 88]
            .into_iter()
            .enumerate()
        {
            s.level = (i + 1) as u8;
            assert_eq!(s.tick_ms(), ms);
        }
    }
    #[test]
    fn recovered_maps_wrap_and_old_tail_collision() {
        let mut s = Snake::default();
        for (i, count) in [0, 12, 12, 12, 15, 24].into_iter().enumerate() {
            s.kind = i;
            assert_eq!(
                (0..16)
                    .flat_map(|y| (0..12).map(move |x| (x, y)))
                    .filter(|&(x, y)| s.wall(x, y))
                    .count(),
                count
            );
            s.start();
            assert!(!s.wall(s.food.0, s.food.1));
            assert!(s.food.1 >= 2);
        }
        s.kind = 0;
        s.start();
        s.body = VecDeque::from([(11, 6), (10, 6), (9, 6)]);
        s.food = (5, 5);
        s.tick(225);
        assert_eq!(s.body[0], (0, 6));
        s.body = VecDeque::from([(1, 1), (1, 2), (2, 2), (2, 1)]);
        s.tick(225);
        assert!(s.over);
    }
    #[test]
    fn same_axis_key_does_not_cancel_queued_turn() {
        let mut s = Snake::default();
        s.start();
        s.food = (0, 2);
        s.key(Key::Up);
        s.key(Key::Right);
        s.tick(s.tick_ms());
        assert_eq!(s.body[0], (8, 5));
        assert_eq!(s.direction, Direction::Up);
        s.key(Key::Left);
        s.key(Key::Down);
        s.tick(s.tick_ms());
        assert_eq!(s.body[0], (7, 5));
    }
    #[test]
    fn native_bonus_score_no_growth_and_tick_expiry() {
        let mut s = Snake::default();
        s.start();
        s.score = 5;
        s.food = (9, 6);
        s.tick(225);
        assert_eq!(s.score, 6);
        assert_eq!(s.bonus.unwrap().countdown, 6);
        s.bonus = Some(Bonus {
            position: (10, 6),
            countdown: 4,
            ticks: 0,
        });
        s.food = (2, 2);
        s.tick(225);
        assert_eq!(s.score, 18);
        assert_eq!(s.body.len(), 4);
        assert!(s.bonus.is_none());
        s.bonus = Some(Bonus {
            position: (0, 2),
            countdown: 1,
            ticks: 0,
        });
        for _ in 0..4 {
            s.bonus_tick();
        }
        assert_eq!(s.bonus.unwrap().countdown, 0);
        for _ in 0..4 {
            s.bonus_tick();
        }
        assert!(s.bonus.is_none());
    }
}
