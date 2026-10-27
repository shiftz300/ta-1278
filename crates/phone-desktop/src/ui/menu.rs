//! One declaration for menu text, availability, marker and typed action.
//! Produces intent only: rendering and input never execute application effects.
use super::{
    list::{Marker, Row},
    navigation::{self, Boundary},
};
use phone_core::input::Key;

pub struct Item<'a, A> {
    pub action: A,
    pub label: &'a str,
    pub enabled: bool,
    pub marker: Marker,
}
impl<'a, A> Item<'a, A> {
    pub const fn new(action: A, label: &'a str) -> Self {
        Self {
            action,
            label,
            enabled: true,
            marker: Marker::None,
        }
    }
    pub const fn child(mut self) -> Self {
        self.marker = Marker::Child;
        self
    }
    pub fn row(&self) -> Row<'a> {
        Row {
            label: self.label,
            enabled: self.enabled,
            marker: self.marker,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum Event<A> {
    Activate(A),
    Back,
    Focus,
    Ignored,
}
/// Ordinary popup semantics: Center activates, Right cancels; Left is not implicit OK.
pub fn event<A: Copy>(items: &[Item<'_, A>], index: &mut usize, key: Key) -> Event<A> {
    if navigation::vertical(index, items.len(), key, Boundary::Wrap) {
        return Event::Focus;
    }
    match key {
        Key::SoftRight => Event::Back,
        Key::Center => items
            .get(*index)
            .filter(|item| item.enabled)
            .map_or(Event::Ignored, |item| Event::Activate(item.action)),
        _ => Event::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_items_are_focusable_but_cannot_activate_and_back_works_when_empty() {
        let items = [
            Item::new(10, "Open"),
            Item {
                enabled: false,
                ..Item::new(20, "Disabled")
            },
        ];
        let mut index = 0;
        assert_eq!(event(&items, &mut index, Key::Down), Event::Focus);
        assert_eq!(index, 1);
        assert_eq!(event(&items, &mut index, Key::Center), Event::Ignored);
        assert!(!items[index].row().enabled);
        assert_eq!(event(&items, &mut index, Key::Down), Event::Focus);
        assert_eq!(event(&items, &mut index, Key::SoftLeft), Event::Ignored);
        assert_eq!(event(&items, &mut index, Key::Center), Event::Activate(10));
        index = 99;
        assert_eq!(event(&items, &mut index, Key::Center), Event::Ignored);
        assert_eq!(event::<u8>(&[], &mut index, Key::SoftRight), Event::Back);
    }
}
