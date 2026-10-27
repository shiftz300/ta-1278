//! One edit control owns measurement, selection, cursor stops and painting.
use super::text::TextMetrics;
use crate::canvas::{Canvas, Color, Line, WHITE};
use phone_core::editor::{Editor, VisualRow};
pub struct Edit {
    rect: [i32; 4],
    size: u16,
    line_height: i32,
    metrics: TextMetrics,
}
impl Edit {
    pub const fn new(rect: [i32; 4]) -> Self {
        Self {
            rect,
            size: 20,
            line_height: 26,
            metrics: TextMetrics::Standard,
        }
    }
    pub const fn sized(mut self, size: u16, line_height: i32) -> Self {
        self.size = size;
        self.line_height = line_height;
        self
    }
    pub const fn message(rect: [i32; 4]) -> Self {
        Self {
            rect,
            size: 20,
            line_height: 26,
            metrics: TextMetrics::Message,
        }
    }
    pub fn draw(&self, c: &mut Canvas, e: &Editor) -> Vec<VisualRow> {
        let Self {
            rect,
            size,
            line_height,
            metrics,
        } = *self;
        let pixels = metrics == TextMetrics::Message;
        let [x, y, width, height] = rect;
        let (units, composing) = e.preview();
        let text = String::from_utf16_lossy(&units);
        let lines = metrics.wrap(c, &text, size, width);
        let cursor = composing.as_ref().map_or(e.cursor, |r| r.start);
        let row = lines
            .partition_point(|l| l.start <= cursor)
            .saturating_sub(1);
        let visible = if pixels {
            (height - 4 + line_height - 1) / line_height
        } else {
            height / line_height
        }
        .max(1) as usize;
        let start = row.saturating_sub(visible.saturating_sub(1));
        // SMS keeps the last visible baseline fixed as it expands or scrolls.
        let y = if pixels {
            y + height - visible as i32 * line_height - 4
        } else {
            y
        };
        for (i, line) in lines.iter().enumerate().skip(start).take(visible) {
            let yy = y + (i - start) as i32 * line_height;
            let highlight = composing.as_ref().map(|r| (r.start, r.end)).or(e.selection);
            if let Some((a, b)) = highlight {
                let (a, b) = (a.min(b).max(line.start), a.max(b).min(line.end));
                if a < b {
                    let prefix = String::from_utf16_lossy(&units[line.start..a]);
                    let sel = String::from_utf16_lossy(&units[a..b]);
                    let xx = metrics.width(c, &prefix, size);
                    let w = metrics.width(c, &sel, size);
                    c.rect(
                        x + xx,
                        yy,
                        w,
                        line_height,
                        if composing.is_some() {
                            WHITE
                        } else {
                            c.accent(3)
                        },
                    );
                }
            }
            metrics.draw(c, x, yy, &line.text, size, WHITE);
            if let Some(r) = &composing {
                let (a, b) = (r.start.max(line.start), r.end.min(line.end));
                if a < b {
                    let prefix = String::from_utf16_lossy(&units[line.start..a]);
                    let xx = metrics.width(c, &prefix, size);
                    let text = String::from_utf16_lossy(&units[a..b]);
                    if e.prediction.active() {
                        c.text(x + xx, yy, &text, 17, [0, 8, 0, 255]);
                    } else {
                        metrics.draw(c, x + xx, yy, &text, size, [0, 8, 0, 255]);
                    }
                }
            }
            if i == row {
                let end = cursor.clamp(line.start, line.end);
                let prefix = String::from_utf16_lossy(&units[line.start..end]);
                let xx = metrics.width(c, &prefix, size);
                if composing.is_none() {
                    c.rect(
                        x + xx,
                        yy + 1,
                        1,
                        if pixels {
                            line_height + 1
                        } else {
                            line_height - 3
                        },
                        WHITE,
                    );
                }
                c.caret = Some((x + xx, yy, line_height));
            }
        }
        lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let mut stops = vec![(line.start, 0)];
                let mut offset = line.start;
                let mut prefix = String::new();
                for ch in line.text.chars() {
                    offset += ch.len_utf16();
                    prefix.push(ch);
                    stops.push((offset, metrics.width(c, &prefix, size)));
                }
                // One UTF-16 offset belongs to the next row at a soft wrap. Do not
                // offer it as an upward target, which would appear not to move.
                if stops.len() > 1 && lines.get(i + 1).is_some_and(|next| next.start == line.end) {
                    stops.pop();
                }
                VisualRow { stops }
            })
            .collect()
    }
}

/// Both the standalone editor and conversation viewport use this measurement.
pub struct MessageLayout {
    pub rows: Vec<Line>,
    pub height: i32,
    pub top: i32,
}
impl MessageLayout {
    pub fn measure(c: &mut Canvas, value: &str, max_height: i32) -> Self {
        let rows = super::text::MESSAGE.wrap(c, value, 20, 200);
        let height = (rows.len() as i32 * 26 + 8).clamp(34, max_height);
        Self {
            rows,
            height,
            top: 254 - height,
        }
    }
}

/// Shared mode marker, fitted to the sampled edit fields.
pub fn input_mode(c: &mut Canvas, editor: &Editor, right: i32, y: i32) {
    let label = editor.mode_label();
    let x = right - super::text::MESSAGE.width(c, label, 16);
    if label != "拼" && label != "笔" {
        c.tint_symbol(crate::assets::Symbol::Pen, x - 20, y + 3, c.accent(3));
    }
    super::text::MESSAGE.draw(c, x, y, label, 16, c.accent(3));
}

/// One continuous message contour, shared by the composer and both thread directions.
pub fn message_tail(c: &mut Canvas, [x, bottom, width]: [i32; 3], incoming: bool, fill: Color) {
    c.rect(x, bottom, width, 2, WHITE);
    let tip = if incoming { x + width - 32 } else { x + 29 };
    c.rect(
        if incoming { tip - 16 } else { tip + 2 },
        bottom,
        15,
        1,
        fill,
    );
    for row in 0..16 {
        let xx = if incoming { tip - 17 + row } else { tip };
        c.rect(xx, bottom + 1 + row, 18 - row, 1, WHITE);
        if row < 12 {
            c.rect(xx + 2, bottom + 1 + row, 14 - row, 1, fill);
        }
    }
}
