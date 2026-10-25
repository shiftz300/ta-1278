use super::*;
use crate::phone::{
    alarm::{self, tables},
    organizer::OrganizerPage,
};
const BLACK: Color = [0, 0, 0, 255];

pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    use OrganizerPage::*;
    c.text(
        6,
        1,
        if p.organizer.page == AlarmMenu {
            "闹钟"
        } else {
            p.organizer_title()
        },
        14,
        WHITE,
    );
    let mut left = "";
    let mut center = "选择";
    let mut right = "←";
    match p.organizer.page {
        Alarms => {
            (left, center) = root(c, p, p.index);
        }
        AlarmForm => {
            let offset = ((p.index + 1) as i32 * 69 - 257).max(0);
            let labels = ["设置时间", "重复播放", "闹钟名称", "闹钟铃声", "重响时间"];
            for (i, label) in labels.iter().enumerate() {
                let y = 24 + i as i32 * 69 - offset;
                if y + 69 <= 24 || y >= 281 {
                    continue;
                }
                let selected = p.index == i;
                if selected && !matches!(i, 0 | 2) {
                    c.rect(
                        0,
                        y.max(24),
                        235,
                        (y + 64).min(281) - y.max(24),
                        c.accent(3),
                    );
                }
                if y + 8 >= 24 {
                    c.text(
                        7,
                        y + 8,
                        label,
                        14,
                        if selected && matches!(i, 0 | 2) {
                            c.accent(3)
                        } else {
                            MUTED
                        },
                    );
                }
                if y + 34 >= 24 {
                    if i == 2 && selected {
                        p.editor_lines =
                            editor_sized(c, &p.organizer.alarm.name, 7, y + 34, 224, 26, 20, 26);
                        c.text(215, y + 8, mode(&p.organizer.alarm.name), 14, MUTED);
                    } else {
                        if i == 0 && selected {
                            let at = [0, 1, 3, 4][p.organizer.alarm.digit];
                            let value = &p.organizer.draft[0];
                            if let (Some(prefix), Some(digit)) =
                                (value.get(..at), value.get(at..at + 1))
                            {
                                let x = 7 + c.width(prefix, 16);
                                let w = c.width(digit, 16);
                                c.rect(x, y + 35, w, 26, WHITE);
                            }
                        }
                        let value = if i == 2 {
                            p.organizer.alarm.name.text()
                        } else if i == 1 {
                            // Single Monday renders as 一 on the captured form.
                            // Combined-day punctuation remains a replica presentation choice.
                            p.organizer.draft[i].replace("星期", "")
                        } else {
                            p.organizer.draft[i].clone()
                        };
                        c.fit(
                            7,
                            y + if i == 0 { 40 } else { 34 },
                            &value,
                            if i == 0 || i == 3 { 16 } else { 20 },
                            WHITE,
                            224,
                        );
                        if i == 0 && selected {
                            let at = [0, 1, 3, 4][p.organizer.alarm.digit];
                            if let (Some(prefix), Some(digit)) =
                                (value.get(..at), value.get(at..at + 1))
                            {
                                let x = 7 + c.width(prefix, 16);
                                c.text(x, y + 40, digit, 16, BLACK);
                            }
                        }
                    }
                }
                if matches!(i, 0 | 2) && y + 60 >= 24 {
                    c.rect(
                        7,
                        y + if i == 2 { 53 } else { 60 },
                        228,
                        3,
                        if selected { c.accent(3) } else { MUTED },
                    );
                }
            }
            c.rect(235, 25, 4, 256, [35, 40, 37, 255]);
            c.rect(235, 25 + offset * 256 / 345, 4, 190, MUTED);
            left = "≡";
            if matches!(p.index, 0 | 2) {
                center = "保存";
            }
            if p.index == 2 && !p.organizer.alarm.name.units.is_empty() {
                right = "⌫";
            }
        }
        AlarmDays => {
            let start = p.index.saturating_sub(5);
            for i in start..(start + 6).min(7) {
                let y = 24 + (i - start) as i32 * 42;
                if i == p.index {
                    c.rect(0, y, 240, 42, c.accent(3));
                }
                let selected = p.organizer.alarm.days & tables().weekday_bits[i] != 0;
                if selected {
                    c.rect(6, y + 12, 18, 18, c.accent(6));
                    for d in 0..4 {
                        c.rect(9 + d, y + 20 + d, 2, 2, WHITE);
                    }
                    for d in 0..7 {
                        c.rect(12 + d, y + 23 - d, 2, 2, WHITE);
                    }
                } else {
                    c.rect(6, y + 12, 18, 18, WHITE);
                    c.rect(8, y + 14, 14, 14, BLACK);
                }
                c.text(36, y + 10, &tables().weekday_labels[i], 20, WHITE);
            }
            if p.organizer.alarm.days & tables().weekday_bits[p.index] != 0 {
                center = "取消选择";
            }
        }
        AlarmSnooze => {
            super::tools::native_list(c, &tables().snooze_labels, p.index);
            let selected = alarm::snooze_index(&p.organizer.draft[4]);
            for i in 0..5 {
                let y = 45 + i as i32 * 42;
                let color = c.accent(if i == p.index { 6 } else { 3 });
                c.ring(225, y, 7, color);
                if i == selected {
                    c.rect(222, y - 3, 6, 6, color);
                }
            }
        }
        AlarmMenu => {
            root(c, p, p.organizer.selected + 1);
            c.dim();
            popup(c, &["编辑", "删除", "全部删除"], p.index);
        }
        AlarmFormMenu => popup(c, &["保存"], 0),
        DeleteAllAlarms => {
            c.text(24, 220, "删除全部闹钟？", 20, WHITE);
            center = "确认";
        }
        _ => {}
    }
    c.rect(0, 282, 240, 38, BLACK);
    super::softkeys::icons(c, left == "≡", false);
    if right == "⌫" {
        c.rect(208, 284, 32, 36, BLACK);
        c.text(211, 290, "⌫", 22, WHITE);
    }
    let width = c.width(center, 20);
    c.text((240 - width) / 2, 290, center, 20, WHITE);
}

