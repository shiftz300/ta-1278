use super::*;
use crate::shell::tools::ToolPage;

pub(super) fn render(c: &mut Canvas, p: &Shell) {
    if p.tools.page == ToolPage::Dial {
        chrome::title(c, "呼叫");
        c.rect(0, 24, 240, 46, c.accent(3));
        c.clipped([0, 24, 240, 46], |c| {
            Edit::new([6, 28, 228, 36])
                .sized(30, 36)
                .draw(c, &p.calls.number);
        });
        let first = p.calls.selected.unwrap_or(0).saturating_sub(3);
        for (i, contact) in p.contacts.records.iter().enumerate().skip(first).take(4) {
            let y = 76 + (i - first) as i32 * 52;
            if p.calls.selected == Some(i) {
                c.rect(0, y, 240, 52, c.accent(3));
            }
            let name = crate::shell::contacts::contact_name(contact);
            crate::ui::form::avatar(c, 7, y + 1, name.chars().next(), [90, 95, 90, 255]);
            c.fit(52, y + 10, &name, 20, WHITE, 180);
        }
        Softkeys::new(
            Button::Menu,
            if super::super::shell::calls::valid(&p.dial_number()) {
                Button::Text("呼叫")
            } else {
                Button::Disabled("呼叫")
            },
            if p.calls.number.units.is_empty() {
                Button::Back
            } else {
                Button::Delete
            },
        )
        .draw(c);
        return;
    }
    chrome::title(c, "通话历史记录");
    let labels: Vec<_> = p
        .calls
        .records
        .iter()
        .map(|r| format!("{}  未拨出", r.number))
        .collect();
    if labels.is_empty() {
        c.text(8, 120, "没有通话记录", 20, WHITE);
    } else {
        list::plain(c, &labels, p.index);
    }
    Softkeys::new(
        Button::Text("拨号"),
        if labels.is_empty() {
            Button::None
        } else {
            Button::Text("打开")
        },
        Button::Back,
    )
    .draw(c);
    if p.calls.confirm_delete {
        c.dim();
        c.text(20, 140, "删除此记录？", 20, WHITE);
        Softkeys::new(Button::None, Button::Text("删除"), Button::Back).draw(c);
    }
}
