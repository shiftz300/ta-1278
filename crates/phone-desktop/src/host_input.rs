use phone_core::input::Key;
use winit::{event::MouseButton, keyboard::KeyCode};
pub fn mouse(button: MouseButton) -> Option<(u32, Key)> {
    match button {
        MouseButton::Left => Some((1000, Key::SoftLeft)),
        MouseButton::Right => Some((1001, Key::SoftRight)),
        MouseButton::Middle => Some((1002, Key::Center)),
        _ => None,
    }
}
pub fn keyboard(code: KeyCode) -> Option<Key> {
    Some(match code {
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Center,
        KeyCode::F1 => Key::SoftLeft,
        KeyCode::F2 | KeyCode::Escape => Key::SoftRight,
        KeyCode::F3 => Key::Call,
        KeyCode::Home => Key::End,
        KeyCode::Backspace | KeyCode::Delete => Key::Delete,
        KeyCode::Digit0 | KeyCode::Numpad0 => Key::Digit(0),
        KeyCode::Digit1 | KeyCode::Numpad1 => Key::Digit(1),
        KeyCode::Digit2 | KeyCode::Numpad2 => Key::Digit(2),
        KeyCode::Digit3 | KeyCode::Numpad3 => Key::Digit(3),
        KeyCode::Digit4 | KeyCode::Numpad4 => Key::Digit(4),
        KeyCode::Digit5 | KeyCode::Numpad5 => Key::Digit(5),
        KeyCode::Digit6 | KeyCode::Numpad6 => Key::Digit(6),
        KeyCode::Digit7 | KeyCode::Numpad7 => Key::Digit(7),
        KeyCode::Digit8 | KeyCode::Numpad8 => Key::Digit(8),
        KeyCode::Digit9 | KeyCode::Numpad9 => Key::Digit(9),
        KeyCode::NumpadMultiply => Key::Star,
        _ => return None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn button_mapping() {
        assert_eq!(mouse(MouseButton::Left), Some((1000, Key::SoftLeft)));
        assert_eq!(mouse(MouseButton::Right), Some((1001, Key::SoftRight)));
        assert_eq!(mouse(MouseButton::Middle), Some((1002, Key::Center)));
    }
}
