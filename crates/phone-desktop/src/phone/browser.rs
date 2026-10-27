//! Handset start/page separation and local browser UI. Transport lives in web.rs.
pub mod data;
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserView {
    Start,
    Page,
    Bookmarks,
    History,
    Offline,
    Settings,
    HomeType,
    FontSize,
    Clear,
    Restore,
}
impl Phone {
    pub fn browser_items(&self) -> Vec<String> {
        match self.browser_view {
            BrowserView::Start => ["输入URL", "书签", "历史", "离线页面"]
                .map(str::to_owned)
                .to_vec(),
            BrowserView::Bookmarks => self
                .browser_data
                .bookmarks
                .iter()
                .map(|b| b.0.clone())
                .collect(),
            BrowserView::History => self
                .browser_data
                .history
                .iter()
                .map(|p| p.0.clone())
                .collect(),
            BrowserView::Offline => self
                .browser_data
                .offline
                .iter()
                .map(|p| p.title.clone())
                .collect(),
            BrowserView::Page => vec![],
            BrowserView::Settings => [
                "SIM1 接入点",
                "SIM2 接入点",
                "主页类型",
                "自定义主页",
                "下载图片",
                "字体大小",
                "清除",
                "恢复预设设置",
            ]
            .map(str::to_owned)
            .to_vec(),
            BrowserView::HomeType => ["自定义"].map(str::to_owned).to_vec(),
            BrowserView::FontSize => ["小号字体", "普通字体"].map(str::to_owned).to_vec(),
            BrowserView::Clear => ["历史", "缓存"].map(str::to_owned).to_vec(),
            BrowserView::Restore => ["取消", "确认"].map(str::to_owned).to_vec(),
        }
    }
    fn address_editor(&mut self) {
        let value = if self.browser_view == BrowserView::Page {
            self.browser.page.url.as_str()
        } else {
            "http://"
        };
        self.overlay = Some(Overlay::Text {
            target: TextTarget::Address,
            editor: Box::new(Editor::new(value, 512)),
        });
    }
    pub(super) fn browser_menu_action(&mut self, kind: MenuKind, index: usize) {
        let submenu = |kind| Some(Overlay::Menu { kind, index: 0 });
        match (kind, index) {
            (MenuKind::BrowserRoot, 1) | (MenuKind::Browser, 6) => {
                self.browser_settings_origin = self.browser_view;
                self.browser_view = BrowserView::Settings;
                self.index = 0;
            }
            (MenuKind::BrowserRoot, 0) | (MenuKind::BrowserOpen, 0) => {
                self.browser_view = BrowserView::Start;
                self.index = 0;
                if self.browser_data.settings.custom_home {
                    self.pending_url = Some(self.browser_data.settings.home_url.clone());
                    self.browser_view = BrowserView::Page;
                }
            }
            (MenuKind::BrowserRoot, 2) => self.overlay = Some(Overlay::CloseBrowser),
            (MenuKind::Browser, 0) => self.overlay = submenu(MenuKind::BrowserOpen),
            (MenuKind::Browser, 1) => self.overlay = submenu(MenuKind::BrowserNavigation),
            (MenuKind::Browser, 2) => self.overlay = submenu(MenuKind::BrowserSave),
            (MenuKind::Browser, 3) => self.notice("没有下载任务"),
            (MenuKind::Browser, 4) => {
                // Sharing prepares a new draft; it never dispatches a service command.
                if self.browser.page.url.is_empty() {
                    return;
                }
                if !self.compose.body.accept_prediction() || !self.compose.to.accept_prediction() {
                    self.notice("请先处理尚未完成的信息输入");
                    return;
                }
                let previous = self.message();
                let already_queued = self.compose.draft.is_none()
                    && self.mailbox.outbox.iter().any(|m| {
                        m.time == previous.time && m.to == previous.to && m.body == previous.body
                    });
                if !already_queued && (!previous.body.is_empty() || !previous.to.is_empty()) {
                    self.save_draft();
                }
                self.compose = Compose::from_message(
                    &Message {
                        body: self.browser.page.url.clone(),
                        time: self.timestamp(),
                        ..Default::default()
                    },
                    None,
                );
                self.chat = Default::default();
                self.go(Screen::Compose);
            }
            (MenuKind::Browser, 5) => self.notice(format!(
                "{}\n{}",
                self.browser.page.title, self.browser.page.url
            )),
            (MenuKind::Browser, 7) => self.overlay = Some(Overlay::ClosePage),
            (MenuKind::BrowserOpen, 1) => self.address_editor(),
            (MenuKind::BrowserOpen, 2..=4) => {
                self.browser_view = [
                    BrowserView::Bookmarks,
                    BrowserView::History,
                    BrowserView::Offline,
                ][index - 2];
                self.index = 0;
            }
            (MenuKind::BrowserNavigation, 0) => {
                self.pending_url = Some(self.browser.page.url.clone())
            }
            (MenuKind::BrowserNavigation, 1) => {
                self.browser.stop();
            }
            (MenuKind::BrowserNavigation, 2) => {
                self.browser.forward();
            }
            (MenuKind::BrowserNavigation, 3) => {
                if self.browser.back() == Back::Close {
                    self.overlay = Some(Overlay::ClosePage);
                }
            }
            (MenuKind::BrowserSave, 0) => {
                self.browser_data.settings.home_url = self.browser.page.url.clone();
                self.browser_data.settings.custom_home = true;
                self.browser_dirty = true;
                self.notice("已设为主页");
            }
            (MenuKind::BrowserSave, 1) => {
                if !self
                    .browser_data
                    .bookmarks
                    .iter()
                    .any(|b| b.1 == self.browser.page.url)
                {
                    self.browser_data.bookmarks.push((
                        self.browser.page.title.clone(),
                        self.browser.page.url.clone(),
                    ));
                }
                self.browser_data.bookmarks.truncate(1000);
                self.browser_dirty = true;
                self.notice("书签已添加");
            }
            (MenuKind::BrowserSave, 2) => {
                self.browser_data.save_page(&self.browser.page);
                self.browser_dirty = true;
                self.notice("离线页面已保存");
            }
            _ => self.notice("浏览器设置尚未实现。"),
        }
    }
    pub(super) fn browser_key(&mut self, key: Key) {
        if matches!(
            self.browser_view,
            BrowserView::Settings
                | BrowserView::HomeType
                | BrowserView::FontSize
                | BrowserView::Clear
                | BrowserView::Restore
        ) {
            self.browser_settings_key(key);
            return;
        }
        if self.browser_view != BrowserView::Page {
            let n = self.browser_items().len();
            match key {
                Key::Up | Key::Down => {
                    vertical(&mut self.index, n, key, Boundary::Wrap);
                }
                Key::SoftLeft => {
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::BrowserRoot,
                        index: 0,
                    })
                }
                Key::SoftRight if self.browser_view == BrowserView::Start => {
                    self.overlay = Some(Overlay::CloseBrowser)
                }
                Key::SoftRight => {
                    self.browser_view = BrowserView::Start;
                    self.index = 0;
                }
                Key::Delete if n > 0 => {
                    match self.browser_view {
                        BrowserView::Bookmarks => {
                            self.browser_data.bookmarks.remove(self.index);
                        }
                        BrowserView::History => {
                            self.browser_data.history.remove(self.index);
                        }
                        BrowserView::Offline => {
                            self.browser_data.offline.remove(self.index);
                        }
                        _ => return,
                    }
                    self.index = self.index.min(n.saturating_sub(2));
                    self.browser_dirty = true;
                }
                Key::Center if n > 0 => match self.browser_view {
                    BrowserView::Start => match self.index {
                        0 => self.address_editor(),
                        i => {
                            self.browser_view = [
                                BrowserView::Bookmarks,
                                BrowserView::History,
                                BrowserView::Offline,
                            ][i - 1];
                            self.index = 0;
                        }
                    },
                    BrowserView::Bookmarks => {
                        self.pending_url = Some(self.browser_data.bookmarks[self.index].1.clone());
                        self.browser_view = BrowserView::Page;
                    }
                    BrowserView::History => {
                        self.pending_url = Some(self.browser_data.history[self.index].1.clone());
                        self.browser_view = BrowserView::Page;
                    }
                    BrowserView::Offline => {
                        self.browser = Browser::new(self.browser_data.offline[self.index].clone());
                        self.browser_view = BrowserView::Page;
                    }
                    _ => unreachable!(),
                },
                _ => {}
            }
            return;
        }
        match key {
            Key::Up => self.browser.move_focus(-1),
            Key::Down => self.browser.move_focus(1),
            Key::Left => self.browser.scroll = self.browser.scroll.saturating_sub(180),
            Key::Right => {
                self.browser.scroll =
                    (self.browser.scroll + 180).min(self.browser.content_height.saturating_sub(216))
            }
            Key::SoftLeft => {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::Browser,
                    index: 0,
                })
            }
            Key::SoftRight => {
                if self.browser.back() == Back::Close {
                    self.overlay = Some(Overlay::ClosePage);
                }
            }
            Key::Center => {
                let i = self.browser.focus;
                match self.browser.page.elements.get(i).cloned() {
                    Some(Element::Link { url, .. }) => self.pending_url = Some(url),
                    Some(Element::Field {
                        value,
                        limit,
                        read_only: false,
                        ..
                    }) => {
                        let mut editor = Editor::new(&value, limit);
                        if matches!(
                            self.browser.page.elements.get(i),
                            Some(Element::Field { password: true, .. })
                        ) {
                            editor.prediction.set_learning_enabled(false);
                        }
                        // Native XHTML constructor 0x88b48fa0 caps the initial
                        // copy too. Keep the host's scalar-safe UTF-16 policy.
                        editor.units.truncate(limit);
                        if editor
                            .units
                            .last()
                            .is_some_and(|u| (0xd800..=0xdbff).contains(u))
                        {
                            editor.units.pop();
                        }
                        editor.cursor = editor.units.len();
                        editor
                            .prediction
                            .set_language(Some(phone_core::predict::Language::Chinese));
                        self.overlay = Some(Overlay::Text {
                            target: TextTarget::Field(i),
                            editor: Box::new(editor),
                        });
                    }
                    Some(Element::Check { .. }) => self.browser.toggle_check(i),
                    Some(Element::Select {
                        selected, multiple, ..
                    }) => {
                        self.overlay = Some(Overlay::Select {
                            element: i,
                            index: 0,
                            selected,
                            multiple,
                        })
                    }
                    Some(Element::Reset { form }) => self.browser.reset(form),
                    Some(Element::Submit { form, .. }) => match web::submit(
                        &self.browser.page,
                        form,
                        self.browser.form_values(form, Some(i)),
                    ) {
                        Ok(url) => self.pending_url = Some(url),
                        Err(e) => self.notice(e),
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
    fn browser_settings_key(&mut self, key: Key) {
        use BrowserView::*;
        let n = self.browser_items().len();
        if self.browser_view == Clear && key == Key::SoftLeft {
            if self.browser_clear[0] {
                self.browser_data.history.clear();
                self.browser_dirty = true;
            }
            if self.browser_clear[1] {
                self.browser.clear_cache();
            }
            self.browser_view = Settings;
            self.index = 6;
            self.notice("清除完成");
            return;
        }
        match key {
            Key::Up => self.index = (self.index + n - 1) % n,
            Key::Down => self.index = (self.index + 1) % n,
            Key::SoftRight => {
                let previous = self.browser_view;
                self.browser_view = match self.browser_view {
                    Settings => self.browser_settings_origin,
                    _ => Settings,
                };
                self.index = match self.browser_view {
                    Settings => match previous {
                        HomeType => 2,
                        FontSize => 5,
                        Clear => 6,
                        Restore => 7,
                        _ => 0,
                    },
                    _ => 0,
                };
            }
            Key::Center => match self.browser_view {
                Settings => match self.index {
                    0 | 1 => self.notice("没有移动网络接入点。网页使用电脑的网络连接。"),
                    2 => {
                        self.browser_view = HomeType;
                        self.index = 0;
                    }
                    3 => self.open_local_text(
                        TextTarget::BrowserHome,
                        self.browser_data.settings.home_url.clone(),
                        false,
                    ),
                    4 => {
                        self.browser_data.settings.download_images =
                            !self.browser_data.settings.download_images;
                        self.browser_dirty = true;
                    }
                    5 => {
                        self.browser_view = FontSize;
                        self.index = self.browser_data.settings.font_size.min(1);
                    }
                    6 => {
                        self.browser_view = Clear;
                        self.browser_clear = [false, true];
                        self.index = 0;
                    }
                    _ => {
                        self.browser_view = Restore;
                        self.index = 0;
                    }
                },
                HomeType => {
                    self.browser_data.settings.custom_home = true;
                    self.browser_view = Settings;
                    self.index = 2;
                    self.browser_dirty = true;
                }
                FontSize => {
                    self.browser_data.settings.font_size = self.index;
                    self.browser_view = Settings;
                    self.index = 5;
                    self.browser_dirty = true;
                }
                Clear => {
                    self.browser_clear[self.index] = !self.browser_clear[self.index];
                }
                Restore => {
                    if self.index == 1 {
                        self.browser_data.settings = Default::default();
                        self.browser_dirty = true;
                    }
                    self.browser_view = Settings;
                    self.index = 7;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
