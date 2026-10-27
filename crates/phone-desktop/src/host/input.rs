use phone_core::input::Key;
use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Debug, PartialEq, Eq)]
pub enum Keyboard {
    Phone(Key),
    Text(char),
}

/// Host keys use fixed US-English physical positions. Never consume logical
/// characters, dead-key composition or text supplied by the system IME.
pub fn key(code: KeyCode, shift: bool) -> Option<Keyboard> {
    use KeyCode::*;
    if shift && matches!(code, Enter | NumpadEnter) {
        return Some(Keyboard::Text('\n'));
    }
    if shift && code == Digit3 {
        return Some(Keyboard::Phone(Key::Hash));
    }
    if shift {
        let symbol = match code {
            Digit1 => Some('!'),
            Digit2 => Some('@'),
            Digit4 => Some('$'),
            Digit5 => Some('%'),
            Digit6 => Some('^'),
            Digit7 => Some('&'),
            Digit8 => Some('*'),
            Digit9 => Some('('),
            Digit0 => Some(')'),
            _ => None,
        };
        if let Some(ch) = symbol {
            return Some(Keyboard::Text(ch));
        }
    }
    if let Some(key) = keyboard(code) {
        return Some(Keyboard::Phone(key));
    }
    let (lower, upper) = match code {
        KeyA => ('a', 'A'),
        KeyB => ('b', 'B'),
        KeyC => ('c', 'C'),
        KeyD => ('d', 'D'),
        KeyE => ('e', 'E'),
        KeyF => ('f', 'F'),
        KeyG => ('g', 'G'),
        KeyH => ('h', 'H'),
        KeyI => ('i', 'I'),
        KeyJ => ('j', 'J'),
        KeyK => ('k', 'K'),
        KeyL => ('l', 'L'),
        KeyM => ('m', 'M'),
        KeyN => ('n', 'N'),
        KeyO => ('o', 'O'),
        KeyP => ('p', 'P'),
        KeyQ => ('q', 'Q'),
        KeyR => ('r', 'R'),
        KeyS => ('s', 'S'),
        KeyT => ('t', 'T'),
        KeyU => ('u', 'U'),
        KeyV => ('v', 'V'),
        KeyW => ('w', 'W'),
        KeyX => ('x', 'X'),
        KeyY => ('y', 'Y'),
        KeyZ => ('z', 'Z'),
        Space => (' ', ' '),
        Minus => ('-', '_'),
        Equal => ('=', '+'),
        BracketLeft => ('[', '{'),
        BracketRight => (']', '}'),
        Backslash => ('\\', '|'),
        Semicolon => (';', ':'),
        Quote => ('\'', '"'),
        Backquote => ('`', '~'),
        Comma => (',', '<'),
        Period => ('.', '>'),
        Slash => ('/', '?'),
        NumpadDecimal => ('.', '.'),
        NumpadDivide => ('/', '/'),
        NumpadAdd => ('+', '+'),
        NumpadSubtract => ('-', '-'),
        NumpadEqual => ('=', '='),
        _ => return None,
    };
    Some(Keyboard::Text(if shift { upper } else { lower }))
}
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
        KeyCode::NumpadHash => Key::Hash,
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
    #[test]
    fn physical_english_keys_do_not_depend_on_host_text_or_layout() {
        use KeyCode::*;
        let letters = [
            KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN,
            KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
        ];
        for (i, code) in letters.into_iter().enumerate() {
            assert_eq!(
                key(code, false),
                Some(Keyboard::Text((b'a' + i as u8) as char))
            );
            assert_eq!(
                key(code, true),
                Some(Keyboard::Text((b'A' + i as u8) as char))
            );
        }
        assert_eq!(key(Quote, false), Some(Keyboard::Text('\'')));
        assert_eq!(key(Quote, true), Some(Keyboard::Text('"')));
        assert_eq!(key(Backquote, false), Some(Keyboard::Text('`')));
        assert_eq!(key(Space, false), Some(Keyboard::Text(' ')));
        assert_eq!(key(Convert, false), None);
        assert_eq!(key(Lang1, false), None);
    }
    #[test]
    fn phone_digits_and_controls_still_reach_internal_input() {
        use KeyCode::*;
        assert_eq!(key(Digit3, true), Some(Keyboard::Phone(Key::Hash)));
        assert_eq!(key(NumpadHash, false), Some(Keyboard::Phone(Key::Hash)));
        assert_eq!(key(F4, false), None);
        assert_eq!(key(F5, false), None);
        assert_eq!(key(Digit6, false), Some(Keyboard::Phone(Key::Digit(6))));
        assert_eq!(key(Numpad6, true), Some(Keyboard::Phone(Key::Digit(6))));
        assert_eq!(key(Digit6, true), Some(Keyboard::Text('^')));
        assert_eq!(key(Enter, false), Some(Keyboard::Phone(Key::Center)));
        assert_eq!(key(Enter, true), Some(Keyboard::Text('\n')));
        assert_eq!(key(ArrowLeft, true), Some(Keyboard::Phone(Key::Left)));
        assert_eq!(key(Backspace, false), Some(Keyboard::Phone(Key::Delete)));
        assert_eq!(key(NumpadMultiply, false), Some(Keyboard::Phone(Key::Star)));
    }
}
