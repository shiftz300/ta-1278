//! Shared listbox and popup rows. Selection and activation remain in app models.
use crate::canvas::{Canvas, MUTED, WHITE};
#[derive(Clone, Copy, Default)]
pub enum Marker {
    #[default]
    None,
    Radio(bool),
    Check(bool),
}
pub struct Row<'a> {
    pub label: &'a str,
    pub enabled: bool,
    pub marker: Marker,
}
impl<'a> Row<'a> {
    pub fn text(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
            marker: Marker::None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct ListStyle {
    pub top: i32,
    pub row_height: i32,
    pub row_width: i32,
    pub visible: usize,
    pub inset: i32,
    pub text_y: i32,
    pub font: u16,
    pub text_width: i32,
    pub marker_shades: Option<(usize, usize)>,
}
impl ListStyle {
    pub const STANDARD: Self = Self {
        top: super::TITLE_HEIGHT,
        row_height: 42,
        row_width: 240,
        visible: 6,
        inset: 6,
        text_y: 10,
        font: 20,
        text_width: 228,
        marker_shades: None,
    };
    pub fn popup(count: usize) -> Self {
        let visible = count.min(6);
        Self {
            top: 276 - visible as i32 * 38,
            row_height: 38,
            visible,
            text_y: 7,
            ..Self::STANDARD
        }
    }
}
pub fn draw<'a>(
    c: &mut Canvas,
    count: usize,
    index: usize,
    style: ListStyle,
    mut row: impl FnMut(usize) -> Row<'a>,
) {
    let start = index.saturating_sub(style.visible.saturating_sub(1));
    for i in start..count.min(start + style.visible) {
        let y = style.top + (i - start) as i32 * style.row_height;
        let item = row(i);
        if i == index {
            c.rect(0, y, style.row_width, style.row_height, c.accent(3));
        }
        let check = matches!(item.marker, Marker::Check(_));
        c.fit(
            if check {
                style.inset.max(36)
            } else {
                style.inset
            },
            y + style.text_y,
            item.label,
            style.font,
            if item.enabled { WHITE } else { MUTED },
            if matches!(item.marker, Marker::None) {
                style.text_width
            } else {
                style.text_width.min(203)
            },
        );
        let color = if let Some((focused, normal)) = style.marker_shades {
            c.accent(if i == index { focused } else { normal })
        } else if i == index {
            WHITE
        } else {
            c.accent(3)
        };
        match item.marker {
            Marker::None => {}
            Marker::Radio(selected) => {
                c.ring(225, y + style.row_height / 2, 7, color);
                if selected {
                    c.rect(222, y + style.row_height / 2 - 3, 6, 6, color);
                }
            }
            Marker::Check(selected) => {
                c.rect(
                    6,
                    y + 12,
                    18,
                    18,
                    if selected { c.accent(6) } else { WHITE },
                );
                if selected {
                    for d in 0..4 {
                        c.rect(9 + d, y + 20 + d, 2, 2, WHITE);
                    }
                    for d in 0..7 {
                        c.rect(12 + d, y + 23 - d, 2, 2, WHITE);
                    }
                } else {
                    c.rect(8, y + 14, 14, 14, [0, 0, 0, 255]);
                }
            }
        }
    }
}
pub fn plain(c: &mut Canvas, labels: &[String], index: usize) {
    if labels.is_empty() {
        c.text(8, 120, "这里没有可显示的内容。", 17, WHITE);
    }
    draw(c, labels.len(), index, ListStyle::STANDARD, |i| {
        Row::text(&labels[i])
    });
}
pub fn popup<'a>(c: &mut Canvas, count: usize, index: usize, row: impl FnMut(usize) -> Row<'a>) {
    let style = ListStyle::popup(count);
    c.rect(
        0,
        style.top,
        240,
        style.visible as i32 * style.row_height,
        [0, 0, 0, 255],
    );
    draw(c, count, index, style, row);
}

/// Native two-line rows used by storage and the Other menu.
pub fn detail_rows<'a>(
    c: &mut Canvas,
    count: usize,
    index: usize,
    mut row: impl FnMut(usize) -> (&'a str, &'a str),
) {
    let start = index.saturating_sub(3);
    for i in start..count.min(start + 4) {
        let (label, value) = row(i);
        let y = 24 + (i - start) as i32 * 64;
        if i == index {
            c.rect(0, y, 240, 64, c.accent(3));
        }
        c.text(6, y + 10, label, 20, WHITE);
        c.text(6, y + 38, value, 16, MUTED);
    }
}
