//! Handset start/page separation and local browser UI. Transport lives in web.rs.
mod state;
pub use state::State;
pub mod data;
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserView {
    Start,
    Page,
    Details,
    Bookmarks,
    History,
    Offline,
    Settings,
    HomeType,
    FontSize,
    Clear,
    Restore,
}
#[derive(Clone, Copy)]
pub enum Action {
    Settings,
    Home,
    Close,
    Downloads,
    Share,
    Details,
    ClosePage,
    Address,
    List(BrowserView),
    Refresh,
    Stop,
    Forward,
    Back,
    SetHome,
    Bookmark,
    SavePage,
    SaveImage,
}
impl Shell {
    pub fn browser_items(&self) -> Vec<String> {
        match self.browsing.view {
            BrowserView::Start => ["输入URL", "书签", "历史", "离线页面"]
                .map(str::to_owned)
                .to_vec(),
            BrowserView::Bookmarks => self
                .browsing
                .data
                .bookmarks
                .iter()
                .map(|b| b.0.clone())
                .collect(),
            BrowserView::History => self
                .browsing
                .data
                .history
                .iter()
                .map(|p| p.0.clone())
                .collect(),
            BrowserView::Offline => self
                .browsing
                .data
                .offline
                .iter()
                .map(|p| p.title.clone())
                .collect(),
            BrowserView::Page | BrowserView::Details => vec![],
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
        let value = if self.browsing.view == BrowserView::Page {
            self.browsing.model.page.url.as_str()
        } else {
            "http://"
        };
        self.overlay = Some(Overlay::Text {
            target: TextTarget::Address,
            editor: Box::new(Editor::new(value, 512)),
        });
    }
    pub(super) fn browser_menu_action(&mut self, action: Action) {
        match action {
            Action::Settings => {
                self.browsing.settings_origin = self.browsing.view;
                self.browsing.view = BrowserView::Settings;
                self.index = 0;
            }
            Action::Home => {
                self.browsing.view = BrowserView::Start;
                self.index = 0;
                if self.browsing.data.settings.custom_home {
                    self.browsing.pending = Some(self.browsing.data.settings.home_url.clone());
                    self.browsing.view = BrowserView::Page;
                }
            }
            Action::Close => self.overlay = Some(Overlay::CloseBrowser),
            Action::Downloads => self.notice("没有下载任务"),
            Action::Share => {
                // Sharing prepares a new draft; it never dispatches a service command.
                if self.browsing.model.page.url.is_empty() {
                    return;
                }
                if !self.messaging.compose.body.accept_prediction()
                    || !self.messaging.compose.to.accept_prediction()
                {
                    self.notice("请先处理尚未完成的信息输入");
                    return;
                }
                let previous = self.message();
                let already_queued = self.messaging.compose.draft.is_none()
                    && self.messaging.mailbox.outbox.iter().any(|m| {
                        m.time == previous.time && m.to == previous.to && m.body == previous.body
                    });
                if !already_queued && (!previous.body.is_empty() || !previous.to.is_empty()) {
                    self.save_draft();
                }
                self.messaging.compose = Compose::from_message(
                    &Message {
                        body: self.browsing.model.page.url.clone(),
                        time: self.timestamp(),
                        ..Default::default()
                    },
                    None,
                );
                self.messaging.chat = Default::default();
                self.go(Screen::Compose);
            }
            Action::Details => {
                self.browsing.view = BrowserView::Details;
                self.index = 0;
            }
            Action::ClosePage => self.overlay = Some(Overlay::ClosePage),
            Action::Address => self.address_editor(),
            Action::List(view) => {
                self.browsing.view = view;
                self.index = 0;
            }
            Action::Refresh => self.browsing.pending = Some(self.browsing.model.page.url.clone()),
            Action::Stop => {
                self.browsing.model.stop();
            }
            Action::Forward => {
                self.browsing.model.forward();
            }
            Action::Back => {
                if self.browsing.model.back() == Back::Close {
                    self.overlay = Some(Overlay::ClosePage);
                }
            }
            Action::SetHome => {
                self.browsing.data.settings.home_url = self.browsing.model.page.url.clone();
                self.browsing.data.settings.custom_home = true;
                self.browsing.dirty = true;
                self.notice("已设为主页");
            }
            Action::Bookmark => {
                if !self
                    .browsing
                    .data
                    .bookmarks
                    .iter()
                    .any(|b| b.1 == self.browsing.model.page.url)
                {
                    self.browsing.data.bookmarks.push((
                        self.browsing.model.page.title.clone(),
                        self.browsing.model.page.url.clone(),
                    ));
                }
                self.browsing.data.bookmarks.truncate(1000);
                self.browsing.dirty = true;
                self.notice("书签已添加");
            }
            Action::SavePage => {
                self.browsing.data.save_page(&self.browsing.model.page);
                self.browsing.dirty = true;
                self.notice("离线页面已保存");
            }
            Action::SaveImage => self.notice("浏览器设置尚未实现。"),
        }
    }
    pub(super) fn browser_key(&mut self, key: Key) {
        if self.browsing.view == BrowserView::Details {
            match key {
                Key::SoftRight | Key::End => self.browsing.view = BrowserView::Page,
                Key::Up => self.index = self.index.saturating_sub(1),
                Key::Down => self.index = self.index.saturating_add(1),
                _ => {}
            }
            return;
        }
        if matches!(
            self.browsing.view,
            BrowserView::Settings
                | BrowserView::HomeType
                | BrowserView::FontSize
                | BrowserView::Clear
                | BrowserView::Restore
        ) {
            self.browser_settings_key(key);
            return;
        }
        if self.browsing.view != BrowserView::Page {
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
                Key::SoftRight if self.browsing.view == BrowserView::Start => {
                    self.overlay = Some(Overlay::CloseBrowser)
                }
                Key::SoftRight => {
                    self.browsing.view = BrowserView::Start;
                    self.index = 0;
                }
                Key::Delete if n > 0 => {
                    match self.browsing.view {
                        BrowserView::Bookmarks => {
                            self.browsing.data.bookmarks.remove(self.index);
                        }
                        BrowserView::History => {
                            self.browsing.data.history.remove(self.index);
                        }
                        BrowserView::Offline => {
                            self.browsing.data.offline.remove(self.index);
                        }
                        _ => return,
                    }
                    self.index = self.index.min(n.saturating_sub(2));
                    self.browsing.dirty = true;
                }
                Key::Center if n > 0 => match self.browsing.view {
                    BrowserView::Start => match self.index {
                        0 => self.address_editor(),
                        i => {
                            self.browsing.view = [
                                BrowserView::Bookmarks,
                                BrowserView::History,
                                BrowserView::Offline,
                            ][i - 1];
                            self.index = 0;
                        }
                    },
                    BrowserView::Bookmarks => {
                        self.browsing.pending =
                            Some(self.browsing.data.bookmarks[self.index].1.clone());
                        self.browsing.view = BrowserView::Page;
                    }
                    BrowserView::History => {
                        self.browsing.pending =
                            Some(self.browsing.data.history[self.index].1.clone());
                        self.browsing.view = BrowserView::Page;
                    }
                    BrowserView::Offline => {
                        self.browsing.model =
                            Browser::new(self.browsing.data.offline[self.index].clone());
                        self.browsing.view = BrowserView::Page;
                    }
                    _ => unreachable!(),
                },
                _ => {}
            }
            return;
        }
        match key {
            Key::Up => self.browsing.model.move_focus(-1),
            Key::Down => self.browsing.model.move_focus(1),
            Key::Left => {
                self.browsing.model.scroll = self.browsing.model.scroll.saturating_sub(180)
            }
            Key::Right => {
                self.browsing.model.scroll = (self.browsing.model.scroll + 180)
                    .min(self.browsing.model.content_height.saturating_sub(216))
            }
            Key::SoftLeft => {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::Browser,
                    index: 0,
                })
            }
            Key::SoftRight => {
                if self.browsing.model.back() == Back::Close {
                    self.overlay = Some(Overlay::ClosePage);
                }
            }
            Key::Center => {
                let i = self.browsing.model.focus;
                match self.browsing.model.page.elements.get(i).cloned() {
                    Some(Element::Link { url, .. }) => self.browsing.pending = Some(url),
                    Some(Element::Field {
                        value,
                        limit,
                        read_only: false,
                        ..
                    }) => {
                        let mut editor = Editor::new(&value, limit);
                        if matches!(
                            self.browsing.model.page.elements.get(i),
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
                    Some(Element::Check { .. }) => self.browsing.model.toggle_check(i),
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
                    Some(Element::Reset { form }) => self.browsing.model.reset(form),
                    Some(Element::Submit { form, .. }) => match web::submit(
                        &self.browsing.model.page,
                        form,
                        self.browsing.model.form_values(form, Some(i)),
                    ) {
                        Ok(url) => self.browsing.pending = Some(url),
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
        if self.browsing.view == Clear && key == Key::SoftLeft {
            if self.browsing.clear[0] {
                self.browsing.data.history.clear();
                self.browsing.dirty = true;
            }
            if self.browsing.clear[1] {
                self.browsing.model.clear_cache();
            }
            self.browsing.view = Settings;
            self.index = 6;
            self.notice("清除完成");
            return;
        }
        match key {
            Key::Up => self.index = (self.index + n - 1) % n,
            Key::Down => self.index = (self.index + 1) % n,
            Key::SoftRight => {
                let previous = self.browsing.view;
                self.browsing.view = match self.browsing.view {
                    Settings => self.browsing.settings_origin,
                    _ => Settings,
                };
                self.index = match self.browsing.view {
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
            Key::Center => match self.browsing.view {
                Settings => match self.index {
                    0 | 1 => self.notice("没有移动网络接入点。网页使用电脑的网络连接。"),
                    2 => {
                        self.browsing.view = HomeType;
                        self.index = 0;
                    }
                    3 => self.open_local_text(
                        TextTarget::BrowserHome,
                        self.browsing.data.settings.home_url.clone(),
                        false,
                    ),
                    4 => {
                        self.browsing.data.settings.download_images =
                            !self.browsing.data.settings.download_images;
                        self.browsing.dirty = true;
                    }
                    5 => {
                        self.browsing.view = FontSize;
                        self.index = self.browsing.data.settings.font_size.min(1);
                    }
                    6 => {
                        self.browsing.view = Clear;
                        self.browsing.clear = [false, true];
                        self.index = 0;
                    }
                    _ => {
                        self.browsing.view = Restore;
                        self.index = 0;
                    }
                },
                HomeType => {
                    self.browsing.data.settings.custom_home = true;
                    self.browsing.view = Settings;
                    self.index = 2;
                    self.browsing.dirty = true;
                }
                FontSize => {
                    self.browsing.data.settings.font_size = self.index;
                    self.browsing.view = Settings;
                    self.index = 5;
                    self.browsing.dirty = true;
                }
                Clear => {
                    self.browsing.clear[self.index] = !self.browsing.clear[self.index];
                }
                Restore => {
                    if self.index == 1 {
                        self.browsing.data.settings = Default::default();
                        self.browsing.dirty = true;
                    }
                    self.browsing.view = Settings;
                    self.index = 7;
                }
                _ => {}
            },
            _ => {}
        }
    }
}
