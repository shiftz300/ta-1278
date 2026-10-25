//! Shared modal dispatch; app-specific actions stay in their own modules.
use super::*;
impl Phone {
    pub(super) fn overlay_event(&mut self, key: Key, now: u64) {
        let overlay = self.overlay.take().unwrap();
        match overlay {
            Overlay::Notice(s) => {
                if key == Key::Up {
                    self.notice_scroll = self.notice_scroll.saturating_sub(1);
                }
                if key == Key::Down {
                    self.notice_scroll =
                        (self.notice_scroll + 1).min(self.notice_lines.saturating_sub(11));
                }
                if !matches!(key, Key::Center | Key::SoftRight | Key::SoftLeft | Key::End) {
                    self.overlay = Some(Overlay::Notice(s));
                }
            }
            Overlay::Menu { kind, mut index } => {
                let n = self.menu_labels(kind).len();
                match key {
                    Key::Up if Self::menu_columns(kind) == 1 => index = (index + n - 1) % n,
                    Key::Up => index = index.saturating_sub(Self::menu_columns(kind)),
                    Key::Down if Self::menu_columns(kind) == 1 => index = (index + 1) % n,
                    Key::Down => index = (index + Self::menu_columns(kind)).min(n - 1),
                    Key::Left if Self::menu_columns(kind) > 1 => index = index.saturating_sub(1),
                    Key::Right if Self::menu_columns(kind) > 1 => index = (index + 1).min(n - 1),
                    Key::SoftRight | Key::End => {
                        let parent = match kind {
                            MenuKind::SmsInsert => Some((MenuKind::Sms, 0)),
                            MenuKind::Smileys | MenuKind::Symbols | MenuKind::Templates => None,
                            MenuKind::BrowserOpen
                            | MenuKind::BrowserNavigation
                            | MenuKind::BrowserSave => Some((MenuKind::Browser, 0)),
                            _ => None,
                        };
                        self.overlay = parent.map(|(kind, index)| Overlay::Menu { kind, index });
                        return;
                    }
                    Key::Center | Key::SoftLeft => {
                        if self.menu_enabled(kind, index) {
                            self.menu_action(kind, index);
                        } else {
                            self.overlay = Some(Overlay::Menu { kind, index });
                        }
                        return;
                    }
                    _ => {}
                }
                self.overlay = Some(Overlay::Menu { kind, index });
            }
            Overlay::Text { target, mut editor } => {
                match key {
                    Key::End => return,
                    Key::SoftRight if editor.units.is_empty() => return,
                    Key::SoftRight => editor.backspace(),
                    Key::SoftLeft | Key::Center => {
                        match target {
                            TextTarget::BrowserHome => {
                                self.browser_data.settings.home_url = editor.text();
                                self.browser_dirty = true;
                            }
                            TextTarget::Address => {
                                let text = editor.text();
                                let url = if text.contains(':') {
                                    text
                                } else {
                                    format!("http://{text}")
                                };
                                self.browser_view = BrowserView::Page;
                                self.pending_url = Some(url);
                            }
                            TextTarget::Tool(i) => self.tool_text(i, editor.text()),
                            TextTarget::Organizer(i) => self.organizer_text(i, editor.text()),
                            TextTarget::Field(i) => {
                                if let Some(Element::Field { value, .. }) =
                                    self.browser.page.elements.get_mut(i)
                                {
                                    *value = editor.text();
                                }
                            }
                        }
                        return;
                    }
                    _ => {}
                }
                editor.tick(now);
                self.overlay = Some(Overlay::Text { target, editor });
                self.edit_key(key, now);
            }
            Overlay::Select {
                element,
                mut index,
                mut selected,
                multiple,
            } => {
                let n = selected.len();
                match key {
                    Key::Up => index = index.saturating_sub(1),
                    Key::Down => index = (index + 1).min(n.saturating_sub(1)),
                    Key::Center | Key::SoftLeft if n > 0 => {
                        if multiple {
                            selected[index] = !selected[index];
                        } else {
                            selected.fill(false);
                            selected[index] = true;
                            self.commit_select(element, selected);
                            return;
                        }
                    }
                    Key::SoftRight | Key::End => {
                        if multiple {
                            self.commit_select(element, selected);
                        }
                        return;
                    }
                    _ => {}
                }
                self.overlay = Some(Overlay::Select {
                    element,
                    index,
                    selected,
                    multiple,
                });
            }
            Overlay::ClosePage => match key {
                Key::Center | Key::SoftLeft => {
                    self.browser_view = BrowserView::Start;
                    self.browser.clear_navigation();
                    self.index = 0;
                }
                Key::SoftRight | Key::End => {}
                _ => self.overlay = Some(Overlay::ClosePage),
            },
            Overlay::CloseBrowser => match key {
                Key::Center => {
                    self.browser_view = BrowserView::Start;
                    self.browser.clear_navigation();
                    self.leave_app();
                }
                Key::SoftRight | Key::End => {}
                _ => self.overlay = Some(Overlay::CloseBrowser),
            },
        }
    }
    fn menu_action(&mut self, kind: MenuKind, index: usize) {
        match kind {
            MenuKind::Sms => match index {
                0 => {
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::SmsInsert,
                        index: 0,
                    })
                }
                1 => self.notice("附件服务暂未接入。"),
                2 => {}
                3 => self.compose.body.clear(),
                _ => self.save_and_close(),
            },
            MenuKind::SmsRecipient => {
                if index == 0 {
                    self.organizer.pick_recipient = true;
                    self.organizer.page = organizer::OrganizerPage::Contacts;
                    self.go(Screen::Contacts);
                } else {
                    self.notice("没有群组。")
                }
            }
            MenuKind::SmsInsert => {
                self.overlay = Some(Overlay::Menu {
                    kind: [MenuKind::Smileys, MenuKind::Symbols, MenuKind::Templates][index],
                    index: 0,
                })
            }
            MenuKind::Symbols | MenuKind::Smileys | MenuKind::Templates => {
                let value = self.menu_labels(kind)[index].clone();
                if !self
                    .compose
                    .body
                    .insert(if value == "换行" { "\n" } else { &value })
                {
                    self.status = "Text limit reached".into();
                }
            }
            MenuKind::ExitSms => match index {
                0 => {
                    self.save_and_close();
                }
                1 => self.go(Screen::Messages),
                _ => {}
            },
            MenuKind::Browser
            | MenuKind::BrowserRoot
            | MenuKind::BrowserOpen
            | MenuKind::BrowserNavigation
            | MenuKind::BrowserSave => self.browser_menu_action(kind, index),
            MenuKind::Music => self.music_action(index),
            MenuKind::MusicLibrary => {
                self.music_view = MusicView::Root;
                self.index = 0;
            }
        }
    }
    fn commit_select(&mut self, i: usize, flags: Vec<bool>) {
        if let Some(Element::Select { selected, .. }) = self.browser.page.elements.get_mut(i) {
            *selected = flags;
        }
    }
}
