//! Focus movement only. Callers keep their route, persistence and activation policy.
use phone_core::input::Key;

#[derive(Clone, Copy)]
pub enum Boundary {
    Wrap,
    Clamp,
}

/// Returns whether an Up/Down key was consumed. Empty lists have no selection.
/// Disabled rows remain focusable, matching the sampled native menus.
pub fn vertical(index: &mut usize, count: usize, key: Key, boundary: Boundary) -> bool {
    if !matches!(key, Key::Up | Key::Down) {
        return false;
    }
    if count == 0 {
        *index = 0;
        return true;
    }
    *index = (*index).min(count - 1);
    *index = match (key, boundary) {
        (Key::Up, Boundary::Wrap) => {
            if *index == 0 {
                count - 1
            } else {
                *index - 1
            }
        }
        (Key::Down, Boundary::Wrap) => {
            if *index == count - 1 {
                0
            } else {
                *index + 1
            }
        }
        (Key::Up, Boundary::Clamp) => index.saturating_sub(1),
        _ => (*index + 1).min(count - 1),
    };
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_shrinking_and_disabled_row_focus_is_bounded() {
        let mut index = usize::MAX;
        assert!(!vertical(&mut index, 0, Key::Center, Boundary::Wrap));
        assert!(vertical(&mut index, 0, Key::Down, Boundary::Wrap));
        assert_eq!(index, 0);
        vertical(&mut index, 3, Key::Up, Boundary::Wrap);
        assert_eq!(index, 2);
        vertical(&mut index, 1, Key::Down, Boundary::Wrap);
        assert_eq!(index, 0);
        vertical(&mut index, 3, Key::Up, Boundary::Clamp);
        assert_eq!(index, 0);
        for _ in 0..5 {
            vertical(&mut index, 3, Key::Down, Boundary::Clamp);
        }
        assert_eq!(index, 2);
    }
}
