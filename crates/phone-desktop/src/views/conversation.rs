//! Native conversation structure with explicitly local ordering/delivery policies.
use super::*;
use crate::phone::conversation::{Popup, number};
const BLACK: Color = [0, 0, 0, 255];
fn stamp(time: u64, short: bool) -> String {
    if time == 0 || time > 4_102_444_800_000 {
        return String::new();
    }
    let seconds = time / 1000;
    let d = phone_core::calendar::Date::from_unix(seconds, 8 * 3600);
    let h = (seconds / 3600 + 8) % 24;
    let m = seconds / 60 % 60;
    if short {
        format!("{h:02}:{m:02}")
    } else {
        format!("{:02}/{:02} {h:02}:{m:02}", d.month, d.day)
    }
}
pub(super) fn root(c: &mut Canvas, p: &Phone) {
    chrome::title(c, "信息");
    let threads = p.mailbox.conversations();
    let start = p.index.saturating_sub(3);
    let mut y = 24;
    for i in start..=(start + 4).min(threads.len()) {
        let height = if i == 0 { 46 } else { 64 };
        if y + height > 282 {
            break;
        }
        if i == p.index {
            c.rect(0, y, 240, height, c.accent(3));
        }
        if i == 0 {
            c.text(69, y + 12, "+ 新建信息", 20, WHITE);
        } else {
            let thread = &threads[i - 1];
            let draft = thread.drafts.last().map(|&i| &p.mailbox.drafts[i]);
            let last = thread.items.last().and_then(|&i| p.mailbox.content(i));
            c.fit(
                6,
                y + 8,
                if thread.peer.is_empty() {
                    "(没有收件人)"
                } else {
                    &thread.peer
                },
                18,
                WHITE,
                165,
            );
            c.fit(
                177,
                y + 11,
                if draft.is_some() { "草稿" } else { "" },
                14,
                WHITE,
                61,
            );
            if draft.is_none() {
                c.fit(177, y + 11, &stamp(thread.time, true), 14, WHITE, 61);
            }
            let preview = draft
                .map(|m| m.body.as_str())
                .or_else(|| last.as_ref().map(|m| m.body))
                .unwrap_or("")
                .replace(['\n', '\r'], " ");
            c.fit(
                6,
                y + 34,
                &preview,
                16,
                if thread.unread > 0 {
                    c.accent(2)
                } else {
                    MUTED
                },
                228,
            );
            if last.as_ref().is_some_and(|m| m.unsent) && draft.is_none() {
                c.text(226, y + 7, "!", 18, MUTED);
            }
        }
        y += height;
    }
    c.rect(0, 282, 240, 38, BLACK);
    Softkeys::menu(if p.index == 0 { "确认" } else { "打开" }).draw(c);
    popup(c, p);
}
pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    let thread = p.mailbox.thread(&p.chat.peer);
    let items = thread.map(|t| t.items).unwrap_or_default();
    let (preview, _) = p.compose.body.preview();
    let bottom =
        crate::ui::editor::MessageLayout::measure(c, &String::from_utf16_lossy(&preview), 130).top;
    let viewport = bottom - 24;
    let mut layouts = vec![];
    let mut total = 0;
    for &item in &items {
        if let Some(m) = p.mailbox.content(item) {
            let lines = c.wrap(m.body, 18, 184);
            let height = lines.len() as i32 * 24 + 4;
            layouts.push((item, lines, total, height));
            total += height + 34;
        }
    }
    p.chat.max_scroll = (total - viewport).max(0);
    if p.chat.focus.is_none() {
        p.chat.scroll = p.chat.max_scroll;
    } else if p.chat.reveal {
        if let Some((_, _, top, height)) = p.chat.focus.and_then(|i| layouts.get(i)) {
            if *top < p.chat.scroll || *height + 34 > viewport {
                p.chat.scroll = *top;
            } else if *top + *height + 34 > p.chat.scroll + viewport {
                p.chat.scroll = *top + *height + 34 - viewport;
            }
        }
        p.chat.reveal = false;
    }
    p.chat.scroll = p.chat.scroll.clamp(0, p.chat.max_scroll);
    for (i, (item, lines, top, height)) in layouts.iter().enumerate() {
        let m = p.mailbox.content(*item).unwrap();
        let y = 24 + top - p.chat.scroll;
        if y + height + 34 < 24 || y > bottom {
            continue;
        }
        // Outgoing follows the observed left composer; direction assignment remains a local policy.
        let x = if m.incoming { 40 } else { 0 };
        let selected = p.chat.focus == Some(i);
        let fill = if selected { c.accent(3) } else { BLACK };
        c.rect(x, y, 200, *height, fill);
        for (row, line) in lines.iter().enumerate() {
            c.text(x + 6, y + row as i32 * 24, &line.text, 18, WHITE);
        }
        if let Some(n) = number(m.body) {
            // Highlight only the actual numeric span, including wrapped rows.
            if let Some(byte) = m.body.find(&n) {
                let start = m.body[..byte].encode_utf16().count();
                let end = start + n.encode_utf16().count();
                for (row, line) in lines.iter().enumerate() {
                    let a = start.max(line.start);
                    let b = end.min(line.end);
                    if a < b {
                        let units: Vec<_> = m.body.encode_utf16().collect();
                        let left = String::from_utf16_lossy(&units[line.start..a]);
                        let part = String::from_utf16_lossy(&units[a..b]);
                        let xx = x + 6 + c.width(&left, 18);
                        let yy = y + row as i32 * 24;
                        let w = c.width(&part, 18);
                        if selected && p.chat.number_focus {
                            c.rect(xx, yy, w, 21, WHITE);
                            c.text(xx, yy, &part, 18, BLACK);
                        }
                        c.rect(
                            xx,
                            yy + 21,
                            w,
                            1,
                            if selected { WHITE } else { c.accent(2) },
                        );
                    }
                }
            }
        }
        c.rect(x, y + height, 200, 2, WHITE);
        let tail = if m.incoming { 208 } else { 29 };
        for row in 0..16 {
            let xx = if m.incoming { tail - 17 + row } else { tail };
            c.rect(xx, y + height + 1 + row, 18 - row, 1, WHITE);
            if row < 12 {
                c.rect(xx + 2, y + height + 1 + row, 14 - row, 1, fill);
            }
        }
        let label = if m.unsent {
            "未发送".into()
        } else {
            stamp(m.time, false)
        };
        let xx = if m.incoming {
            x + 3
        } else {
            200 - c.width(&label, 14) - 3
        };
        c.text(
            xx,
            y + height + 7,
            &label,
            14,
            if m.unsent { MUTED } else { WHITE },
        );
    }
    compose::body(c, p, true);
    // Mask scrolling content at both edges; status and composer are fixed layers.
    c.rect(0, 0, 240, 24, BLACK);
    super::status(c);
    c.fit(6, 1, &p.chat.peer, 14, WHITE, 125);
    if p.chat.max_scroll > 0 {
        c.rect(237, 24, 2, viewport, MUTED);
        c.rect(
            237,
            24 + (viewport - 12) * p.chat.scroll / p.chat.max_scroll,
            2,
            12,
            WHITE,
        );
    }
    popup(c, p);
}
fn popup(c: &mut Canvas, p: &Phone) {
    let labels = match p.chat.popup {
        Popup::None => return,
        Popup::Root => ["草稿", "未发送"],
        Popup::Bubble => ["删除", "转发"],
        Popup::Number => ["呼叫号码", "发送信息"],
        Popup::Delete => {
            c.dim();
            c.text(44, 190, "删除此信息？", 20, WHITE);
            c.rect(0, 282, 240, 38, BLACK);
            Softkeys::back("确认").draw(c);
            return;
        }
    };
    c.dim();
    list::popup(c, labels.len(), p.chat.choice, |i| list::Row {
        label: labels[i],
        enabled: !(p.chat.popup == Popup::Number && i == 0),
        marker: list::Marker::None,
    });
    c.rect(0, 282, 240, 38, BLACK);
    Softkeys::back("选择").draw(c);
}
