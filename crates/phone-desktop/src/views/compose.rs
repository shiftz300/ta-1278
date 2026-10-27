//! SMS composer: bottom-growing bubble and caret-anchored prediction.
use super::*;
pub(super) fn render(c: &mut Canvas, p: &Shell) {
    chrome::title(c, "新信息");
    c.text(6, 27, "收件人", 20, MUTED);
    if !p.messaging.compose.recipient {
        c.fit(72, 27, &p.messaging.compose.to.text(), 20, MUTED, 162);
    }
    c.rect(
        6,
        61,
        228,
        3,
        if p.messaging.compose.recipient {
            c.accent(3)
        } else {
            MUTED
        },
    );
    if p.messaging.compose.recipient {
        // Empty native recipient field uses its placeholder at x=6, with caret before it.
        let x = if p.messaging.compose.to.units.is_empty() {
            6
        } else {
            72
        };
        c.layout.editor = Some(Edit::new([x, 27, 234 - x, 26]).draw(c, &p.messaging.compose.to));
    }
    body(c, p, false);
    // A partial first text row is clipped by the fixed title/status strip.
    c.rect(0, 0, 240, 24, [0, 0, 0, 255]);
    super::status(c);
    chrome::title(c, "新信息");
}

pub(super) fn body(c: &mut Canvas, p: &Shell, chat: bool) -> i32 {
    let active = !p.messaging.compose.recipient && (!chat || p.messaging.chat.focus.is_none());
    let (preview, _) = p.messaging.compose.body.preview();
    // Native recipient-focus capture: body begins no higher than y=91.
    let max_height = if chat {
        130
    } else if p.messaging.compose.recipient {
        163
    } else {
        230
    };
    let layout = crate::ui::editor::MessageLayout::measure(
        c,
        &String::from_utf16_lossy(&preview),
        max_height,
    );
    let (body_rows, height, y) = (layout.rows, layout.height, layout.top);
    let lines = body_rows.len();
    if chat {
        c.rect(0, y, 240, 320 - y, [0, 0, 0, 255]);
    }
    let bubble = if !active {
        [48, 48, 48, 255]
    } else {
        c.accent(3)
    };
    c.rect(0, y, 200, height, bubble);
    crate::ui::editor::message_tail(c, [0, 254, 200], false, bubble);
    if p.messaging.compose.body.units.is_empty() && !p.messaging.compose.body.prediction.active() {
        text::MESSAGE.draw(c, 0, y + 4, "键入信息", 20, MUTED);
        if active {
            c.rect(0, y + 5, 1, 27, WHITE);
            c.caret = Some((0, y + 4, 26));
        }
        if active {
            c.layout.editor = Some(vec![phone_core::editor::VisualRow {
                stops: vec![(0, 0)],
            }]);
        }
    } else if active {
        c.layout.editor =
            Some(Edit::message([0, y + 4, 200, height - 4]).draw(c, &p.messaging.compose.body));
    } else {
        // The inactive body has no caret/selection and never changes recipient metrics.
        let row = body_rows
            .partition_point(|r| r.start <= p.messaging.compose.body.cursor)
            .saturating_sub(1);
        let visible = ((height - 4) / 26).max(1) as usize;
        let start = row.saturating_sub(visible.saturating_sub(1));
        for (i, line) in body_rows.iter().skip(start).take(visible).enumerate() {
            text::MESSAGE.draw(c, 0, y + 4 + i as i32 * 26, &line.text, 20, WHITE);
        }
    }
    if !chat && active && height == 230 && lines > 9 {
        // Native 161-character capture: fixed track y=24..281 at the right edge.
        let visible = 9usize;
        let (_, composing) = p.messaging.compose.body.preview();
        let cursor = composing
            .as_ref()
            .map_or(p.messaging.compose.body.cursor, |r| r.start);
        let row = body_rows
            .partition_point(|r| r.start <= cursor)
            .saturating_sub(1);
        let first = row.saturating_sub(visible - 1);
        c.rect(236, 24, 4, 258, [38, 43, 37, 255]);
        c.rect(
            236,
            24 + (first * 258 / lines) as i32,
            4,
            (visible * 258 / lines) as i32,
            [112, 117, 111, 255],
        );
    }
    let count = sms::count(&String::from_utf16_lossy(&preview));
    text::MESSAGE.draw(
        c,
        49,
        253,
        &format!("{}/{}", count.remaining, count.segments),
        20,
        WHITE,
    );
    let e = if p.messaging.compose.recipient {
        &p.messaging.compose.to
    } else {
        &p.messaging.compose.body
    };
    if p.messaging.compose.recipient {
        crate::ui::editor::input_mode(c, e, 228, 70);
    } else {
        crate::ui::editor::input_mode(c, e, 193, 258);
    }
    let center = if chat && !active {
        if p.messaging.chat.number_focus {
            "保存"
        } else {
            ""
        }
    } else if e.prediction.active() {
        "选择"
    } else {
        "发送"
    };
    let enabled = (chat && p.messaging.chat.number_focus)
        || (!p.messaging.compose.to.units.is_empty() && !p.messaging.compose.body.units.is_empty());
    Softkeys::new(
        if p.messaging.compose.recipient {
            Button::Contacts
        } else {
            Button::Menu
        },
        if enabled {
            Button::Text(center)
        } else {
            Button::Disabled(center)
        },
        if active && !e.units.is_empty() {
            Button::Delete
        } else {
            Button::Back
        },
    )
    .draw(c);
    y
}
