//! Playable replacement rules. Menu names are observed; native physics/maps were not recovered.
use crate::input::Key;
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
pub struct Snake {
    pub body: VecDeque<(i32, i32)>,
    pub food: (i32, i32),
    pub direction: Direction,
    pending: Direction,
    pub score: u32,
    pub running: bool,
    pub over: bool,
    pub kind: usize,
    pub level: u8,
    elapsed: u64,
    seed: u32,
}
impl Default for Snake {
    fn default() -> Self {
        Self {
            body: VecDeque::new(),
            food: (15, 10),
            direction: Direction::Right,
            pending: Direction::Right,
            score: 0,
            running: false,
            over: false,
            kind: 0,
            level: 3,
            elapsed: 0,
            seed: 1278,
        }
    }
}
impl Snake {
    pub const W: i32 = 24;
    pub const H: i32 = 24;
    pub fn start(&mut self) {
        self.body = VecDeque::from([(8, 12), (7, 12), (6, 12)]);
        self.direction = Direction::Right;
        self.pending = Direction::Right;
        self.score = 0;
        self.over = false;
        self.running = true;
        self.elapsed = 0;
        self.place_food();
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
        if !reverse {
            self.pending = d;
        }
    }
    pub fn wall(&self, x: i32, y: i32) -> bool {
        match self.kind {
            1 => x == 0 || y == 0 || x == 23 || y == 23,
            2 => y == 5 && (4..20).contains(&x) || y == 18 && (4..20).contains(&x),
            3 => x == 12 && (3..10).contains(&y) || y == 12 && (14..22).contains(&x),
            4 => x == 4 && (3..21).contains(&y) || x == 19 && (3..21).contains(&y),
            5 => {
                y == 6 && (2..10).contains(&x)
                    || y == 17 && (14..22).contains(&x)
                    || x == 12 && (6..18).contains(&y)
            }
            _ => false,
        }
    }
    fn place_food(&mut self) {
        for _ in 0..Self::W * Self::H * 2 {
            self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let n = (self.seed >> 8) % (Self::W * Self::H) as u32;
            let p = (n as i32 % Self::W, n as i32 / Self::W);
            if !self.body.contains(&p) && !self.wall(p.0, p.1) {
                self.food = p;
                return;
            }
        }
        for y in 0..Self::H {
            for x in 0..Self::W {
                if !self.body.contains(&(x, y)) && !self.wall(x, y) {
                    self.food = (x, y);
                    return;
                }
            }
        }
        self.running = false;
        self.over = true;
    }
    pub fn tick(&mut self, ms: u64) {
        if !self.running || self.body.is_empty() {
            return;
        }
        self.elapsed = self.elapsed.saturating_add(ms.min(1000));
        let delay = 320 - self.level.min(9) as u64 * 28;
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
            let eating = next == self.food;
            let occupied = self
                .body
                .iter()
                .take(self.body.len() - usize::from(!eating))
                .any(|p| *p == next);
            if occupied || self.wall(next.0, next.1) {
                self.running = false;
                self.over = true;
                break;
            }
            self.body.push_front(next);
            if eating {
                self.score += 10;
                self.place_food();
            } else {
                self.body.pop_back();
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_reverse_and_eat() {
        let mut s = Snake::default();
        s.start();
        s.food = (9, 12);
        s.key(Key::Left);
        s.tick(240);
        assert_eq!(s.body[0], (9, 12));
        assert_eq!(s.score, 10);
        assert_eq!(s.body.len(), 4);
        s.running = false;
        let before = s.body.clone();
        s.tick(999);
        assert_eq!(s.body, before);
    }
}
