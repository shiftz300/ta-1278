use crate::web;
use phone_core::{
    browser::{Back, Browser, Element},
    editor::Editor,
    input::{Event, Key, Phase},
    music::Music,
    sms::{Mailbox, Message},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Home,
    Menu,
    Messages,
    Drafts,
    Outbox,
    Compose,
    Browser,
    Music,
    Dial,
    Contacts,
    Settings,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuKind {
    Sms,
    ExitSms,
    Browser,
    Music,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextTarget {
    Address,
    Field(usize),
}
#[derive(Clone, Debug)]
pub enum Overlay {
    Menu {
        kind: MenuKind,
        index: usize,
    },
    Text {
        target: TextTarget,
        editor: Editor,
    },
    Select {
        element: usize,
        index: usize,
        selected: Vec<bool>,
        multiple: bool,
    },
    CloseBrowser,
    Notice(String),
}
pub struct Compose {
    pub to: Editor,
    pub body: Editor,
    pub recipient: bool,
    pub draft: Option<usize>,
}
impl Default for Compose {
    fn default() -> Self {
        let mut to = Editor::new("", 40);
        to.numeric = true;
        Self {
            to,
            body: Editor::default(),
            recipient: true,
            draft: None,
        }
    }
}
pub struct Phone {
    pub screen: Screen,
    pub index: usize,
    pub overlay: Option<Overlay>,
    pub mailbox: Mailbox,
    pub compose: Compose,
    pub browser: Browser,
    pub music: Music,
    pub dial: Editor,
    pub status: String,
    pub pending_url: Option<String>,
    pub editor_lines: Vec<usize>,
    pub dirty: bool,
    pub notice_scroll: usize,
    pub notice_lines: usize,
}
impl Phone {
    pub fn new(mailbox: Mailbox) -> Self {
        let mut dial = Editor::new("", 40);
        dial.numeric = true;
        Self {
            screen: Screen::Home,
            index: 0,
            overlay: None,
            mailbox,
            compose: Compose::default(),
            browser: Browser::new(web::demo()),
            music: Music::default(),
            dial,
            status: String::new(),
            pending_url: None,
            editor_lines: vec![0],
            dirty: false,
            notice_scroll: 0,
            notice_lines: 0,
        }
    }
    pub fn go(&mut self, s: Screen) {
        self.music.event(Event {
            key: Key::End,
            phase: Phase::Cancel,
        });
        self.screen = s;
        self.index = 0;
        self.overlay = None;
    }
    pub fn notice(&mut self, s: impl Into<String>) {
        self.notice_scroll = 0;
        self.notice_lines = 0;
        self.overlay = Some(Overlay::Notice(s.into()));
    }
    pub fn menu_labels(&self, kind: MenuKind) -> Vec<String> {
        match kind {
            MenuKind::Sms => vec![
                "Queue (no service)",
                "Save draft",
                "Insert new line",
                "Clear text",
                "Change input mode",
                "Close message",
            ],
            MenuKind::ExitSms => vec!["Save and close", "Discard changes", "Keep editing"],
            MenuKind::Browser => vec![
                "Open address",
                "Demo page",
                "Reload",
                "Page address",
                "Close browser",
            ],
            MenuKind::Music => vec!["Repeat: cycle", "Shuffle: toggle", "Back to menu"],
        }
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
    pub fn editor(&self) -> Option<&Editor> {
        match &self.overlay {
            Some(Overlay::Text { editor, .. }) => Some(editor),
            Some(_) => None,
            None => match self.screen {
                Screen::Compose => Some(if self.compose.recipient {
                    &self.compose.to
                } else {
                    &self.compose.body
                }),
                Screen::Dial => Some(&self.dial),
                _ => None,
            },
        }
    }
    pub fn editor_mut(&mut self) -> Option<&mut Editor> {
        match &mut self.overlay {
            Some(Overlay::Text { editor, .. }) => Some(editor),
            Some(_) => None,
            None => match self.screen {
                Screen::Compose => Some(if self.compose.recipient {
                    &mut self.compose.to
                } else {
                    &mut self.compose.body
                }),
                Screen::Dial => Some(&mut self.dial),
                _ => None,
            },
        }
    }
    pub fn text(&mut self, s: &str) {
        if let Some(e) = self.editor_mut()
            && !e.insert(s)
        {
            self.status = "Text limit reached".into();
        }
    }
    pub fn tab(&mut self) {
        if self.screen == Screen::Compose && self.overlay.is_none() {
            self.compose.to.commit();
            self.compose.body.commit();
            self.compose.recipient = !self.compose.recipient;
        }
    }
    pub fn input_mode(&mut self) {
        if let Some(e) = self.editor_mut() {
            e.commit();
            if e.numeric {
                e.numeric = false;
                e.uppercase = false;
            } else if e.uppercase {
                e.uppercase = false;
                e.numeric = true;
            } else {
                e.uppercase = true;
            }
        }
    }
    pub fn tick(&mut self, now: u64, dt: f32) {
        if let Some(e) = self.editor_mut() {
            e.tick(now);
        }
        if self.screen != Screen::Music || self.overlay.is_some() {
            self.music.event(Event {
                key: Key::End,
                phase: Phase::Cancel,
            });
        }
        self.music.tick(dt);
    }
    pub fn handle(&mut self, event: Event, now: u64) {
        let Event { key, phase } = event;
        if phase == Phase::Cancel {
            self.music.event(event);
            if let Some(e) = self.editor_mut() {
                e.commit();
            }
            return;
        }
        if self.screen == Screen::Music && self.overlay.is_none() {
            self.music.event(event);
        }
        if phase == Phase::Long {
            if let Key::Digit(d) = key
                && let Some(e) = self.editor_mut()
            {
                e.long_digit(d);
            }
            return;
        }
        if !matches!(phase, Phase::Down | Phase::Repeat) {
            return;
        }
        if phase == Phase::Repeat
            && !matches!(
                key,
                Key::Up | Key::Down | Key::Left | Key::Right | Key::Delete
            )
        {
            return;
        }
        if self.overlay.is_some() {
            self.overlay_event(key, now);
            return;
        }
        if key == Key::End {
            if self.screen == Screen::Compose {
                self.close_compose();
            } else {
                self.go(Screen::Home);
            }
            return;
        }
        if self.editor().is_some() && self.edit_key(key, now) {
            return;
        }
        match self.screen{
   Screen::Home=>match key{Key::Center|Key::SoftLeft=>self.go(Screen::Menu),Key::SoftRight=>self.go(Screen::Contacts),Key::Call=>self.go(Screen::Dial),Key::Digit(d)=>{self.go(Screen::Dial);self.dial.digit(d,now);},_=>{}},
   Screen::Menu=>match key{Key::Up=>self.index=self.index.saturating_sub(2),Key::Down=>self.index=(self.index+2).min(5),Key::Left=>self.index=self.index.saturating_sub(1),Key::Right=>self.index=(self.index+1).min(5),Key::Center|Key::SoftLeft=>self.go([Screen::Messages,Screen::Browser,Screen::Music,Screen::Dial,Screen::Contacts,Screen::Settings][self.index]),Key::SoftRight=>self.go(Screen::Home),_=>{}},
   Screen::Messages=>match key{Key::Up=>self.index=self.index.saturating_sub(1),Key::Down=>self.index=(self.index+1).min(2),Key::Center|Key::SoftLeft=>match self.index{0=>{self.compose=Compose::default();self.go(Screen::Compose);},1=>self.go(Screen::Drafts),_=>self.go(Screen::Outbox)},Key::SoftRight=>self.go(Screen::Menu),_=>{}},
   Screen::Drafts|Screen::Outbox=>self.mail_list_key(key),
   Screen::Compose=>match key{Key::Center if self.compose.recipient=>self.compose.recipient=false,Key::Center|Key::SoftLeft=>self.overlay=Some(Overlay::Menu{kind:MenuKind::Sms,index:0}),Key::SoftRight=>{if self.editor().is_some_and(|e|!e.units.is_empty()){self.editor_mut().unwrap().backspace();}else{self.close_compose();}},Key::Up if self.compose.recipient=>{},_=>{}},
   Screen::Browser=>self.browser_key(key),
   Screen::Music=>match key{Key::SoftLeft=>self.overlay=Some(Overlay::Menu{kind:MenuKind::Music,index:0}),Key::SoftRight=>self.go(Screen::Menu),_=>{}},
   Screen::Dial=>match key{Key::Call|Key::Center|Key::SoftLeft=>self.notice("Phone service is not connected. No call was made."),Key::SoftRight=>{if self.dial.units.is_empty(){self.go(Screen::Menu);}else{self.dial.backspace();}},_=>{}},
   Screen::Contacts=>match key{Key::SoftRight=>self.go(Screen::Menu),Key::Center|Key::SoftLeft=>self.notice("No contacts backend. No device contacts were imported."),_=>{}},
   Screen::Settings=>match key{Key::SoftRight=>self.go(Screen::Menu),Key::Center|Key::SoftLeft=>self.notice("240 x 320 logical display. Multi-tap input. T9 prediction is not implemented. Local data only."),_=>{}}
  }
    }
    fn edit_key(&mut self, key: Key, now: u64) -> bool {
        let lines = self.editor_lines.clone();
        let e = self.editor_mut().unwrap();
        match key {
            Key::Digit(d) => e.digit(d, now),
            Key::Star => {
                e.insert("*");
            }
            Key::Hash => {
                self.input_mode();
            }
            Key::Delete => e.backspace(),
            Key::Left => e.move_horizontal(-1),
            Key::Right => e.move_horizontal(1),
            Key::Up => e.vertical(-1, &lines),
            Key::Down => e.vertical(1, &lines),
            _ => return false,
        }
        true
    }
    pub fn message(&self) -> Message {
        Message {
            to: self.compose.to.text(),
            body: self.compose.body.text(),
        }
    }
    fn save_draft(&mut self) {
        let message = self.message();
        if message.to.is_empty() && message.body.is_empty() {
            return;
        }
        let i = self.mailbox.save(self.compose.draft, message);
        self.compose.draft = Some(i);
        self.dirty = true;
    }
    fn close_compose(&mut self) {
        if self.compose.to.units.is_empty() && self.compose.body.units.is_empty() {
            self.go(Screen::Messages);
        } else {
            self.compose.to.commit();
            self.compose.body.commit();
            self.overlay = Some(Overlay::Menu {
                kind: MenuKind::ExitSms,
                index: 0,
            });
        }
    }
    fn mail_list_key(&mut self, key: Key) {
        let drafts = self.screen == Screen::Drafts;
        let count = if drafts {
            self.mailbox.drafts.len()
        } else {
            self.mailbox.outbox.len()
        };
        match key {
            Key::Up => self.index = self.index.saturating_sub(1),
            Key::Down => self.index = (self.index + 1).min(count.saturating_sub(1)),
            Key::SoftRight => self.go(Screen::Messages),
            Key::Delete if count > 0 => {
                if drafts {
                    self.mailbox.drafts.remove(self.index);
                } else {
                    self.mailbox.outbox.remove(self.index);
                }
                self.index = self.index.min(count.saturating_sub(2));
                self.dirty = true;
            }
            Key::Center | Key::SoftLeft if count > 0 => {
                if drafts {
                    let m = &self.mailbox.drafts[self.index];
                    let mut c = Compose::default();
                    c.to.insert(&m.to);
                    c.body.insert(&m.body);
                    c.draft = Some(self.index);
                    c.recipient = false;
                    self.compose = c;
                    self.go(Screen::Compose);
                } else {
                    let m = &self.mailbox.outbox[self.index];
                    self.notice(format!("NOT SENT\nTo: {}\n{}", m.to, m.body));
                }
            }
            _ => {}
        }
    }
    fn overlay_event(&mut self, key: Key, now: u64) {
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
                    Key::Up => index = index.saturating_sub(1),
                    Key::Down => index = (index + 1).min(n - 1),
                    Key::SoftRight | Key::End => return,
                    Key::Center | Key::SoftLeft => {
                        self.menu_action(kind, index);
                        return;
                    }
                    _ => {}
                }
                self.overlay = Some(Overlay::Menu { kind, index });
            }
            Overlay::Text { target, mut editor } => {
                match key {
                    Key::SoftRight | Key::End => return,
                    Key::SoftLeft | Key::Center => {
                        match target {
                            TextTarget::Address => {
                                let text = editor.text();
                                let url = if text.contains(':') {
                                    text
                                } else {
                                    format!("https://{text}")
                                };
                                self.pending_url = Some(url);
                            }
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
            Overlay::CloseBrowser => match key {
                Key::Center | Key::SoftLeft => self.go(Screen::Menu),
                Key::SoftRight | Key::End => {}
                _ => self.overlay = Some(Overlay::CloseBrowser),
            },
        }
    }
    fn menu_action(&mut self, kind: MenuKind, index: usize) {
        match kind {
            MenuKind::Sms => match index {
                0 => {
                    let m = self.message();
                    match self.mailbox.queue(self.compose.draft, m) {
                        Ok(()) => {
                            self.dirty = true;
                            self.go(Screen::Messages);
                            self.notice("Queued locally. No SMS service: message was NOT sent.");
                        }
                        Err(e) => self.notice(e),
                    }
                }
                1 => {
                    self.save_draft();
                    self.notice("Draft saved locally");
                }
                2 => {
                    self.compose.recipient = false;
                    self.compose.body.insert("\n");
                }
                3 => self.compose.body.clear(),
                4 => self.input_mode(),
                _ => self.close_compose(),
            },
            MenuKind::ExitSms => match index {
                0 => {
                    self.save_draft();
                    self.go(Screen::Messages);
                }
                1 => self.go(Screen::Messages),
                _ => {}
            },
            MenuKind::Browser => match index {
                0 => {
                    self.overlay = Some(Overlay::Text {
                        target: TextTarget::Address,
                        editor: Editor::new(&self.browser.page.url, 512),
                    })
                }
                1 => self.pending_url = Some("about:demo".into()),
                2 => self.pending_url = Some(self.browser.page.url.clone()),
                3 => self.notice(self.browser.page.url.clone()),
                _ => self.overlay = Some(Overlay::CloseBrowser),
            },
            MenuKind::Music => match index {
                0 => self.music.cycle_repeat(),
                1 => self.music.shuffle = !self.music.shuffle,
                _ => self.go(Screen::Menu),
            },
        }
    }
    fn commit_select(&mut self, i: usize, flags: Vec<bool>) {
        if let Some(Element::Select { selected, .. }) = self.browser.page.elements.get_mut(i) {
            *selected = flags;
        }
    }
    fn browser_key(&mut self, key: Key) {
        match key {
            Key::Up => self.browser.move_focus(-1),
            Key::Down => self.browser.move_focus(1),
            Key::Left => self.browser.move_focus(-5),
            Key::Right => self.browser.move_focus(5),
            Key::SoftLeft => {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::Browser,
                    index: 0,
                })
            }
            Key::SoftRight => {
                if self.browser.back() == Back::Close {
                    self.overlay = Some(Overlay::CloseBrowser);
                }
            }
            Key::Center => {
                let i = self.browser.focus;
                if let Some(e) = self.browser.page.elements.get(i).cloned() {
                    match e {
                        Element::Link { url, .. } => self.pending_url = Some(url),
                        Element::Field { value, limit, .. } => {
                            self.overlay = Some(Overlay::Text {
                                target: TextTarget::Field(i),
                                editor: Editor::new(&value, limit),
                            })
                        }
                        Element::Check { .. } => {
                            if let Element::Check { checked, .. } =
                                &mut self.browser.page.elements[i]
                            {
                                *checked = !*checked;
                            }
                        }
                        Element::Select {
                            selected, multiple, ..
                        } => {
                            let index = selected.iter().position(|v| *v).unwrap_or(0);
                            self.overlay = Some(Overlay::Select {
                                element: i,
                                index,
                                selected,
                                multiple,
                            });
                        }
                        Element::Reset { form } => self.browser.reset(form),
                        Element::Submit { form, .. } => match web::submit(
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
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "phone_tests.rs"]
mod tests;
