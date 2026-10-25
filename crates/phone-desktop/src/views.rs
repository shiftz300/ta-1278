mod alarm;
mod browser;
mod browser_chrome;
mod browser_layout;
mod compose;
mod contacts;
mod conversation;
mod files;
mod mail;
mod music;
mod organizer;
mod pickers;
mod settings;
mod snake;
mod tools;
use crate::ui::chrome::status;
use crate::ui::softkeys::{Button, Softkeys};
use crate::ui::{chrome, editor::Edit, list, softkeys, text};
use crate::{canvas::*, phone::*};
use phone_core::{browser::Element, editor::Editor, sms};
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
            Softkeys::new(
                Button::Shortcut(p.preferences.left_shortcut),
                Button::Text("功能表"),
                Button::Shortcut(p.preferences.right_shortcut),
            )
            .draw(c);
        }
        Screen::Menu => {
            let entries = &crate::menu::ENTRIES;
            c.text(
                6,
                1,
                entries[p.index].label,
                14,
                if entries[p.index]
                    .screen
                    .is_some_and(crate::menu::phone_disabled)
                {
                    MUTED
                } else {
                    WHITE
                },
            );
            let first_row = (p.index / 3).saturating_sub(2);
            for (i, entry) in entries.iter().enumerate().skip(first_row * 3).take(12) {
                let x = 38 + (i % 3) as i32 * 81;
                let y = 68 + (i / 3 - first_row) as i32 * 82;
                if p.index == i {
                    c.ring(x, y, 31, c.accent(2));
                }
                c.icon(entry.icon, x - 38, y - 38, 76);
                if entry.screen.is_some_and(crate::menu::phone_disabled) {
                    c.grey(x - 38, y - 38, 76, 76);
                }
            }
            c.rect(0, 288, 240, 32, [0, 0, 0, 255]);
            Softkeys::menu("").draw(c);
        }
        Screen::Messages => conversation::root(c, p),
        Screen::Conversation => conversation::render(c, p),
        Screen::Drafts | Screen::Outbox | Screen::Inbox => mail::folder(c, p),
        Screen::Mail(page) => mail::render(c, p, page),
        Screen::Compose => compose::render(c, p),
        Screen::Browser => browser::render(c, p),
        Screen::Music => music::render(c, p),
        Screen::Snake => snake::render(c, p),
        Screen::Contacts | Screen::Organizer(_) => organizer::render(c, p),
        Screen::Settings => settings::render(c, p),
        Screen::Tools(_) => tools::render(c, p),
    }
    if p.mailbox.unread() > 0
        && !(p.screen == Screen::Snake
            && matches!(
                p.snake_view,
                crate::phone::snake::SnakeView::Game
                    | crate::phone::snake::SnakeView::GameOver
                    | crate::phone::snake::SnakeView::Saved
            ))
    {
        c.rect(136, 5, 11, 8, WHITE);
        c.rect(138, 7, 7, 4, [0, 0, 0, 255]);
        c.rect(137, 12, 3, 3, WHITE);
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
                Softkeys::new(Button::Text("确定"), Button::None, Button::Text("关闭")).draw(c);
            }
            Overlay::Menu { kind, index } => {
                if p.screen == Screen::Browser {
                    browser_chrome::menu(c, p, kind, index);
                } else if matches!(kind, MenuKind::Symbols | MenuKind::Smileys) {
                    pickers::render(c, p, kind, index);
                } else {
                    chrome::title(c, "选项");
                    let labels = p.menu_labels(kind);
                    list::draw(c, labels.len(), index, list::ListStyle::STANDARD, |i| {
                        list::Row {
                            label: &labels[i],
                            enabled: p.menu_enabled(kind, i),
                            marker: list::Marker::None,
                        }
                    });
                    Softkeys::back("选择").draw(c);
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
                        p.editor_lines = Edit::new([6, 28, 228, 234]).draw(c, &masked);
                    } else {
                        p.editor_lines = Edit::new([6, 28, 228, 234]).draw(c, &e);
                    }
                    c.text(12, 270, mode(&e), 12, MUTED);
                    Softkeys::new(
                        Button::Text("确认"),
                        Button::None,
                        if e.units.is_empty() {
                            Button::Back
                        } else {
                            Button::Delete
                        },
                    )
                    .draw(c);
                }
            }
            Overlay::Select {
                element,
                index,
                selected,
                ..
            } => {
                chrome::title(c, "选择");
                if let Some(Element::Select { options, .. }) = p.browser.page.elements.get(element)
                {
                    list::draw(
                        c,
                        options.len(),
                        index,
                        list::ListStyle {
                            font: 18,
                            text_width: 204,
                            ..list::ListStyle::STANDARD
                        },
                        |i| list::Row {
                            label: &options[i].0,
                            enabled: true,
                            marker: list::Marker::Radio(selected.get(i) == Some(&true)),
                        },
                    );
                }
                Softkeys::back("选择").draw(c);
            }
            Overlay::ClosePage => {
                c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
                c.text(90, 247, "关闭当前页面？", 18, WHITE);
                Softkeys::new(Button::None, Button::Text("确认"), Button::Close).draw(c);
            }
            Overlay::CloseBrowser => {
                c.rect(0, 282, 240, 38, [0, 0, 0, 255]);
                c.text(30, 239, "是否要关闭浏览器？", 20, WHITE);
                Softkeys::new(Button::None, Button::Text("确认"), Button::Close).draw(c);
            }
        }
    }
    if let Some(e) = p.editor() {
        crate::candidate_view::render(c, e);
    }
    alarm::alert(c, p);
}
fn mode(e: &Editor) -> &'static str {
    e.mode_label()
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
            Screen::Inbox,
            Screen::Compose,
            Screen::Browser,
            Screen::Music,
            Screen::Conversation,
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

