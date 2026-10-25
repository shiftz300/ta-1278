mod browser;
mod browser_chrome;
mod browser_layout;
mod compose;
mod music;
mod organizer;
mod pickers;
mod settings;
mod snake;
mod softkeys;
mod tools;
use crate::{canvas::*, phone::*};
use phone_core::{
    browser::Element,
    editor::{Editor, VisualRow},
    sms,
};
pub fn render(c: &mut Canvas, p: &mut Phone) {
    c.accent = if p.screen == Screen::Settings
        && p.settings_page == crate::phone::settings::SettingsPage::Theme
    {
        p.theme_preview
    } else {
        p.accent
    };
    c.caret = None;
    c.background();
    status(c);
    match p.screen {
        Screen::Home => {
            let seconds = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 8 * 3600;
            let time = format!("{:02}:{:02}", seconds / 3600 % 24, seconds / 60 % 60);
            let date = phone_core::calendar::Date::default();
            let w = c.width(&time, 54);
            c.text((240 - w) / 2, 88, &time, 54, WHITE);
            let s = format!(
                "{}月{}日 星期{}",
                date.month,
                date.day,
                ["一", "二", "三", "四", "五", "六", "日"][date.weekday()]
            );
            let w = c.width(&s, 18);
            c.text((240 - w) / 2, 153, &s, 18, WHITE);
            let shortcuts = crate::phone::settings::SHORTCUTS;
            c.fit(
                6,
                296,
                shortcuts[p.preferences.left_shortcut],
                16,
                WHITE,
                67,
            );
            c.fit(
                166,
                296,
                shortcuts[p.preferences.right_shortcut],
                16,
                WHITE,
                68,
            );
            c.text(90, 296, "功能表", 20, WHITE);
        }
        Screen::Menu => {
            let entries = &crate::menu::ENTRIES;
            c.text(6, 1, entries[p.index].label, 14, WHITE);
            let first_row = (p.index / 3).saturating_sub(2);
            for (i, entry) in entries.iter().enumerate().skip(first_row * 3).take(12) {
                let x = 38 + (i % 3) as i32 * 81;
                let y = 68 + (i / 3 - first_row) as i32 * 82;
                if p.index == i {
                    c.ring(x, y, 31, c.accent(2));
                }
                c.icon(entry.icon, x - 38, y - 38, 76);
            }
            c.rect(0, 288, 240, 32, [0, 0, 0, 255]);
            footer(c, "≡", "←");
            c.text(104, 298, "选择", 14, WHITE);
        }
        Screen::Messages => {
            c.text(6, 1, "信息", 14, WHITE);
            let labels = [
                "+ 新建信息".to_owned(),
                format!("草稿 ({})", p.mailbox.drafts.len()),
                format!("发件箱 ({})", p.mailbox.outbox.len()),
            ];
            for (i, label) in labels.iter().enumerate() {
                let y = 24 + i as i32 * 46;
                if i == p.index {
                    c.rect(0, y, 240, 46, c.accent(3));
                }
                let w = c.width(label, 20);
                c.text((240 - w) / 2, y + 13, label, 20, WHITE);
            }
            footer(c, "", "←");
            c.text(98, 296, "确认", 20, WHITE);
        }
        Screen::Drafts | Screen::Outbox => {
            let drafts = p.screen == Screen::Drafts;
            title(
                c,
                if drafts {
                    "草稿"
                } else {
                    "发件箱 · 未发送"
                },
            );
            let items: Vec<_> = if drafts {
                &p.mailbox.drafts
            } else {
                &p.mailbox.outbox
            }
            .iter()
            .map(|m| {
                format!(
                    "{}  {}",
                    if m.to.is_empty() {
                        "无收件人"
                    } else {
                        &m.to
                    },
                    m.body.replace('\n', " ")
                )
            })
            .collect();
            list(c, &items, p.index, 54, 210);
            c.text(8, 273, "删除键：删除这条信息", 12, MUTED);
            footer(c, "打开", "返回");
        }
        Screen::Compose => compose::render(c, p),
        Screen::Browser => browser::render(c, p),
        Screen::Music => music::render(c, p),
        Screen::Snake => snake::render(c, p),
        Screen::Dial => {
            title(c, "Phone");
            p.editor_lines = editor(c, &p.dial, 10, 84, 220, 140);
            c.text(10, 252, "No phone service", 15, MUTED);
            footer(
                c,
                "Call",
                if p.dial.units.is_empty() {
                    "返回"
                } else {
                    "Clear"
                },
            );
        }
        Screen::Contacts | Screen::Organizer(_) => organizer::render(c, p),
        Screen::Settings => settings::render(c, p),
        Screen::Tools(_) => tools::render(c, p),
    }
    if !p.status.is_empty() {
        c.rect(0, 274, 240, 22, [83, 30, 32, 255]);
        c.fit(5, 276, &p.status, 12, WHITE, 230);
    }
    if let Some(overlay) = p.overlay.clone() {
        let browser_modal = p.screen == Screen::Browser
            && matches!(
                overlay,
                Overlay::Menu { .. } | Overlay::ClosePage | Overlay::CloseBrowser
            );
        if browser_modal {
            c.dim();
        } else {
            c.background();
        }
        match overlay {
            Overlay::Notice(s) => {
                c.text(12, 51, "提示", 18, WHITE);
                let lines = c.wrap(&s, 15, 212);
                p.notice_lines = lines.len();
                for (i, l) in lines.iter().skip(p.notice_scroll).take(11).enumerate() {
                    c.text(12, 86 + i as i32 * 18, &l.text, 15, WHITE);
                }
                footer(c, "确定", "关闭");
            }
            Overlay::Menu { kind, index } => {
                if p.screen == Screen::Browser {
                    browser_chrome::menu(c, p, kind, index);
                } else if matches!(kind, MenuKind::Symbols | MenuKind::Smileys) {
                    pickers::render(c, p, kind, index);
                } else {
                    c.text(6, 1, "选项", 14, WHITE);
                    let labels = p.menu_labels(kind);
                    let start = index.saturating_sub(5);
                    for (i, label) in labels.iter().enumerate().skip(start).take(6) {
                        let y = 24 + (i - start) as i32 * 42;
                        if i == index {
                            c.rect(0, y, 240, 42, c.accent(3));
                        }
                        c.fit(
                            6,
                            y + 10,
                            label,
                            20,
                            if p.menu_enabled(kind, i) {
                                WHITE
                            } else {
                                MUTED
                            },
                            228,
                        );
                    }
                    footer(c, "", "←");
                    c.text(98, 296, "选择", 20, WHITE);
                }
            }
            Overlay::Text { target, editor: e } => {
                if p.screen == Screen::Browser {
                    browser_chrome::editor(c, p, target, &e);
                } else {
                    c.text(
                        6,
                        1,
                        if target == TextTarget::Address {
                            "输入URL"
                        } else {
                            "编辑"
                        },
                        18,
                        WHITE,
                    );
                    let hidden = matches!(target,TextTarget::Field(i) if matches!(p.browser.page.elements.get(i),Some(Element::Field{password:true,..})));
                    if hidden {
                        let mut masked = Editor::new(&"*".repeat(e.units.len()), e.limit);
                        masked.cursor = e.cursor;
                        p.editor_lines = editor(c, &masked, 6, 28, 228, 234);
                    } else {
                        p.editor_lines = editor(c, &e, 6, 28, 228, 234);
                    }
                    c.text(12, 270, mode(&e), 12, MUTED);
                    footer(
                        c,
                        "确认",
                        if e.units.is_empty() {
                            "返回"
                        } else {
                            "删除"
                        },
                    );
                }
            }
            Overlay::Select {
                element,
                index,
                selected,
                ..
            } => {
                c.text(6, 1, "选择", 14, WHITE);
                if let Some(Element::Select { options, .. }) = p.browser.page.elements.get(element)
                {
                    let start = index.saturating_sub(5);
                    for (i, (label, _)) in options.iter().enumerate().skip(start).take(6) {
                        let y = 24 + (i - start) as i32 * 42;
                        if index == i {
                            c.rect(0, y, 240, 42, c.accent(3));
                        }
                        c.fit(6, y + 10, label, 18, WHITE, 204);
                        c.ring(225, y + 21, 7, if index == i { WHITE } else { c.accent(3) });
                        if selected.get(i) == Some(&true) {
                            c.rect(
                                222,
                                y + 18,
                                6,
                                6,
                                if index == i { WHITE } else { c.accent(3) },
                            );
                        }
                    }
                }
                softkeys::browser(c, false, "选择", false);
            }
            Overlay::ClosePage => {
                c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
                c.text(90, 247, "关闭当前页面？", 18, WHITE);
                softkeys::browser(c, false, "确认", true);
            }
            Overlay::CloseBrowser => {
                c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
                c.text(30, 239, "是否要关闭浏览器？", 20, WHITE);
                softkeys::browser(c, false, "确认", true);
            }
        }
    }
    if let Some(e) = p.editor() {
        crate::candidate_view::render(c, e);
    }
}
fn status(c: &mut Canvas) {
    c.text(184, 1, "离线", 12, WHITE);
    c.rect(218, 4, 17, 9, WHITE);
    c.rect(220, 6, 12, 5, [30, 30, 30, 255]);
    c.rect(235, 7, 2, 3, WHITE);
}
fn mode(e: &Editor) -> &'static str {
    e.mode_label()
}
fn title(c: &mut Canvas, s: &str) {
    c.fit(6, 23, s, 18, WHITE, 228);
}
fn footer(c: &mut Canvas, left: &str, right: &str) {
    c.text(6, 294, left, 20, WHITE);
    let width = c.width(right, 20);
    c.text(234 - width, 294, right, 20, WHITE);
}
fn list(c: &mut Canvas, labels: &[String], index: usize, y: i32, height: i32) {
    if labels.is_empty() {
        c.text(12, y + 20, "空", 16, MUTED);
        return;
    }
    let visible = (height / 32) as usize;
    let start = index.saturating_sub(visible.saturating_sub(1));
    for (i, s) in labels.iter().enumerate().skip(start).take(visible) {
        let yy = y + (i - start) as i32 * 32;
        if i == index {
            c.rect(7, yy, 226, 30, c.accent(3));
        }
        c.fit(13, yy + 5, s, 16, WHITE, 214);
    }
    if labels.len() > visible {
        c.rect(236, y, 2, height, [44, 60, 78, 255]);
        c.rect(
            236,
            y + (height - 10) * index as i32 / (labels.len() - 1) as i32,
            2,
            10,
            WHITE,
        );
    }
}
fn editor(c: &mut Canvas, e: &Editor, x: i32, y: i32, width: i32, height: i32) -> Vec<VisualRow> {
    editor_sized(c, e, x, y, width, height, 20, 26)
}
#[allow(clippy::too_many_arguments)]
fn editor_sized(
    c: &mut Canvas,
    e: &Editor,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    size: u16,
    line_height: i32,
) -> Vec<VisualRow> {
    let (units, composing) = e.preview();
    let text = String::from_utf16_lossy(&units);
    let lines = c.wrap(&text, size, width);
    let cursor = composing.as_ref().map_or(e.cursor, |r| r.start);
    let row = lines
        .partition_point(|l| l.start <= cursor)
        .saturating_sub(1);
    let visible = (height / line_height).max(1) as usize;
    let start = row.saturating_sub(visible.saturating_sub(1));
    for (i, line) in lines.iter().enumerate().skip(start).take(visible) {
        let yy = y + (i - start) as i32 * line_height;
        let highlight = composing.as_ref().map(|r| (r.start, r.end)).or(e.selection);
        if let Some((a, b)) = highlight {
            let (a, b) = (a.min(b).max(line.start), a.max(b).min(line.end));
            if a < b {
                let prefix = String::from_utf16_lossy(&units[line.start..a]);
                let sel = String::from_utf16_lossy(&units[a..b]);
                let xx = c.width(&prefix, size);
                let w = c.width(&sel, size);
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
        c.text(x, yy, &line.text, size, WHITE);
        if let Some(r) = &composing {
            let (a, b) = (r.start.max(line.start), r.end.min(line.end));
            if a < b {
                let prefix = String::from_utf16_lossy(&units[line.start..a]);
                let xx = c.width(&prefix, size);
                c.text(
                    x + xx,
                    yy,
                    &String::from_utf16_lossy(&units[a..b]),
                    17,
                    [0, 8, 0, 255],
                );
            }
        }
        if i == row {
            let end = cursor.clamp(line.start, line.end);
            let prefix = String::from_utf16_lossy(&units[line.start..end]);
            let xx = c.width(&prefix, size);
            if composing.is_none() {
                c.rect(x + xx, yy + 1, 1, line_height - 3, WHITE);
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
                stops.push((offset, c.width(&prefix, size)));
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn render_all_screens_and_compose_overlays() {
        let mut c = Canvas::new().unwrap();
        let mut p = Phone::new(Default::default());
        for screen in [
            Screen::Home,
            Screen::Menu,
            Screen::Messages,
            Screen::Drafts,
            Screen::Outbox,
            Screen::Compose,
            Screen::Browser,
            Screen::Music,
            Screen::Dial,
            Screen::Contacts,
            Screen::Settings,
        ] {
            p.go(screen);
            render(&mut c, &mut p);
        }
        p.go(Screen::Compose);
        p.compose.body.insert("中😀\nhello");
        for recipient in [true, false] {
            p.compose.recipient = recipient;
            for overlay in [
                Overlay::Menu {
                    kind: MenuKind::Sms,
                    index: 0,
                },
                Overlay::Menu {
                    kind: MenuKind::ExitSms,
                    index: 0,
                },
                Overlay::Notice("Saved".into()),
            ] {
                p.overlay = Some(overlay);
                render(&mut c, &mut p);
            }
        }
        p.overlay = None;
        render(&mut c, &mut p);
        assert_eq!(c.pixels.len(), 240 * 320 * 4);
    }
}
