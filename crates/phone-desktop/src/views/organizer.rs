use super::*;
use crate::phone::organizer::OrganizerPage;
pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    use OrganizerPage::*;
    if crate::phone::alarm::is_alarm(p.organizer.page) {
        super::alarm::render(c, p);
        return;
    }
    let page = p.organizer.page;
    if page == OrganizerPage::EventRingtone {
        crate::ui::tones::draw(c, "铃声", p.index, &p.organizer.draft[4], "默认");
        return;
    }
    if crate::phone::contacts::is_contact(page) {
        super::contacts::render(c, p);
        return;
    }
    chrome::title(c, p.organizer_title());
    let mut left = Button::None;
    let mut center = "选择";
    if page == Calendar {
        calendar(c, p);
        left = Button::Menu;
        center = "打开";
    } else if matches!(page, EventForm | EventFormMenu) {
        event_form(c, p);
        if page == EventFormMenu {
            PopupMenu::default().draw(c, 1, 0, |_| list::Row::text("储存"));
        }
        return;
    } else if page == CalendarMenu {
        calendar(c, p);
        let items = p.organizer_items();
        PopupMenu::default().draw(c, items.len(), p.index, |i| list::Row::text(&items[i]));
        return;
    } else if matches!(page, DeleteContact | DeleteEvent | DeleteAlarm | DeletePast) {
        c.text(
            16,
            180,
            if page == DeletePast {
                "删除今天之前的项目？"
            } else {
                "删除此项目？"
            },
            18,
            WHITE,
        );
        center = "确认";
    } else {
        let items = p.organizer_items();
        list::plain(c, &items, p.index);
        left = match page {
            Contacts | Alarms | Events => Button::Text("新增"),
            EventForm | AlarmForm | ContactForm => Button::Text("保存"),
            ContactDetail => Button::Menu,
            _ => Button::None,
        };
    }
    Softkeys::new(left, Button::Text(center), Button::Back).draw(c);
}

fn calendar(c: &mut Canvas, p: &Phone) {
    let date = p.organizer.date;
    c.rect(0, 0, 100, 24, [0, 0, 0, 255]);
    chrome::title(c, "月份");
    c.text(
        6,
        32,
        &format!("{:04}.{:02}", date.year, date.month),
        20,
        c.accent(3),
    );
    // The approved January 1 capture shows this caption. Other holiday tables
    // have not been recovered, so no lunar or regional dates are inferred.
    if date.month == 1 && date.day == 1 {
        let width = c.width("元旦", 16);
        c.text(234 - width, 24, "元旦", 16, WHITE);
    }
    for (i, label) in ["一", "二", "三", "四", "五", "六", "日"]
        .iter()
        .enumerate()
    {
        c.text(10 + i as i32 * 34, 67, label, 16, c.accent(3));
    }
    let first = phone_core::calendar::Date { day: 1, ..date };
    let start = first.next(-(first.weekday() as i32));
    for cell in 0..42 {
        let d = start.next(cell);
        let x = cell % 7 * 34 + 17;
        let y = 106 + cell / 7 * 32;
        if d == date {
            for dy in -15i32..=15 {
                for dx in -15i32..=15 {
                    if dx * dx + dy * dy <= 225 {
                        c.rect(x + dx, y + dy, 1, 1, c.accent(3));
                    }
                }
            }
        }
        let text = d.day.to_string();
        let text_x = x - c.width(&text, 14) / 2;
        c.text(
            text_x,
            y - 9,
            &text,
            14,
            if d.month == date.month { WHITE } else { MUTED },
        );
        if p.local_data.events.iter().any(|e| e.occurs(d)) {
            c.rect(x - 2, y + 11, 4, 2, WHITE);
        }
    }
}
fn event_form(c: &mut Canvas, p: &mut Phone) {
    use crate::ui::form::{Field, FieldKind};
    chrome::title(c, "新建事件");
    let focus = if p.organizer.page == OrganizerPage::EventFormMenu {
        p.organizer.event.return_index
    } else {
        p.index
    };
    let rows = [
        ("主题", 24, 70),
        ("时间", 94, 70),
        ("提醒", 164, 60),
        ("位置", 224, 70),
        ("铃声", 294, 60),
        ("重复", 354, 60),
        ("备注", 414, 70),
    ];
    let (_, top, height) = rows[focus.min(6)];
    let offset = crate::ui::form::scroll_to(top, height, 281);
    c.clipped([0, 24, 235, 257], |c| {
        for (i, (label, top, height)) in rows.into_iter().enumerate() {
            let y = top - offset;
            if y + height <= 24 || y >= 281 {
                continue;
            }
            let editable = matches!(i, 0 | 3 | 6);
            let active = focus == i && p.organizer.page == OrganizerPage::EventForm;
            let raw = p.organizer.draft.get(i).map(String::as_str).unwrap_or("");
            let value = if i == 1 {
                p.event_datetime()
            } else if i == 2 && raw.ends_with("分钟前") {
                format!("提前{}分钟", raw.trim_end_matches("分钟前"))
            } else if i == 5 && raw == "不重复" {
                "不必重复".into()
            } else {
                raw.to_owned()
            };
            Field {
                label,
                value: if i == 1 || (editable && active) {
                    ""
                } else {
                    &value
                },
                placeholder: if editable && active && !p.organizer.event.input.units.is_empty() {
                    ""
                } else {
                    match i {
                        0 => "添加标题",
                        3 => "添加位置",
                        6 => "添加文字",
                        _ => "",
                    }
                },
                focused: focus == i,
                kind: if matches!(i, 2 | 4 | 5) {
                    FieldKind::Choice
                } else if i == 1 {
                    FieldKind::Date
                } else {
                    FieldKind::Input
                },
            }
            .draw_sized(c, y, height, true);
            if editable && active {
                p.editor_lines = Edit::new([6, y + 33, 228, 26]).draw(c, &p.organizer.event.input);
                c.text(201, y + 8, mode(&p.organizer.event.input), 16, MUTED);
            }
            if i == 1 {
                crate::ui::form::datetime(
                    c,
                    &p.organizer.event.date.iso(),
                    &p.organizer.draft[1],
                    y + 33,
                    active.then_some(p.organizer.event.digit),
                );
            }
        }
    });
    c.rect(235, 24, 4, 257, c.accent(6));
    c.rect(235, 24 + offset * 257 / 460, 4, 257 * 257 / 460, MUTED);
    Softkeys::new(
        Button::Menu,
        Button::Text(if matches!(focus, 2 | 4 | 5) {
            "选择"
        } else {
            "储存"
        }),
        if p.event_editor_active() && !p.organizer.event.input.units.is_empty() {
            Button::Delete
        } else {
            Button::Back
        },
    )
    .draw(c);
}