pub(super) fn alert(c: &mut Canvas, p: &Phone) {
    if let Some(a) = p.organizer.alarm.alerts.front() {
        c.caret = None;
        c.background();
        status(c);
        c.text(6, 1, "闹钟", 14, WHITE);
        let w = c.width(&a.time, 40);
        c.text((240 - w) / 2, 100, &a.time, 40, WHITE);
        c.fit(12, 164, &a.name, 24, WHITE, 216);
        footer(c, "贪睡", "停止");
        c.text(100, 296, "停止", 20, WHITE);
    }
}

fn root(c: &mut Canvas, p: &Phone, index: usize) -> (&'static str, &'static str) {
    let first = index.saturating_sub(3);
    if first == 0 {
        c.rect(1, 25, 238, 34, c.accent(if index == 0 { 3 } else { 6 }));
        c.text(69, 32, "+ 新建闹钟", 20, WHITE);
    }
    for (i, a) in p.local_data.alarms.iter().enumerate().skip(first).take(4) {
        let y = 64 + (i - first) as i32 * 69;
        if i + 1 == index {
            c.rect(0, y, 240, 64, c.accent(3));
        }
        c.fit(
            6,
            y + 9,
            &format!("{} | {}", a.fields[0], a.fields[2]),
            20,
            WHITE,
            190,
        );
        c.fit(
            7,
            y + 37,
            &alarm::repeat_label(alarm::repeat_mask(&a.fields[1])),
            18,
            MUTED,
            188,
        );
        c.ring(216, y + 32, 10, if a.enabled { WHITE } else { MUTED });
        c.rect(
            if a.enabled { 217 } else { 206 },
            y + 26,
            9,
            12,
            if a.enabled { WHITE } else { MUTED },
        );
    }
    if let Some(a) = index
        .checked_sub(1)
        .and_then(|i| p.local_data.alarms.get(i))
    {
        return ("≡", if a.enabled { "关闭" } else { "打开" });
    }

    ("", "选择")
}

fn popup(c: &mut Canvas, labels: &[&str], index: usize) {
    let top = 276 - labels.len() as i32 * 38;
    c.rect(0, top, 240, labels.len() as i32 * 38, BLACK);
    for (i, label) in labels.iter().enumerate() {
        let y = top + i as i32 * 38;
        if i == index {
            c.rect(0, y, 240, 38, c.accent(3));
        }
        c.text(6, y + 7, label, 20, WHITE);
    }
}
