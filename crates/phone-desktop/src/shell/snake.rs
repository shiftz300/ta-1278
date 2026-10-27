mod state;
use super::*;
use serde::{Deserialize, Serialize};
pub use state::State;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SnakeView {
    #[default]
    Menu,
    Game,
    GameOver,
    Saved,
    Types,
    Level,
    Scores,
    Sound,
}
/// Host storage mirrors the native six exposed map score lists; no shell NV is read.
#[derive(Clone, Serialize, Deserialize)]
pub struct SnakeData {
    pub level: u8,
    pub kind: usize,
    pub sound: bool,
    pub scores: [[u16; 5]; 6],
}
impl Default for SnakeData {
    fn default() -> Self {
        Self {
            level: 1,
            kind: 0,
            sound: true,
            scores: [[0; 5]; 6],
        }
    }
}
impl SnakeData {
    pub fn valid(&self) -> bool {
        (1..=8).contains(&self.level)
            && self.kind < 6
            && self
                .scores
                .iter()
                .all(|rows| rows.windows(2).all(|r| r[0] >= r[1]))
    }
    pub fn record(&mut self, kind: usize, score: u32) {
        let rows = &mut self.scores[kind.min(5)];
        let score = score.min(u16::MAX as u32) as u16;
        if let Some(i) = rows.iter().position(|&old| score > old) {
            rows.copy_within(i..4, i + 1);
            rows[i] = score;
        }
    }
}
pub const GAME_TYPES: [&str; 6] = ["经典", "盒子", "隧道", "磨坊", "铁轨", "公寓"];
impl Shell {
    /// Install validated host settings in both the model and the picker.
    pub fn apply_snake_data(&mut self, data: SnakeData) {
        self.gaming.model.level = data.level;
        self.gaming.model.kind = data.kind;
        self.gaming.level_preview = data.level;
        self.gaming.data = data;
    }
    pub fn snake_items(&self) -> Vec<String> {
        match self.gaming.view {
            SnakeView::Menu | SnakeView::Saved => {
                let mut v = vec![];
                if self.snake_can_resume() {
                    v.push("继续".into());
                }
                v.extend(["新游戏", "关卡", "游戏类型", "最高得分", "声音"].map(str::to_owned));
                v
            }
            SnakeView::Types => GAME_TYPES.map(str::to_owned).to_vec(),
            SnakeView::Sound => ["关", "开"].map(str::to_owned).to_vec(),
            _ => vec![],
        }
    }
    fn snake_can_resume(&self) -> bool {
        !self.gaming.model.body.is_empty() && !self.gaming.model.over
    }
    fn snake_menu(&mut self, row: usize) {
        self.gaming.view = SnakeView::Menu;
        self.index = row + usize::from(self.snake_can_resume());
    }
    pub(super) fn snake_key(&mut self, key: Key) {
        match self.gaming.view {
            SnakeView::GameOver => {
                if key == Key::SoftRight {
                    self.snake_menu(0);
                }
                return;
            }
            SnakeView::Saved => {
                if key == Key::Center {
                    self.snake_menu(0);
                }
                return;
            }
            SnakeView::Game => {
                if key == Key::SoftRight {
                    self.gaming.model.running = false;
                    self.gaming.view = SnakeView::Menu;
                    self.index = 0;
                } else {
                    self.gaming.model.key(key);
                }
                return;
            }
            SnakeView::Level => {
                match key {
                    Key::Left => {
                        self.gaming.level_preview =
                            self.gaming.level_preview.saturating_sub(1).max(1)
                    }
                    Key::Right => {
                        self.gaming.level_preview = (self.gaming.level_preview + 1).min(8)
                    }
                    Key::Center => {
                        self.gaming.model.level = self.gaming.level_preview;
                        self.gaming.data.level = self.gaming.model.level;
                        self.gaming.dirty = true;
                        self.gaming.model.discard();
                        self.snake_menu(0);
                        self.gaming.view = SnakeView::Saved;
                    }
                    Key::SoftRight => self.snake_menu(1),
                    _ => {}
                }
                return;
            }
            SnakeView::Scores => {
                if key == Key::SoftRight {
                    self.snake_menu(3);
                }
                return;
            }
            _ => {}
        }
        let n = self.snake_items().len();
        match key {
            Key::Up => self.index = (self.index + n - 1) % n,
            Key::Down => self.index = (self.index + 1) % n,
            Key::SoftRight => match self.gaming.view {
                SnakeView::Menu => self.leave_app(),
                SnakeView::Types => self.snake_menu(2),
                SnakeView::Sound => self.snake_menu(4),
                _ => {}
            },
            Key::Center => match self.gaming.view {
                SnakeView::Types => {
                    self.gaming.model.kind = self.index;
                    self.gaming.data.kind = self.index;
                    self.gaming.dirty = true;
                    // The native picker retains the paused game and this row.
                    self.snake_menu(2);
                }
                SnakeView::Sound => {
                    self.gaming.data.sound = self.index == 1;
                    self.gaming.dirty = true;
                    self.snake_menu(4);
                }
                SnakeView::Menu => {
                    let resume = self.snake_can_resume();
                    if resume && self.index == 0 {
                        self.gaming.model.running = true;
                        self.gaming.view = SnakeView::Game;
                    } else {
                        match self.index - usize::from(resume) {
                            0 => {
                                self.gaming.model.level = self.gaming.data.level;
                                self.gaming.model.kind = self.gaming.data.kind;
                                self.gaming.model.start();
                                self.gaming.view = SnakeView::Game;
                            }
                            1 => {
                                self.gaming.level_preview = self.gaming.data.level;
                                self.gaming.view = SnakeView::Level;
                            }
                            2 => {
                                self.gaming.view = SnakeView::Types;
                                self.index = self.gaming.data.kind;
                            }
                            3 => self.gaming.view = SnakeView::Scores,
                            4 => {
                                self.gaming.view = SnakeView::Sound;
                                self.index = usize::from(self.gaming.data.sound);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
