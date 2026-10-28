//! Shared modal dispatch; app-specific actions stay in their own modules.
use super::*;
impl Shell {
    pub(super) fn overlay_event(&mut self, key: Key, now: u64) {
        let overlay = self.overlay.take().unwrap();
        match overlay {
            Overlay::Symbols { mut index, parent } => {
                let items = MenuKind::Symbols.items();
                let cols = MenuKind::Symbols.columns();
                match key {
                    Key::Up => index = index.saturating_sub(cols),
                    Key::Down => index = (index + cols).min(items.len() - 1),
                    Key::Left => index = index.saturating_sub(1),
                    Key::Right => index = (index + 1).min(items.len() - 1),
                    Key::SoftRight | Key::End => {
                        self.overlay = parent.map(|p| *p);
                        return;
                    }
                    Key::Center | Key::SoftLeft => {
                        self.overlay = parent.map(|p| *p);
                        self.text(items[index].label);
                        return;
                    }
                    _ => {}
                }
                self.overlay = Some(Overlay::Symbols { index, parent });
            }
            Overlay::TimerElapsed => {
                if key == Key::Center {
                    self.tools.timer.start(self.tools.timer_seconds);
                    self.tools.timer_notice = false;
                    self.tools.dirty = true;
                } else if matches!(key, Key::SoftRight | Key::End) {
                    self.tools.timer = Default::default();
                    self.tools.timer_notice = false;
                    self.tools.timer_feedback = Some(("计时已停止", 2000));
                    self.tools.dirty = true;
                } else {
                    self.overlay = Some(Overlay::TimerElapsed);
                }
            }
            Overlay::ServiceAlert {
                backend,
                id,
                title,
                body,
            } => {
                if matches!(key, Key::Center | Key::SoftLeft | Key::SoftRight | Key::End) {
                    if let Some(a) = self
                        .messaging
                        .mailbox
                        .services
                        .alerts
                        .iter_mut()
                        .find(|a| a.backend == backend && a.alert.id == id)
                    {
                        a.dismissed = true;
                        self.messaging.dirty = true;
                    }
                } else {
                    if key == Key::Up {
                        self.notice_scroll = self.notice_scroll.saturating_sub(1);
                    }
                    if key == Key::Down {
                        self.notice_scroll =
                            (self.notice_scroll + 1).min(self.notice_lines.saturating_sub(11));
                    }
                    self.overlay = Some(Overlay::ServiceAlert {
                        backend,
                        id,
                        title,
                        body,
                    });
                }
            }
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
                let n = kind.items().len();
                match key {
                    Key::Up if kind.columns() == 1 => index = (index + n - 1) % n,
                    Key::Up => index = index.saturating_sub(kind.columns()),
                    Key::Down if kind.columns() == 1 => index = (index + 1) % n,
                    Key::Down => index = (index + kind.columns()).min(n - 1),
                    Key::Left if kind.columns() > 1 => index = index.saturating_sub(1),
                    Key::Right if kind.columns() > 1 => index = (index + 1).min(n - 1),
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
                        let item = self.menu_item(kind, index);
                        if item.enabled {
                            self.menu_action(item.action);
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
                    Key::SoftLeft if matches!(target, TextTarget::FileName(_)) => return,
                    Key::End => return,
                    Key::SoftRight if editor.units.is_empty() => return,
                    Key::SoftRight => editor.backspace(),
                    Key::SoftLeft | Key::Center => {
                        match target {
                            TextTarget::Setting(i) => self.setting_text(i, editor.text()),
                            TextTarget::ContactGroup => self.contact_group_name(editor.text()),
                            TextTarget::ContactGroupRename => {
                                self.rename_contact_group(editor.text())
                            }
                            TextTarget::PlaylistName => self.playlist_name(editor.text()),
                            TextTarget::BrowserHome => {
                                self.browsing.data.settings.home_url = editor.text();
                                self.browsing.dirty = true;
                            }
                            TextTarget::Address => {
                                let text = editor.text();
                                let url = if text.contains(':') {
                                    text
                                } else {
                                    format!("http://{text}")
                                };
                                self.browsing.view = BrowserView::Page;
                                self.browsing.pending = Some(url);
                            }
                            TextTarget::FileName(new) => self.file_name(new, editor.text()),
                            TextTarget::Tool(i) => self.tool_text(i, editor.text()),
                            TextTarget::Organizer(i) => self.organizer_text(i, editor.text()),
                            TextTarget::Field(i) => {
                                if let Some(Element::Field { value, .. }) =
                                    self.browsing.model.page.elements.get_mut(i)
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
                    self.browsing.view = BrowserView::Start;
                    self.browsing.model.clear_navigation();
                    self.index = 0;
                }
                Key::SoftRight | Key::End => {}
                _ => self.overlay = Some(Overlay::ClosePage),
            },
            Overlay::CloseBrowser => match key {
                Key::Center => {
                    self.browsing.view = BrowserView::Start;
                    self.browsing.model.clear_navigation();
                    self.leave_app();
                }
                Key::SoftRight | Key::End => {}
                _ => self.overlay = Some(Overlay::CloseBrowser),
            },
        }
    }
    fn menu_action(&mut self, action: menu::Action) {
        use menu::Action::*;
        match action {
            Open(kind) => self.overlay = Some(Overlay::Menu { kind, index: 0 }),
            DialContacts => self.launch(Screen::Contacts),
            DialBack => self.leave_dialer(),
            Attachment => self.notice("附件服务暂未接入。"),
            RemoveAttachment | KeepEditing => {}
            ClearText => self.messaging.compose.body.clear(),
            SaveDraft => self.save_and_close(),
            SelectRecipient => {
                self.contacts.pick_recipient = true;
                self.contacts.page = contacts::Page::Contacts;
                self.go(Screen::Contacts);
            }
            NoGroups => self.notice("没有群组。"),
            Insert(value) => {
                if !self.messaging.compose.body.insert(value) {
                    self.status = "Text limit reached".into();
                }
            }
            DiscardDraft => self.go(Screen::Messages),
            Browser(action) => self.browser_menu_action(action),
            Music(action) => self.music_action(action),
            RefreshMusic => {
                self.playback.refresh = true;
                self.playback.wait_dismissed = false;
                self.playback.view = MusicView::Root;
                self.index = 0;
            }
        }
    }
    fn commit_select(&mut self, i: usize, flags: Vec<bool>) {
        if let Some(Element::Select { selected, .. }) = self.browsing.model.page.elements.get_mut(i)
        {
            *selected = flags;
        }
    }
}
