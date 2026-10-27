//! Shared listbox and popup rows. Selection and activation remain in app models.
use crate::canvas::{Canvas, MUTED, WHITE};
#[derive(Clone, Copy, Default)]
pub enum Marker {
    #[default]
    None,
    Child,
    Radio(bool),
    Check(bool),
    Switch(bool),
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
    pub first: Option<usize>,
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
        first: None,
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
    let start = style
        .first
        .unwrap_or(0)
        .min(index)
        .max(index.saturating_sub(style.visible.saturating_sub(1)));
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
        } else {
            c.accent(if i == index { 6 } else { 3 })
        };
        match item.marker {
            Marker::None => {}
            Marker::Child => {
                let cy = y + style.row_height / 2;
                for d in -10_i32..=10 {
                    c.rect(
                        228 - d.abs(),
                        cy + d,
                        1,
                        1,
                        if item.enabled { WHITE } else { MUTED },
                    );
                }
            }
            Marker::Switch(on) => {
                let y = y + style.row_height / 2;
                let track = if item.enabled {
                    c.accent(3)
                } else {
                    c.accent(6)
                };
                for dy in -9_i32..=9 {
                    for dx in -15_i32..=15 {
                        let edge = (dx.abs() - 6).max(0);
                        if edge * edge + dy * dy <= 81 {
                            c.rect(219 + dx, y + dy, 1, 1, track);
                        }
                    }
                }
                let knob = if item.enabled { WHITE } else { MUTED };
                for dy in -6_i32..=6 {
                    for dx in -6_i32..=6 {
                        if dx * dx + dy * dy <= 36 {
                            c.rect(if on { 225 } else { 213 } + dx, y + dy, 1, 1, knob);
                        }
                    }
                }
            }
            Marker::Radio(selected) => {
                radio(c, selected, 216, y + style.row_height / 2 - 9, color);
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
                    c.rect(
                        8,
                        y + 14,
                        14,
                        14,
                        if i == index {
                            c.accent(3)
                        } else {
                            [0, 0, 0, 255]
                        },
                    );
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
    popup_at(c, count, index, None, row);
}
pub fn popup_at<'a>(
    c: &mut Canvas,
    count: usize,
    index: usize,
    top: Option<i32>,
    row: impl FnMut(usize) -> Row<'a>,
) {
    let mut style = ListStyle::popup(count);
    if let Some(top) = top {
        style.top = top;
    }
    if count > style.visible {
        style.row_width = 234;
        style.text_width = 222;
    }
    c.rect(
        0,
        style.top,
        240,
        if top.is_some() {
            count.min(style.visible) as i32 * style.row_height
        } else {
            super::SOFTKEY_TOP - style.top
        },
        [0, 0, 0, 255],
    );
    draw(c, count, index, style, row);
    if count > style.visible {
        let height = style.visible as i32 * style.row_height;
        let start = index.saturating_sub(style.visible - 1);
        c.rect(236, style.top, 4, height, c.accent(6));
        c.rect(
            236,
            style.top + start as i32 * height / count as i32,
            4,
            style.visible as i32 * height / count as i32,
            MUTED,
        );
    }
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

/// Theme-colored native 18px radio artwork, with the original round center.
pub fn radio(c: &mut Canvas, selected: bool, x: i32, y: i32, color: crate::canvas::Color) {
    c.tint_symbol(
        if selected {
            crate::assets::Symbol::Selected
        } else {
            crate::assets::Symbol::Radio
        },
        x,
        y,
        color,
    );
}
