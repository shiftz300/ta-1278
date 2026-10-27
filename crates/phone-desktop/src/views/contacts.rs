//! Handset-shaped phonebook views using shared edit, form, list and softkey controls.
use super::*;
use crate::phone::{
    contacts::{DETAIL_MENU, ROOT_MENU},
    organizer::{OrganizerPage as Page, contact_name},
};
use crate::ui::form::{self, Field, FieldKind};
const BLACK: Color = [0, 0, 0, 255];
pub(super) fn render(c: &mut Canvas, p: &Phone) {
    match p.organizer.page {
        Page::ContactGroupMenu => {
            chrome::title(c, "选项");
            PopupMenu::default()
                .unshaded()
                .draw(c, 3, p.index, |i| list::Row {
                    label: ["新增组", "重命名", "删除组"][i],
                    enabled: i == 0 || p.contacts.ui.selected_group.is_some(),
                    marker: list::Marker::None,
                });
        }
        Page::ContactGroupDelete => {
            chrome::title(c, "通话组");
            c.text(10, 120, "是否删除此通话组？", 20, WHITE);
            c.text(10, 155, "联系人将保留。", 18, MUTED);
            Softkeys::new(Button::None, Button::Text("确认"), Button::Close).draw(c);
        }
        Page::ContactRingtone => {
            crate::ui::tones::draw(c, "铃声", p.index, &p.organizer.draft[7], "预设")
        }
        Page::ContactGroups | Page::ContactGroupChoice => {
            chrome::title(c, p.organizer_title());
            let labels = p.contact_choices();
            list::plain(c, &labels, p.index);
            Softkeys::new(
                if p.organizer.page == Page::ContactGroups {
                    Button::Menu
                } else {
                    Button::None
                },
                Button::Text("选择"),
                Button::Back,
            )
            .draw(c);
        }
        Page::Contacts => root(c, p),
        Page::ContactsMenu => {
            root(c, p);
            PopupMenu::default().draw(c, ROOT_MENU.len(), p.index, |i| list::Row {
                label: ROOT_MENU[i],
                enabled: p.contact_menu_enabled(i),
                marker: list::Marker::None,
            });
        }
        Page::ContactForm => form(c, p, p.index, true),
        Page::ContactFormMenu => {
            form(c, p, p.contacts.ui.return_index, false);
            PopupMenu::default().draw(c, 1, 0, |_| list::Row::text("保存"));
        }
        Page::ContactDetail => detail(c, p, p.index),
        Page::ContactMenu => {
            detail(c, p, p.contacts.ui.return_index);
            PopupMenu::default().draw(c, DETAIL_MENU.len(), p.index, |i| list::Row {
                label: DETAIL_MENU[i],
                enabled: p.contact_menu_enabled(i),
                marker: list::Marker::None,
            });
        }
        Page::DeleteContact => {
            detail(c, p, 0);
            c.dim();
            c.text(30, 241, "是否删除此联系人？", 20, WHITE);
            Softkeys::new(Button::None, Button::Text("确认"), Button::Close).draw(c);
        }
        _ => {}
    }
}
fn root(c: &mut Canvas, p: &Phone) {
    chrome::title(c, "通讯录");
    if !p.contacts.ui.query.units.is_empty() {
        c.layout.editor = Some(Edit::new([6, 27, 228, 26]).draw(c, &p.contacts.ui.query));
    }
    c.rect(0, 57, 240, 3, MUTED);
    let ids = p.contact_ids();
    let selected = if p.organizer.page == Page::ContactsMenu {
        p.contacts.ui.return_index
    } else {
        p.index
    }
    .min(ids.len().saturating_sub(1));
    let names: Vec<_> = ids
        .iter()
        .map(|&i| contact_name(&p.contacts.records[i]))
        .collect();
    list::draw(
        c,
        names.len(),
        selected,
        list::ListStyle {
            top: 66,
            row_height: 52,
            visible: 4,
            inset: 52,
            text_y: 16,
            font: 16,
            text_width: 178,
            ..list::ListStyle::STANDARD
        },
        |i| list::Row::text(&names[i]),
    );
    let first = selected.saturating_sub(3);
    for (i, name) in names.iter().enumerate().skip(first).take(4) {
        form::avatar(
            c,
            7,
            72 + (i - first) as i32 * 52,
            name.chars().next(),
            [90, 95, 90, 255],
        );
        contact_picture(
            c,
            p,
            &p.contacts.records[ids[i]][0],
            7,
            72 + (i - first) as i32 * 52,
        );
    }
    // The handset empty-store caption has not been sampled: keep the list empty.
    Softkeys::menu(if ids.is_empty() { "" } else { "打开" }).draw(c);
}
fn form(c: &mut Canvas, p: &Phone, index: usize, active: bool) {
    chrome::title(c, "新联系人");
    // Mixed edit/date/list children have distinct native row metrics.
    const TOPS: [i32; 9] = [24, 86, 154, 222, 290, 358, 426, 486, 550];
    let top = TOPS[index.min(8)];
    let scroll = form::scroll_to(
        top,
        if index >= 7 { 68 } else { 56 },
        if index >= 7 {
            280
        } else if index == 6 {
            272
        } else {
            270
        },
    );
    form::avatar(c, 7, 34 - scroll, None, [90, 95, 90, 255]);
    if let Some(path) = p.organizer.draft.first() {
        contact_picture(c, p, path, 7, 34 - scroll);
    }
    c.text(
        52,
        43 - scroll,
        "添加图片",
        20,
        if index == 0 { WHITE } else { MUTED },
    );
    for (i, top) in TOPS.iter().enumerate().skip(1) {
        let y = top - scroll;
        if y + 68 < 24 || y >= 282 {
            continue;
        }
        let label = crate::phone::organizer::CONTACT_FIELDS[i];
        let value = p.organizer.draft.get(i).map(String::as_str).unwrap_or("");
        let placeholder = if i <= 2 {
            "添加名称"
        } else if i <= 5 {
            "添加号码"
        } else {
            ""
        };
        let rect = Field {
            label,
            value: if i == index && active && (1..=6).contains(&i) {
                ""
            } else {
                value
            },
            placeholder,
            focused: i == index,
            kind: if i >= 7 {
                FieldKind::Choice
            } else if i == 6 {
                FieldKind::Date
            } else {
                FieldKind::Input
            },
        }
        .draw(c, y);
        if i == index && active && (1..=5).contains(&i) {
            // Clear the placeholder before the common edit control paints this field.
            c.rect(6, y + 25, 228, 26, BLACK);
            if p.contacts.ui.input.units.is_empty() {
                c.text(6, y + 27, placeholder, 20, MUTED);
                c.rect(6, y + 28, 1, 23, WHITE);
                c.caret = Some((6, y + 27, 26));
            } else {
                c.layout.editor = Some(Edit::new(rect).draw(c, &p.contacts.ui.input));
            }
            crate::ui::editor::input_mode(c, &p.contacts.ui.input, 228, y + 4);
        }
    }
    if index == 6 && active {
        let y = TOPS[6] - scroll;
        form::date(c, &p.organizer.draft[6], y + 25, p.contacts.ui.date_digit);
    }
    c.rect(236, 28, 4, 248, [40, 45, 40, 255]);
    let thumb = 248 * 248 / 606;
    c.rect(236, 28 + scroll * 248 / 606, 4, thumb, MUTED);
    c.rect(0, 0, 240, 24, BLACK);
    status(c);
    chrome::title(c, "新联系人");
    Softkeys::new(
        Button::Menu,
        if active && p.contacts.ui.input.prediction.active() {
            Button::Text("选择")
        } else {
            Button::None
        },
        if active
            && (1..=5).contains(&index)
            && (!p.contacts.ui.input.units.is_empty() || p.contacts.ui.input.prediction.active())
        {
            Button::Delete
        } else {
            Button::Back
        },
    )
    .draw(c);
}
fn detail(c: &mut Canvas, p: &Phone, index: usize) {
    let name = p.selected_contact_name();
    chrome::title(c, &name);
    form::avatar(c, 7, 31, name.chars().next(), [90, 95, 90, 255]);
    if let Some(row) = p.contacts.records.get(p.organizer.selected) {
        contact_picture(c, p, &row[0], 7, 31);
    }
    c.fit(52, 41, &name, 16, WHITE, 178);
    let numbers = p.contact_numbers();
    let labels: Vec<_> = numbers
        .iter()
        .flat_map(|(field, number)| {
            let kind = ["手机", "住宅", "单位"][field - 3];
            [
                (format!("呼叫({kind})"), number.clone()),
                (format!("发送信息({kind})"), number.clone()),
            ]
        })
        .collect();
    let start = index.saturating_sub(1);
    for (i, (label, number)) in labels.iter().enumerate().skip(start).take(2) {
        let y = 76 + (i - start) as i32 * 68;
        if i == index {
            c.rect(0, y, 240, 68, c.accent(3));
        }
        let color = WHITE;
        c.text(6, y + 7, label, 16, color);
        c.text(6, y + 32, number, 16, color);
    }
    Softkeys::new(
        Button::Menu,
        if labels.is_empty() {
            Button::None
        } else if index.is_multiple_of(2) {
            Button::Text("呼叫")
        } else {
            Button::Text("发送信息")
        },
        Button::Back,
    )
    .draw(c);
}
fn contact_picture(c: &mut Canvas, p: &Phone, path: &str, x: i32, y: i32) {
    if let Some(img) = p.media.pictures.get(path) {
        let img = image::imageops::thumbnail(img, 40, 40);
        c.image(
            &img,
            x + (40 - img.width() as i32) / 2,
            y + (40 - img.height() as i32) / 2,
        );
    }
}
