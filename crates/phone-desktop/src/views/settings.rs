use super::*;
use crate::phone::settings::{SHORTCUTS, SettingsPage};
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    if p.settings.page == SettingsPage::Ringtone {
        crate::ui::tones::draw(c, "铃声", p.index, &p.settings.preferences.ringtone, "预设");
        return;
    }
    chrome::title(c, p.settings_title());
    if p.settings.page == SettingsPage::Theme {
        for i in 0..6 {
            let x = i as i32 * 41;
            c.rect(x, 24, 36, 36, c.theme.color(i, 3));
            if i == p.settings.preview {
                c.rect(x, 24, 36, 3, WHITE);
                c.rect(x, 57, 36, 3, WHITE);
                c.rect(x, 24, 3, 36, WHITE);
                c.rect(x + 33, 24, 3, 36, WHITE);
            }
        }
        c.text(6, 68, "预览", 14, WHITE);
        for (i, icon) in [17, 4, 6, 1, 0, 7, 5, 18, 19].iter().enumerate() {
            let x = 38 + (i % 3) as i32 * 81;
            let y = 128 + (i / 3) as i32 * 78;
            if i == 4 {
                c.ring(x, y, 31, c.accent(3));
            }
            c.icon(*icon, x - 38, y - 38, 76);
        }
        c.rect(0, 290, 240, 30, [0, 0, 0, 255]);
        Softkeys::back("选择").draw(c);
        return;
    }
    if p.settings.page == SettingsPage::Shortcuts {
        for (i, (label, assigned)) in [
            ("右键", p.settings.preferences.right_shortcut),
            ("左键", p.settings.preferences.left_shortcut),
        ]
        .iter()
        .enumerate()
        {
            let y = 24 + i as i32 * 64;
            if p.index == i {
                c.rect(0, y, 240, 64, c.accent(3));
            }
            c.text(6, y + 8, label, 20, WHITE);
            c.text(6, y + 36, SHORTCUTS[*assigned], 16, MUTED);
        }
    } else {
        let mut labels = p.settings_labels();
        use SettingsPage::*;
        match p.settings.page {
            Connections => {
                labels[0] = format!(
                    "移动数据  {}",
                    if p.settings.preferences.mobile_data {
                        "开"
                    } else {
                        "关"
                    }
                )
            }
            Sounds => {
                labels[0] = format!(
                    "静音  {}",
                    if p.settings.preferences.silent {
                        "开"
                    } else {
                        "关"
                    }
                );
                labels[1] = format!("铃声音量  {}", p.settings.preferences.volume);
                labels[3] = format!(
                    "按键音  {}",
                    if p.settings.preferences.keypad_sound {
                        "开"
                    } else {
                        "关"
                    }
                );
            }
            Time => {
                labels[2] = format!("时区  UTC{:+}", p.settings.preferences.timezone);
                labels[3] = format!(
                    "时间格式  {}小时",
                    if p.settings.preferences.clock_12h {
                        12
                    } else {
                        24
                    }
                );
            }
            _ => {}
        }
        list::draw(
            c,
            labels.len(),
            p.index,
            list::ListStyle {
                first: matches!(p.settings.page, SettingsPage::Shortcut { .. })
                    .then_some(p.settings.first),
                text_width: 203,
                ..list::ListStyle::STANDARD
            },
            |i| {
                let marker = if let SettingsPage::Shortcut { left } = p.settings.page {
                    list::Marker::Radio(
                        i == if left {
                            p.settings.preferences.left_shortcut
                        } else {
                            p.settings.preferences.right_shortcut
                        },
                    )
                } else {
                    list::Marker::None
                };
                list::Row {
                    label: &labels[i],
                    marker,
                    enabled: p.setting_enabled(i),
                }
            },
        );
    }
    Softkeys::back("选择").draw(c);
}
