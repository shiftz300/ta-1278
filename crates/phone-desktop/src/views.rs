mod alarm;
mod browser;
mod calls;
mod compose;
mod contacts;
mod conversation;
mod converter;
mod files;
mod mail;
mod media;
mod music;
mod organizer;
mod pickers;
mod radio;
mod settings;
pub(crate) mod snake;
mod tools;
mod wallpaper;
use crate::ui::chrome::status;
use crate::ui::popup::Popup as PopupMenu;
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
    wallpaper::draw(c, p);
    status(c);
    match p.screen {
        Screen::Home => {
            let (date, minutes) = p.local_clock();
            let hour = minutes / 60;
            let time = format!(
                "{:02}:{:02}",
                if p.preferences.clock_12h {
                    (hour + 11) % 12 + 1
                } else {
                    hour
                },
                minutes % 60
            );
            let w = c.width(&time, 40);
            c.text((240 - w) / 2, 42, &time, 40, WHITE);
            let s = format!(
                "星期{}, {:04}.{:02}.{:02}",
                ["一", "二", "三", "四", "五", "六", "日"][date.weekday()],
                date.year,
                date.month,
                date.day
            );
            let w = c.width(&s, 14);
            c.text((240 - w) / 2, 96, &s, 14, WHITE);
            if !p.preferences.greeting.is_empty() {
                c.fit(6, 190, &p.preferences.greeting, 18, WHITE, 228);
            }
            Softkeys::new(
                Button::Shortcut(p.visible_shortcut(true)),
                Button::Text("功能表"),
                Button::Shortcut(p.visible_shortcut(false)),
            )
            .draw(c);
        }
        Screen::Menu => {
            let entries = crate::menu::visible(p.apps);
            p.index = p.index.min(entries.len() - 1);
            c.text(6, 1, entries[p.index].label, 14, WHITE);
            let first_row = (p.index / 3).saturating_sub(2);
            for (i, entry) in entries.iter().enumerate().skip(first_row * 3).take(12) {
                let x = 38 + (i % 3) as i32 * 81;
                let y = 68 + (i / 3 - first_row) as i32 * 82;
                if p.index == i {
                    c.ring(x, y, 31, c.accent(2));
                }
                c.icon_enabled(entry.icon, x - 38, y - 38, 76, true);
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
        Screen::Media => media::render(c, p),
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
        let file_editor = matches!(
            overlay,
            Overlay::Text {
                target: TextTarget::FileName(_),
                ..
            }
        );
        let music_library_menu = matches!(
            overlay,
            Overlay::Menu {
                kind: MenuKind::MusicLibrary,
                ..
            }
        );
        if browser_modal
            || file_editor
            || music_library_menu
            || matches!(
                overlay,
                Overlay::Menu {
                    kind: MenuKind::Sms | MenuKind::SmsInsert | MenuKind::Dial,
                    ..
                }
            )
            || matches!(overlay, Overlay::TimerElapsed)
        {
            c.dim();
        } else {
            c.background();
        }
        match overlay {
            Overlay::TimerElapsed => tools::timer_elapsed(c, p),
            Overlay::Notice(_) | Overlay::ServiceAlert { .. } => {
                let (title, s) = match &overlay {
                    Overlay::Notice(s) => ("提示", s),
                    Overlay::ServiceAlert { title, body, .. } => (title.as_str(), body),
                    _ => unreachable!(),
                };
                c.fit(12, 51, title, 18, WHITE, 212);
                let lines = c.wrap(s, 15, 212);
                p.notice_lines = lines.len();
                for (i, l) in lines.iter().skip(p.notice_scroll).take(11).enumerate() {
                    c.text(12, 86 + i as i32 * 18, &l.text, 15, WHITE);
                }
                Softkeys::new(Button::Text("确定"), Button::None, Button::Text("关闭")).draw(c);
            }
            Overlay::Menu { kind, index } => {
                if p.screen == Screen::Browser {
                    browser::chrome::menu(c, p, kind, index);
                } else if kind == MenuKind::MusicLibrary {
                    let labels = p.menu_labels(kind);
                    PopupMenu::default()
                        .unshaded()
                        .draw(c, labels.len(), index, |i| list::Row::text(&labels[i]));
                } else if matches!(kind, MenuKind::Sms | MenuKind::SmsInsert | MenuKind::Dial) {
                    let labels = p.menu_labels(kind);
                    let mut popup = PopupMenu::default().unshaded();
                    if kind == MenuKind::SmsInsert {
                        chrome::title(c, "插入选项");
                        popup = popup.at(84);
                    }
                    popup.draw(c, labels.len(), index, |i| list::Row {
                        label: &labels[i],
                        enabled: p.menu_enabled(kind, i),
                        marker: if kind == MenuKind::Sms && i < 2 {
                            list::Marker::Child
                        } else {
                            list::Marker::None
                        },
                    });
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
                    browser::chrome::editor(c, p, target, &e);
                } else if matches!(target, TextTarget::FileName(_)) {
                    c.rect(0, 184, 240, 136, [0, 0, 0, 255]);
                    let folder = target == TextTarget::FileName(true)
                        || p.files
                            .entries
                            .get(p.files.selected)
                            .is_some_and(|e| e.directory);
                    c.text(
                        6,
                        193,
                        if folder {
                            "文件夹名称"
                        } else {
                            "文件名"
                        },
                        20,
                        WHITE,
                    );
                    p.editor_lines = Edit::new([7, 221, 227, 28]).sized(20, 26).draw(c, &e);
                    c.rect(6, 251, 229, 3, c.accent(3));
                    c.text(205, 260, mode(&e), 16, MUTED);
                    Softkeys::new(Button::Close, Button::Text("确认"), Button::Delete).draw(c);
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
                status(c);
                chrome::title(c, "选择");
                if let Some(Element::Select { options, .. }) = p.browser.page.elements.get(element)
                {
                    list::draw(
                        c,
                        options.len(),
                        index,
                        list::ListStyle {
                            font: 16,
                            text_width: 204,
                            marker_shades: Some((6, 3)),
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
        crate::ui::candidates::render(c, e);
    }
    if matches!(p.screen, Screen::Tools(_))
        && matches!(
            p.tools.page,
            crate::phone::tools::ToolPage::Other | crate::phone::tools::ToolPage::TimerMenu
        )
        && p.overlay.is_none()
        && let Some((label, _)) = p.tools.timer_feedback
    {
        c.rect(0, 0, 240, 24, c.accent(5));
        chrome::title(c, label);
    }
    alarm::alert(c, p);
}
fn mode(e: &Editor) -> &'static str {
    e.mode_label()
}

#[cfg(test)]
mod audit;
#[cfg(test)]
mod samples;
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
                Overlay::ServiceAlert {
                    backend: "test".into(),
                    id: "weather".into(),
                    title: "天气提醒".into(),
                    body: "测试预警内容\n请留意天气变化。".into(),
                },
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
        // The triangular tail and rectangular body have one continuous interior.
        assert_eq!(pixel(&c, 35, 254), fill);
        assert_eq!(pixel(&c, 35, 255), fill);
        assert_eq!(pixel(&c, 29, 254), WHITE);
        assert_eq!(pixel(&c, 48, 254), WHITE);
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
            Screen::Tools(crate::phone::tools::ToolPage::Converter),
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
    fn editor_softkeys_use_shared_geometry_with_app_specific_labels() {
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
        let browser_delete = region(&c, 194, 284, 46, 36);
        assert_ne!(region(&c, 208, 284, 32, 36), message_delete);
        c.background();
        Softkeys::new(Button::Text("确认"), Button::None, Button::Text("删除")).draw(&mut c);
        assert_eq!(region(&c, 194, 284, 46, 36), browser_delete);
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
