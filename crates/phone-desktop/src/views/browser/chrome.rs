//! System/browser modal geometry measured from the target handset, separate from web layout.
use crate::views::*;
pub(in crate::views) fn menu(c: &mut Canvas, p: &Phone, kind: MenuKind, index: usize) {
    let labels = p.menu_labels(kind);
    let child = matches!(
        kind,
        MenuKind::BrowserOpen | MenuKind::BrowserNavigation | MenuKind::BrowserSave
    );
    let count = labels.len().min(6);
    let start = index.saturating_sub(count - 1);
    let row = 38;
    let top = if child {
        144 - (count as i32 * row) / 2
    } else {
        276 - count as i32 * row
    };
    if child {
        c.rect(0, 48, 240, 234, [45, 50, 45, 255]);
        c.text(
            0,
            1,
            match kind {
                MenuKind::BrowserOpen => "打开",
                MenuKind::BrowserNavigation => "导航",
                _ => "保存",
            },
            14,
            WHITE,
        );
    }
    // Root popup also owns the gap below its last row. Otherwise the dimmed
    // web page shows through at y=276..281 as a false horizontal scrollbar.
    let panel_height = if child { count as i32 * row } else { 282 - top };
    c.rect(0, top, 240, panel_height, [0, 0, 0, 255]);
    list::draw(
        c,
        labels.len(),
        index,
        list::ListStyle {
            top,
            row_height: row,
            visible: count,
            text_y: 7,
            row_width: if labels.len() > 6 { 234 } else { 240 },
            text_width: 214,
            ..list::ListStyle::STANDARD
        },
        |i| list::Row {
            label: &labels[i],
            enabled: p.menu_enabled(kind, i),
            marker: list::Marker::None,
        },
    );
    if kind == MenuKind::Browser {
        for i in start..(start + count).min(3) {
            c.text(216, top + (i - start) as i32 * row + 5, "›", 26, WHITE);
        }
    }
    if labels.len() > 6 {
        c.rect(236, top, 4, count as i32 * row, [46, 51, 46, 255]);
        c.rect(
            236,
            top + (start as i32 * count as i32 * row / labels.len() as i32),
            4,
            (count * count * row as usize / labels.len()) as i32,
            MUTED,
        );
    }
    c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
    Softkeys::back("选择").draw(c);
}
pub(in crate::views) fn editor(c: &mut Canvas, p: &mut Phone, target: TextTarget, e: &Editor) {
    status(c);
    c.text(
        6,
        1,
        if target == TextTarget::Address {
            "输入URL"
        } else {
            "编辑"
        },
        14,
        WHITE,
    );
    let hidden = matches!(target,TextTarget::Field(i) if matches!(p.browser.page.elements.get(i),Some(Element::Field{password:true,..})));
    let (preview, _) = e.preview();
    let text = if hidden {
        "*".repeat(preview.len())
    } else {
        String::from_utf16_lossy(&preview)
    };
    let rows = c.wrap(&text, 16, 228).len().clamp(1, 10);
    let height = rows as i32 * 23;
    if hidden {
        let mut masked = Editor::new(&text, e.limit);
        masked.cursor = e.cursor;
        p.editor_lines = Edit::new([6, 27, 228, height])
            .sized(16, 23)
            .draw(c, &masked);
    } else {
        p.editor_lines = Edit::new([6, 27, 228, height]).sized(16, 23).draw(c, e);
    }
    c.rect(6, 27 + height, 228, 3, c.accent(3));
    crate::ui::editor::input_mode(c, e, 232, 34 + height);
    Softkeys::new(
        if target == TextTarget::Address && e.units.is_empty() {
            Button::None
        } else {
            Button::Text("确认")
        },
        Button::None,
        Button::Text(if e.units.is_empty() {
            if target == TextTarget::Address {
                "取消"
            } else {
                "返回"
            }
        } else {
            "删除"
        }),
    )
    .draw(c);
}
pub(in crate::views) fn settings(c: &mut Canvas, p: &Phone) {
    let modal = matches!(
        p.browser_view,
        BrowserView::HomeType | BrowserView::FontSize | BrowserView::Clear
    );
    let active = match p.browser_view {
        BrowserView::HomeType => 2,
        BrowserView::FontSize => 5,
        BrowserView::Clear => 6,
        _ => p.index,
    };
    let rows = [
        (42, 52),
        (94, 52),
        (188, 52),
        (240, 52),
        (334, if active == 4 { 36 } else { 26 }),
        (360, 52),
        (412, if active == 6 { 36 } else { 28 }),
        (480, 36),
    ];
    let bottom = rows[active.min(7)].0 + rows[active.min(7)].1;
    let scroll = (bottom - 276).max(0);
    for (y, label) in [
        (24, ""),
        (146, "主页设置"),
        (292, "内容设置"),
        (440, "恢复预设设置"),
    ] {
        let yy = y - scroll;
        c.text(6, yy + 9, label, 16, MUTED);
        c.rect(0, yy + 9 + 22, 240, 3, [64, 64, 64, 255]);
    }
    let labels = [
        "SIM1接入点",
        "SIM2接入点",
        "设置主页类型",
        "自定义主页",
        "下载图片",
        "字体大小",
        "清除",
        "恢复预设设置",
    ];
    for (i, (top, h)) in rows.iter().enumerate() {
        let y = top - scroll;
        if y + h < 24 || y >= 282 {
            continue;
        }
        if i == active {
            c.rect(0, y, 240, *h, c.accent(3));
        }
        let standalone = matches!(i, 4 | 6 | 7);
        let selected_standalone = standalone && i == active;
        c.text(
            6,
            y + if standalone && !selected_standalone {
                2
            } else {
                4
            },
            labels[i],
            if selected_standalone { 24 } else { 20 },
            WHITE,
        );
        let value = match i {
            0 | 1 => "未设置",
            2 => "自定义",
            3 => p.browser_data.settings.home_url.as_str(),
            5 => {
                if p.browser_data.settings.font_size == 0 {
                    "小号字体"
                } else {
                    "普通字体"
                }
            }
            _ => "",
        };
        c.fit(6, y + if i < 2 { 29 } else { 27 }, value, 12, WHITE, 220);
        if i == 4 {
            let yy = y + (h - 18) / 2;
            c.rect(212, yy, 18, 18, WHITE);
            c.rect(
                214,
                yy + 2,
                14,
                14,
                if i == active {
                    c.accent(3)
                } else {
                    [0, 0, 0, 255]
                },
            );
            if p.browser_data.settings.download_images {
                check(c, 215, yy + 5);
            }
        } else if i < 7 {
            let cy = y + h / 2;
            for dy in -8..=8 {
                for dx in -8..=8 {
                    if dx * dx + dy * dy <= 64 {
                        c.rect(221 + dx, cy + dy, 1, 1, WHITE);
                    }
                }
            }
            for row in 0..4 {
                c.rect(218 + row, cy - 1 + row, 7 - 2 * row, 1, MUTED);
            }
        }
    }
    c.rect(0, 0, 240, 24, [0, 0, 0, 255]);
    chrome::title(c, "设置");
    status(c);
    c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
    Softkeys::back("选择").draw(c);
    if modal {
        c.dim();
        // Native picker owns an opaque panel beyond its highlighted rows.
        let (panel_top, panel_height) = if p.browser_view == BrowserView::HomeType {
            (76, 170)
        } else {
            (62, 194)
        };
        c.rect(0, panel_top, 240, panel_height, [0, 0, 0, 255]);
        let labels = p.browser_items();
        let top = if p.browser_view == BrowserView::HomeType {
            123
        } else {
            110
        };
        c.rect(12, top, 216, labels.len() as i32 * 42, [0, 0, 0, 255]);
        for (i, label) in labels.iter().enumerate() {
            let y = top + i as i32 * 42;
            if i == p.index {
                c.rect(12, y, 216, 42, c.accent(3));
            }
            c.text(
                if p.browser_view == BrowserView::Clear {
                    12
                } else {
                    18
                },
                y + 10,
                label,
                18,
                WHITE,
            );
            if p.browser_view == BrowserView::Clear {
                c.rect(201, y + 12, 18, 18, WHITE);
                c.rect(
                    203,
                    y + 14,
                    14,
                    14,
                    if i == p.index {
                        c.accent(3)
                    } else {
                        [0, 0, 0, 255]
                    },
                );
                if p.browser_clear[i] {
                    c.rect(201, y + 12, 18, 18, c.accent(3));
                    check(c, 204, y + 17);
                }
                continue;
            }
            let selected = if p.browser_view == BrowserView::HomeType {
                0
            } else {
                p.browser_data.settings.font_size.min(1)
            };
            list::radio(
                c,
                i == selected,
                204,
                y + 12,
                if i == p.index {
                    c.accent(6)
                } else {
                    c.accent(3)
                },
            );
        }
    }
    if p.browser_view == BrowserView::Clear {
        c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
        Softkeys::new(Button::Text("清除"), Button::Text("选择"), Button::Back).draw(c);
    }
}

fn check(c: &mut Canvas, x: i32, y: i32) {
    for d in 0..4 {
        c.rect(x + d, y + 3 + d, 2, 2, WHITE);
    }
    for d in 0..8 {
        c.rect(x + 3 + d, y + 6 - d, 2, 2, WHITE);
    }
}
