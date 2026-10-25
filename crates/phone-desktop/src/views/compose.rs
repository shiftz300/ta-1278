//! SMS composer: bottom-growing bubble and caret-anchored prediction.
use super::*;
pub(super) fn render(c: &mut Canvas, p: &mut Phone) {
    c.text(6, 1, "新信息", 14, WHITE);
    c.text(6, 27, "收件人", 20, MUTED);
    if !p.compose.recipient {
        c.fit(72, 27, &p.compose.to.text(), 20, MUTED, 162);
    }
    c.rect(
        6,
        61,
        228,
        3,
        if p.compose.recipient {
            c.accent(3)
        } else {
            MUTED
        },
    );
    if p.compose.recipient {
        // Empty native recipient field uses its placeholder at x=6, with caret before it.
        let x = if p.compose.to.units.is_empty() { 6 } else { 72 };
        p.editor_lines = editor(c, &p.compose.to, x, 27, 234 - x, 26);
    }
    let (preview, _) = p.compose.body.preview();
    let body_rows = c.wrap(&String::from_utf16_lossy(&preview), 20, 184);
    let lines = body_rows.len();
    // Native recipient-focus capture: body begins no higher than y=91.
    let max_height = if p.compose.recipient { 163 } else { 230 };
    let height = (lines as i32 * 26 + 8).clamp(34, max_height);
    let y = 254 - height;
    let bubble = if p.compose.recipient {
        [48, 48, 48, 255]
    } else {
        c.accent(3)
    };
    c.rect(0, y, 200, height, bubble);
    c.rect(0, 254, 200, 2, WHITE);
    for row in 0..16 {
        c.rect(29, 255 + row, 18 - row, 1, WHITE);
        if row < 12 {
            c.rect(31, 255 + row, 14 - row, 1, bubble);
        }
    }
    if p.compose.body.units.is_empty() && !p.compose.body.prediction.active() {
        c.text(6, y + 4, "键入信息", 20, MUTED);
        if !p.compose.recipient {
            c.caret = Some((6, y + 4, 26));
        }
        if !p.compose.recipient {
            p.editor_lines = vec![phone_core::editor::VisualRow {
                stops: vec![(0, 0)],
            }];
        }
    } else if !p.compose.recipient {
        p.editor_lines = editor_sized(c, &p.compose.body, 6, y + 4, 184, height - 8, 20, 26);
    } else {
        // The inactive body has no caret/selection and never changes recipient metrics.
        let row = body_rows
            .partition_point(|r| r.start <= p.compose.body.cursor)
            .saturating_sub(1);
        let visible = ((height - 4) / 26).max(1) as usize;
        let start = row.saturating_sub(visible.saturating_sub(1));
        for (i, line) in body_rows.iter().skip(start).take(visible).enumerate() {
            c.text(6, y + 4 + i as i32 * 26, &line.text, 20, WHITE);
        }
    }
    let count = sms::count(&String::from_utf16_lossy(&preview));
    c.text(
        48,
        258,
        &format!("{}/{}", count.remaining, count.segments),
        20,
        WHITE,
    );
    let e = if p.compose.recipient {
        &p.compose.to
    } else {
        &p.compose.body
    };
    if p.compose.recipient {
        c.text(187, 70, &format!("✎{}", mode(e)), 16, c.accent(3));
    } else {
        c.fit(164, 259, mode(e), 16, MUTED, 70);
    }
    softkeys::icons(c, !p.compose.recipient, false);
    if p.compose.recipient {
        softkeys::contacts(c);
    }
    if !e.units.is_empty() || e.prediction.active() {
        c.rect(209, 288, 30, 26, [0, 0, 0, 255]);
        c.text(209, 290, "⌫", 24, WHITE);
    }
    c.text(
        98,
        290,
        if e.prediction.active() {
            "选择"
        } else {
            "发送"
        },
        20,
        if p.compose.to.units.is_empty() || p.compose.body.units.is_empty() {
            MUTED
        } else {
            WHITE
        },
    );
}
