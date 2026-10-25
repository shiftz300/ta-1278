use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SnakeView {
    #[default]
    Menu,
    Game,
    Types,
    Level,
}
pub const GAME_TYPES: [&str; 6] = ["经典", "盒子", "隧道", "磨坊", "轨道", "公寓"];
impl Phone {
    pub fn snake_items(&self) -> Vec<String> {
        match self.snake_view {
            SnakeView::Menu => {
                let mut v = vec![];
                if !self.snake.body.is_empty() && !self.snake.over {
                    v.push("继续".into());
                }
                v.extend(["新游戏", "游戏类型", "等级", "退出"].map(str::to_owned));
                v
            }
            SnakeView::Types => GAME_TYPES.map(str::to_owned).to_vec(),
            _ => vec![],
        }
    }
    pub(super) fn snake_key(&mut self, key: Key) {
        if self.snake_view == SnakeView::Game {
            match key {
                Key::SoftRight => {
                    self.snake.running = false;
                    self.snake_view = SnakeView::Menu;
                    self.index = 0;
                }
                _ => self.snake.key(key),
            }
            return;
        }
        if self.snake_view == SnakeView::Level {
            match key {
                Key::Left => self.snake.level = self.snake.level.saturating_sub(1),
                Key::Right => self.snake.level = (self.snake.level + 1).min(9),
                Key::Center | Key::SoftRight => {
                    self.snake_view = SnakeView::Menu;
                    self.index = 0;
                }
                _ => {}
            }
            return;
        }
        let n = self.snake_items().len();
        match key {
            Key::Up => self.index = (self.index + n - 1) % n,
            Key::Down => self.index = (self.index + 1) % n,
            Key::SoftRight => {
                if self.snake_view == SnakeView::Menu {
                    self.leave_app();
                } else {
                    self.snake_view = SnakeView::Menu;
                    self.index = 0;
                }
            }
            Key::Center => {
                if self.snake_view == SnakeView::Types {
                    self.snake.kind = self.index;
                    self.snake.running = false;
                    self.snake.body.clear();
                    self.snake_view = SnakeView::Menu;
                    self.index = 0;
                } else {
                    let resume = !self.snake.body.is_empty() && !self.snake.over;
                    if resume && self.index == 0 {
                        self.snake.running = true;
                        self.snake_view = SnakeView::Game;
                    } else {
                        match self.index - usize::from(resume) {
                            0 => {
                                self.snake.start();
                                self.snake_view = SnakeView::Game;
                            }
                            1 => {
                                self.snake_view = SnakeView::Types;
                                self.index = self.snake.kind;
                            }
                            2 => self.snake_view = SnakeView::Level,
                            _ => self.leave_app(),
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
