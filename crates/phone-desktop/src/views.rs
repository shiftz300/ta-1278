use crate::{canvas::*, phone::*};
use phone_core::{browser::Element, editor::Editor, sms};
pub fn render(c: &mut Canvas, p: &mut Phone) {
    c.clear(BG);
    c.rect(0, 0, 240, 20, [20, 38, 61, 255]);
    c.text(7, 1, "OFFLINE", 12, MUTED);
    c.text(172, 1, "TA-1278", 12, MUTED);
    match p.screen {
        Screen::Home => {
            c.rect(0, 20, 240, 278, [21, 45, 77, 255]);
            c.text(24, 75, "Nokia", 34, WHITE);
            c.text(25, 120, "215 4G", 20, WHITE);
            c.text(25, 157, "Interaction prototype", 14, MUTED);
            c.text(25, 220, "No SIM / local services", 14, WHITE);
            footer(c, "Menu", "Contacts");
        }
        Screen::Menu => {
            title(c, "Menu");
            let labels = [
                "Messages", "Browser", "Music", "Phone", "Contacts", "Settings",
            ];
            for (i, label) in labels.iter().enumerate() {
                let x = 8 + (i % 2) as i32 * 116;
                let y = 53 + (i / 2) as i32 * 78;
                if p.index == i {
                    c.rect(x, y, 108, 72, BLUE);
                }
                c.icon(i, x + 36, y + 4, 36);
                c.fit(x + 6, y + 44, label, 15, WHITE, 98);
            }
            footer(c, "Select", "Back");
        }
        Screen::Messages => {
            title(c, "Messages");
            list(
                c,
                &[
                    "Write message".into(),
                    format!("Drafts ({})", p.mailbox.drafts.len()),
                    format!("Outbox ({})", p.mailbox.outbox.len()),
                ],
                p.index,
                54,
                220,
            );
            footer(c, "Select", "Back");
        }
        Screen::Drafts | Screen::Outbox => {
            let drafts = p.screen == Screen::Drafts;
            title(
                c,
                if drafts {
                    "Drafts"
                } else {
                    "Outbox · NOT SENT"
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
                        "No recipient"
                    } else {
                        &m.to
                    },
                    m.body.replace('\n', " ")
                )
            })
            .collect();
            list(c, &items, p.index, 54, 210);
            c.text(8, 273, "Backspace: delete entry", 12, MUTED);
            footer(c, "Open", "Back");
        }
        Screen::Compose => {
            title(c, "Write message");
            let count = sms::count(&p.compose.body.text());
            c.fit(
                10,
                51,
                &format!("To: {}", p.compose.to.text()),
                15,
                if p.compose.recipient { WHITE } else { MUTED },
                220,
            );
            c.rect(8, 72, 224, 1, BLUE);
            c.text(
                10,
                78,
                &format!(
                    "{} / {}    {}",
                    count.remaining,
                    count.segments,
                    if count.ucs2 { "Unicode" } else { "Latin" }
                ),
                12,
                MUTED,
            );
            if p.compose.recipient {
                p.editor_lines = editor(c, &p.compose.to, 10, 102, 220, 152);
            } else {
                p.editor_lines = editor(c, &p.compose.body, 10, 102, 220, 152);
            }
            let active_editor = if p.compose.recipient {
                &p.compose.to
            } else {
                &p.compose.body
            };
            c.text(
                10,
                270,
                &format!("{} · Tab: To / Text", mode(active_editor)),
                12,
                MUTED,
            );
            footer(
                c,
                "Options",
                if active_editor.units.is_empty() {
                    "Back"
                } else {
                    "Clear"
                },
            );
        }
        Screen::Browser => {
            title(
                c,
                if p.browser.loading {
                    "Loading…"
                } else {
                    &p.browser.page.title
                },
            );
            let labels: Vec<_> = p.browser.page.elements.iter().map(Element::label).collect();
            browser_list(c, &labels, p.browser.focus);
            if let Some(err) = &p.browser.error {
                c.rect(0, 252, 240, 44, [83, 30, 32, 255]);
                c.fit(6, 256, err, 13, WHITE, 228);
                c.text(6, 275, "Previous page kept", 12, MUTED);
            }
            footer(
                c,
                "Options",
                if p.browser.loading { "Stop" } else { "Back" },
            );
        }
        Screen::Music => {
            title(c, "Music · logic test");
            c.icon(2, 88, 65, 64);
            c.fit(
                15,
                149,
                p.music
                    .tracks
                    .get(p.music.index)
                    .map_or("Empty library", |t| &t.0),
                18,
                WHITE,
                210,
            );
            c.text(
                17,
                180,
                if p.music.playing { "Playing" } else { "Paused" },
                15,
                WHITE,
            );
            let duration = p.music.duration();
            c.rect(16, 209, 208, 4, [50, 65, 85, 255]);
            c.rect(
                16,
                209,
                (208. * p.music.position / duration.max(1.)).clamp(0., 208.) as i32,
                4,
                [96, 177, 246, 255],
            );
            c.text(
                16,
                222,
                &format!(
                    "{:02}:{:02} / {:02}:{:02}",
                    p.music.position as u32 / 60,
                    p.music.position as u32 % 60,
                    duration as u32 / 60,
                    duration as u32 % 60
                ),
                14,
                WHITE,
            );
            c.text(
                16,
                246,
                &format!(
                    "Vol {} · {:?} · shuffle {}",
                    p.music.volume, p.music.repeat, p.music.shuffle
                ),
                12,
                MUTED,
            );
            c.text(16, 273, "Simulated progress · no audio", 12, MUTED);
            footer(c, "Options", "Back");
        }
        Screen::Dial => {
            title(c, "Phone");
            p.editor_lines = editor(c, &p.dial, 10, 84, 220, 140);
            c.text(10, 252, "No phone service", 15, MUTED);
            footer(
                c,
                "Call",
                if p.dial.units.is_empty() {
                    "Back"
                } else {
                    "Clear"
                },
            );
        }
        Screen::Contacts => {
            title(c, "Contacts");
            c.text(16, 90, "No local contacts", 18, WHITE);
            c.text(16, 126, "Backend not connected", 14, MUTED);
            footer(c, "Info", "Back");
        }
        Screen::Settings => {
            title(c, "Settings");
            list(
                c,
                &[
                    "240 x 320 display".into(),
                    "Multi-tap / direct typing".into(),
                    "F4: abc / ABC / 123".into(),
                    "T9: not implemented".into(),
                    "Local test services".into(),
                ],
                0,
                60,
                210,
            );
            footer(c, "Info", "Back");
        }
    }
    if !p.status.is_empty() {
        c.rect(0, 274, 240, 22, [83, 30, 32, 255]);
        c.fit(5, 276, &p.status, 12, WHITE, 230);
    }
    if let Some(overlay) = p.overlay.clone() {
        c.rect(3, 49, 234, 247, [24, 39, 58, 255]);
        match overlay {
            Overlay::Notice(s) => {
                c.text(12, 51, "Notice", 18, WHITE);
                let lines = c.wrap(&s, 15, 212);
                p.notice_lines = lines.len();
                for (i, l) in lines.iter().skip(p.notice_scroll).take(11).enumerate() {
                    c.text(12, 86 + i as i32 * 18, &l.text, 15, WHITE);
                }
                footer(c, "OK", "Close");
            }
            Overlay::Menu { kind, index } => {
                c.text(12, 51, "Options", 18, WHITE);
                list(c, &p.menu_labels(kind), index, 82, 204);
                footer(c, "Select", "Back");
            }
            Overlay::Text { target, editor: e } => {
                c.text(
                    12,
                    51,
                    if target == TextTarget::Address {
                        "Address"
                    } else {
                        "Edit field"
                    },
                    18,
                    WHITE,
                );
                let hidden = matches!(target,TextTarget::Field(i) if matches!(p.browser.page.elements.get(i),Some(Element::Field{password:true,..})));
                if hidden {
                    let mut masked = Editor::new(&"*".repeat(e.units.len()), e.limit);
                    masked.cursor = e.cursor;
                    p.editor_lines = editor(c, &masked, 12, 90, 212, 160);
                } else {
                    p.editor_lines = editor(c, &e, 12, 90, 212, 160);
                }
                c.text(12, 270, mode(&e), 12, MUTED);
                footer(c, "OK", "Cancel");
            }
            Overlay::Select {
                element,
                index,
                selected,
                multiple,
            } => {
                c.text(12, 51, "Select", 18, WHITE);
                if let Element::Select { options, .. } = &p.browser.page.elements[element] {
                    let labels = options
                        .iter()
                        .zip(&selected)
                        .map(|(o, s)| format!("[{}] {}", if *s { "x" } else { " " }, o.0))
                        .collect::<Vec<_>>();
                    list(c, &labels, index, 82, 204);
                }
                footer(
                    c,
                    if multiple { "Toggle" } else { "Choose" },
                    if multiple { "Done" } else { "Cancel" },
                );
            }
            Overlay::CloseBrowser => {
                c.text(12, 80, "Close browser?", 18, WHITE);
                footer(c, "Close", "Cancel");
            }
        }
    }
}
fn mode(e: &Editor) -> &'static str {
    if e.numeric {
        "123"
    } else if e.uppercase {
        "ABC"
    } else {
        "abc"
    }
}
fn title(c: &mut Canvas, s: &str) {
    c.rect(0, 20, 240, 28, [24, 49, 79, 255]);
    c.fit(8, 24, s, 18, WHITE, 224);
}
fn footer(c: &mut Canvas, left: &str, right: &str) {
    c.rect(0, 297, 240, 23, [25, 49, 78, 255]);
    c.text(8, 299, left, 14, WHITE);
    let width = c.width(right, 14);
    c.text(232 - width, 299, right, 14, WHITE);
}
fn list(c: &mut Canvas, labels: &[String], index: usize, y: i32, height: i32) {
    if labels.is_empty() {
        c.text(12, y + 20, "Empty", 16, MUTED);
        return;
    }
    let visible = (height / 32) as usize;
    let start = index.saturating_sub(visible.saturating_sub(1));
    for (i, s) in labels.iter().enumerate().skip(start).take(visible) {
        let yy = y + (i - start) as i32 * 32;
        if i == index {
            c.rect(7, yy, 226, 30, BLUE);
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
fn browser_list(c: &mut Canvas, labels: &[String], focus: usize) {
    let start = focus.saturating_sub(1);
    let mut y = 52;
    for (i, label) in labels.iter().enumerate().skip(start) {
        let lines = c.wrap(label, 14, 216);
        let count = lines.len().min(7);
        let height = count as i32 * 18 + 8;
        if y + height > 294 && i > focus {
            break;
        }
        if i == focus {
            c.rect(5, y, 230, height.min(294 - y), BLUE);
        }
        for (j, line) in lines.iter().take(count).enumerate() {
            if y + 4 + j as i32 * 18 + 18 > 294 {
                break;
            }
            c.text(10, y + 4 + j as i32 * 18, &line.text, 14, WHITE);
        }
        y += height;
        if y >= 294 {
            break;
        }
    }
}
fn editor(c: &mut Canvas, e: &Editor, x: i32, y: i32, width: i32, height: i32) -> Vec<usize> {
    let lines = c.wrap(&e.text(), 17, width);
    let row = lines
        .partition_point(|l| l.start <= e.cursor)
        .saturating_sub(1);
    let visible = (height / 22) as usize;
    let start = row.saturating_sub(visible.saturating_sub(1));
    for (i, line) in lines.iter().enumerate().skip(start).take(visible) {
        let yy = y + (i - start) as i32 * 22;
        if let Some((a, b)) = e.selection {
            let (a, b) = (a.min(b).max(line.start), a.max(b).min(line.end));
            if a < b {
                let prefix = String::from_utf16_lossy(&e.units[line.start..a]);
                let sel = String::from_utf16_lossy(&e.units[a..b]);
                let xx = c.width(&prefix, 17);
                let w = c.width(&sel, 17);
                c.rect(x + xx, yy, w, 22, BLUE);
            }
        }
        c.text(x, yy, &line.text, 17, WHITE);
        if i == row {
            let end = e.cursor.clamp(line.start, line.end);
            let prefix = String::from_utf16_lossy(&e.units[line.start..end]);
            let xx = c.width(&prefix, 17);
            c.rect(x + xx, yy + 1, 1, 19, [105, 195, 255, 255]);
        }
    }
    lines.iter().map(|l| l.start).collect()
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
