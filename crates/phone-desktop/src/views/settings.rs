use super::*;
use crate::phone::settings::{SHORTCUTS, SettingsPage};
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    c.text(6, 1, p.settings_title(), 14, WHITE);
    if p.settings_page == SettingsPage::Theme {
        for i in 0..6 {
            let x = i as i32 * 41;
            c.rect(x, 24, 36, 36, c.theme.color(i, 3));
            if i == p.theme_preview {
                c.rect(x, 24, 36, 3, WHITE);
                c.rect(x, 57, 36, 3, WHITE);
                c.rect(x, 24, 3, 36, WHITE);
                c.rect(x + 33, 24, 3, 36, WHITE);
            }
        }
        c.text(6, 68, "预览", 14, WHITE);
        for (i, icon) in [17, 4, 6, 1, 0, 7, 5, 20, 21].iter().enumerate() {
            let x = 38 + (i % 3) as i32 * 81;
            let y = 128 + (i / 3) as i32 * 78;
            if i == 4 {
                c.ring(x, y, 31, c.accent(3));
            }
            c.icon(*icon, x - 38, y - 38, 76);
        }
        c.rect(0, 290, 240, 30, [0, 0, 0, 255]);
        footer(c, "", "←");
        c.text(98, 296, "选择", 20, WHITE);
        return;
    }
    if p.settings_page == SettingsPage::Shortcuts {
        for (i, (label, assigned)) in [
            ("右键", p.preferences.right_shortcut),
            ("左键", p.preferences.left_shortcut),
        ]
        .iter()
        .enumerate()
        {
            let y = 24 + i as i32 * 96;
            if p.index == i {
                c.rect(0, y, 240, 64, c.accent(3));
            }
            c.text(6, y + 8, label, 20, WHITE);
            c.text(6, y + 36, SHORTCUTS[*assigned], 16, MUTED);
        }
    } else {
        let labels = p.settings_labels();
        let start = p.index.saturating_sub(5);
        for (i, label) in labels.iter().enumerate().skip(start).take(6) {
            let y = 24 + (i - start) as i32 * 42;
            if p.index == i {
                c.rect(0, y, 240, 42, c.accent(3));
            }
            c.fit(6, y + 10, label, 20, WHITE, 203);
            if let SettingsPage::Shortcut { left } = p.settings_page {
                let selected = if left {
                    p.preferences.left_shortcut
                } else {
                    p.preferences.right_shortcut
                };
                c.ring(
                    225,
                    y + 21,
                    7,
                    if p.index == i { WHITE } else { c.accent(3) },
                );
                if selected == i {
                    c.rect(
                        222,
                        y + 18,
                        6,
                        6,
                        if p.index == i { WHITE } else { c.accent(3) },
                    );
                }
            }
        }
    }
    footer(c, "", "←");
    c.text(98, 296, "选择", 20, WHITE);
}
