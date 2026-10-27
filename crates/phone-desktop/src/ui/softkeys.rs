//! One three-slot softkey control. Pages supply actions, never baselines or glyphs.
use super::{SOFTKEY_TEXT_Y, SOFTKEY_TOP};
use crate::canvas::{Canvas, MUTED, WHITE};
#[derive(Clone, Copy)]
pub enum Button<'a> {
    None,
    Text(&'a str),
    Disabled(&'a str),
    Menu,
    Back,
    Close,
    Delete,
    Contacts,
    Phone,
    Lap,
    Reset,
    Record,
    Recordings,
    Percent,
    Transfer,
    Shortcut(usize),
}
#[derive(Clone, Copy)]
pub struct Softkeys<'a> {
    pub left: Button<'a>,
    pub center: Button<'a>,
    pub right: Button<'a>,
    accent_panel: bool,
}
impl<'a> Softkeys<'a> {
    pub const fn new(left: Button<'a>, center: Button<'a>, right: Button<'a>) -> Self {
        Self {
            left,
            center,
            right,
            accent_panel: false,
        }
    }
    pub const fn back(label: &'a str) -> Self {
        Self::new(Button::None, Button::Text(label), Button::Back)
    }
    pub const fn menu(label: &'a str) -> Self {
        Self::new(Button::Menu, Button::Text(label), Button::Back)
    }
    pub const fn accent_panel(mut self) -> Self {
        self.accent_panel = true;
        self
    }
    pub fn draw(self, c: &mut Canvas) {
        let top = if self.accent_panel { 282 } else { SOFTKEY_TOP };
        c.rect(
            0,
            top,
            240,
            320 - top,
            if self.accent_panel {
                c.accent(3)
            } else {
                [0, 0, 0, 255]
            },
        );
        for (slot, button) in [self.left, self.center, self.right].into_iter().enumerate() {
            draw_button(c, slot, button);
        }
    }
}
fn draw_button(c: &mut Canvas, slot: usize, button: Button<'_>) {
    let x = [5, 107, 208][slot];
    match button {
        Button::None => {}
        Button::Lap | Button::Reset | Button::Record | Button::Recordings | Button::Percent => {
            use super::art::UtilityIcon;
            let icon = match button {
                Button::Lap => UtilityIcon::Lap,
                Button::Reset => UtilityIcon::Reset,
                Button::Record => UtilityIcon::Record,
                Button::Recordings => UtilityIcon::Recordings,
                _ => UtilityIcon::Percent,
            };
            super::art::utility(c, icon, x, 287);
        }
        Button::Transfer => {
            c.rect(x, 302, 17, 1, WHITE);
            for d in 0..10 {
                c.rect(x + 16 + d, 302 - d, 1, 1, WHITE);
            }
            c.rect(x + 20, 293, 6, 1, WHITE);
            c.rect(x + 25, 293, 1, 7, WHITE);
        }
        Button::Phone => super::art::phone(c, x),
        Button::Text(label) | Button::Disabled(label) => {
            let width = c.width(label, 20);
            let x = match slot {
                0 => 6,
                1 => (240 - width) / 2,
                _ => 234 - width,
            };
            c.text(
                x,
                SOFTKEY_TEXT_Y,
                label,
                20,
                if matches!(button, Button::Disabled(_)) {
                    MUTED
                } else {
                    WHITE
                },
            );
        }
        Button::Menu | Button::Back => c.softkey_symbol(matches!(button, Button::Back), x, 287),
        Button::Shortcut(index) => c.home_shortcut(index, x, 287),
        Button::Close => {
            for d in 0..14 {
                c.rect(x + 7 + d, 291 + d, 2, 2, WHITE);
                c.rect(x + 20 - d, 291 + d, 2, 2, WHITE);
            }
        }
        Button::Delete => {
            // Fitted to the sampled native outline, shared by every editor.
            c.rect(x + 10, 291, 15, 2, WHITE);
            c.rect(x + 10, 306, 15, 2, WHITE);
            c.rect(x + 23, 291, 2, 17, WHITE);
            for d in 0..9 {
                c.rect(x + 2 + d, 299 - d, 2, 2, WHITE);
                c.rect(x + 2 + d, 298 + d, 2, 2, WHITE);
            }
            for d in 0..7 {
                c.rect(x + 12 + d, 296 + d, 1, 1, WHITE);
                c.rect(x + 18 - d, 296 + d, 1, 1, WHITE);
            }
        }
        Button::Contacts => {
            for (dx, y) in [(5, 294), (19, 294), (12, 292)] {
                for dy in -3..=3 {
                    for xx in -3..=3 {
                        if xx * xx + dy * dy <= 9 {
                            c.rect(x + dx + xx, y + dy, 1, 1, WHITE);
                        }
                    }
                }
                for row in 0..6 {
                    let half = (row + 2).min(5);
                    c.rect(x + dx - half, y + 5 + row, half * 2 + 1, 1, WHITE);
                }
            }
        }
    }
}
/// A specialized control shares the bar and baseline, while adding transport keys.
pub fn transport(c: &mut Canvas, playing: bool) {
    Softkeys::menu("").draw(c);
    c.text(76, SOFTKEY_TEXT_Y, "◀", 20, WHITE);
    c.rect(77, 296, 2, 14, WHITE);
    c.text(160, SOFTKEY_TEXT_Y, "▶", 20, WHITE);
    c.rect(178, 296, 2, 14, WHITE);
    if playing {
        c.rect(113, 296, 5, 14, WHITE);
        c.rect(122, 296, 5, 14, WHITE);
    } else {
        c.text(112, SOFTKEY_TEXT_Y, "▶", 20, WHITE);
    }
}