#[cfg(test)]
mod sms_geometry_tests {
    use super::*;
    #[test]
    fn native_sms_rows_share_baseline_and_keep_counter_and_footer_clear() {
        let mut c = Canvas::new().unwrap();
        let mut p = Phone::new(Default::default());
        p.go(Screen::Compose);
        p.compose.recipient = false;
        p.compose.body.prediction.set_language(None);
        p.compose.body.insert("你");
        render(&mut c, &mut p);
        assert_eq!(c.caret.unwrap().1, 224);
        let fill = c.accent(3);
        let pixel = |c: &Canvas, x: usize, y: usize| {
            c.pixels[(y * 240 + x) * 4..(y * 240 + x) * 4 + 4].to_vec()
        };
        assert_eq!(pixel(&c, 199, 220), fill);
        assert_eq!(pixel(&c, 199, 219), [0, 0, 0, 255]);
        let chrome = c.pixels[254 * 240 * 4..].to_vec();
        p.compose.body.insert("\n");
        render(&mut c, &mut p);
        assert_eq!(c.caret.unwrap().1, 224);
        assert_eq!(pixel(&c, 199, 194), fill);
        assert_eq!(pixel(&c, 199, 193), [0, 0, 0, 255]);
        // Count changes, but the softkey row does not move.
        assert_eq!(&c.pixels[286 * 240 * 4..], &chrome[(286 - 254) * 240 * 4..]);
        p.compose.body.clear();
        p.compose
            .body
            .insert(&format!("{}1", "1234567890".repeat(16)));
        render(&mut c, &mut p);
        assert_eq!(p.editor_lines.len(), 11);
        assert_eq!(c.caret.unwrap().1, 224);
        assert_eq!(pixel(&c, 199, 24), fill);
        assert_eq!(pixel(&c, 199, 23), [0, 0, 0, 255]);
        assert_eq!(pixel(&c, 238, 30), [38, 43, 37, 255]);
        assert_eq!(pixel(&c, 238, 100), [112, 117, 111, 255]);
        let (_, top, height) = c.caret.unwrap();
        assert_eq!(
            pixel(&c, c.caret.unwrap().0 as usize, (top + height + 1) as usize),
            WHITE
        );
    }
    #[test]
    fn sms_integer_advances_word_wrap_and_utf16_stops_agree() {
        let mut c = Canvas::new().unwrap();
        let rows = text::MESSAGE.wrap(
            &mut c,
            &format!("呢{}123456789", "1234567890".repeat(6)),
            20,
            200,
        );
        assert_eq!(rows[0].text, "呢");
        assert_eq!(rows[1].text, "1234567890123456");
        assert_eq!(rows.len(), 6);
        assert_eq!(text::MESSAGE.width(&mut c, &rows[1].text, 20), 192);
        let mut p = Phone::new(Default::default());
        p.go(Screen::Compose);
        p.compose.recipient = false;
        p.compose.body.insert("中😀\n12");
        render(&mut c, &mut p);
        assert_eq!(p.editor_lines[1].stops, [(4, 0), (5, 12), (6, 24)]);
    }
}

#[cfg(test)]
mod shared_controls_tests {
    use super::*;
    fn region(c: &Canvas, x: usize, y: usize, width: usize, height: usize) -> Vec<u8> {
        (y..y + height)
            .flat_map(|row| {
                c.pixels[(row * 240 + x) * 4..(row * 240 + x + width) * 4]
                    .iter()
                    .copied()
            })
            .collect()
    }
    #[test]
    fn select_softkey_is_identical_across_apps() {
        let mut c = Canvas::new().unwrap();
        let mut p = Phone::new(Default::default());
        let mut expected = None;
        for screen in [
            Screen::Settings,
            Screen::Snake,
            Screen::Tools(crate::phone::tools::ToolPage::Other),
            Screen::Browser,
        ] {
            p.launch(screen);
            render(&mut c, &mut p);
            let actual = region(&c, 82, 284, 76, 36);
            if let Some(expected) = &expected {
                assert_eq!(&actual, expected, "{screen:?}");
            } else {
                expected = Some(actual);
            }
        }
    }
    #[test]
    fn message_and_browser_editors_share_delete_control() {
        let mut c = Canvas::new().unwrap();
        let mut p = Phone::new(Default::default());
        p.go(Screen::Compose);
        p.compose.recipient = false;
        p.compose.body.insert("你");
        render(&mut c, &mut p);
        let message_delete = region(&c, 208, 284, 32, 36);
        p.go(Screen::Browser);
        p.overlay = Some(Overlay::Text {
            target: TextTarget::Address,
            editor: Box::new(Editor::new("test", 4096)),
        });
        render(&mut c, &mut p);
        assert_eq!(region(&c, 208, 284, 32, 36), message_delete);
        // A grid without a center action must not inherit the editor's label.
        p.go(Screen::Menu);
        render(&mut c, &mut p);
        assert!(
            region(&c, 82, 284, 76, 36)
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 0, 0, 255])
        );
    }
}
