//! State owned by this application.
use super::*;

pub struct State {
    pub model: phone_core::snake::Snake,
    pub view: snake::SnakeView,
    pub level_preview: u8,
    pub data: snake::SnakeData,
    pub dirty: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            model: Default::default(),
            view: Default::default(),
            level_preview: 1,
            data: Default::default(),
            dirty: false,
        }
    }
}
