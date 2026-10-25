//! Document and navigation state; the desktop adapter supplies parsed HTTP results.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Element {
    Text(String),
    Link {
        label: String,
        url: String,
    },
    Field {
        label: String,
        name: String,
        value: String,
        initial: String,
        limit: usize,
        password: bool,
        read_only: bool,
        form: usize,
    },
    Check {
        radio: bool,
        label: String,
        name: String,
        checked: bool,
        initial: bool,
        value: String,
        form: usize,
    },
    Select {
        label: String,
        name: String,
        options: Vec<(String, String)>,
        selected: Vec<bool>,
        initial: Vec<bool>,
        multiple: bool,
        form: usize,
    },
    Submit {
        label: String,
        name: String,
        value: String,
        form: usize,
    },
    Reset {
        form: usize,
    },
}
impl Element {
    pub fn label(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Link { label, .. } => format!("› {label}"),
            Self::Field {
                label,
                value,
                password,
                ..
            } => format!(
                "{label}: [{}]",
                if *password {
                    "*".repeat(value.chars().count())
                } else {
                    value.clone()
                }
            ),
            Self::Check { label, checked, .. } => {
                format!("[{}] {label}", if *checked { "x" } else { " " })
            }
            Self::Select {
                label,
                options,
                selected,
                ..
            } => format!(
                "{label}: {}",
                options
                    .iter()
                    .zip(selected)
                    .filter(|(_, s)| **s)
                    .map(|(o, _)| o.0.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Submit { label, .. } => format!("[ {label} ]"),
            Self::Reset { .. } => "[ Reset ]".into(),
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub hidden: Vec<(String, String)>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Page {
    #[serde(default)]
    pub flow: Vec<crate::web_flow::Flow>,
    pub url: String,
    pub title: String,
    pub elements: Vec<Element>,
    pub forms: Vec<Form>,
}
#[derive(Clone, Debug)]
struct Entry {
    page: Page,
    focus: usize,
    scroll: usize,
}
#[derive(Debug, PartialEq)]
pub enum Back {
    Stopped,
    History,
    Close,
}
pub struct Browser {
    pub page: Page,
    pub focus: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub request: u64,
    history: Vec<Entry>,
    future: Vec<Entry>,
    pub scroll: usize,
    pub content_height: usize,
    pub bounds: Vec<(usize, usize)>,
}
impl Browser {
    pub fn new(page: Page) -> Self {
        Self {
            page,
            focus: 0,
            loading: false,
            error: None,
            request: 0,
            history: vec![],
            future: vec![],
            scroll: 0,
            content_height: 0,
            bounds: Vec::new(),
        }
    }
    pub fn start(&mut self) -> u64 {
        self.request += 1;
        self.loading = true;
        self.error = None;
        self.request
    }
    pub fn complete(&mut self, id: u64, result: Result<Page, String>) -> bool {
        if id != self.request || !self.loading {
            return false;
        }
        self.loading = false;
        match result {
            Ok(page) => {
                if self.page.url.is_empty() || self.page.url == page.url {
                    self.page = page;
                } else {
                    self.history.push(Entry {
                        page: std::mem::replace(&mut self.page, page),
                        focus: self.focus,
                        scroll: self.scroll,
                    });
                }
                if self.history.len() > 32 {
                    self.history.remove(0);
                }
                self.future.clear();
                self.scroll = 0;
                self.bounds.clear();
                self.focus = self
                    .page
                    .elements
                    .iter()
                    .position(|e| !matches!(e, Element::Text(_)))
                    .unwrap_or(0);
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
        true
    }
    pub fn back(&mut self) -> Back {
        if self.loading {
            self.request += 1;
            self.loading = false;
            Back::Stopped
        } else if let Some(e) = self.history.pop() {
            self.future.push(Entry {
                page: std::mem::replace(&mut self.page, e.page),
                focus: self.focus,
                scroll: self.scroll,
            });
            self.focus = e.focus;
            self.scroll = e.scroll;
            self.bounds.clear();
            self.error = None;
            Back::History
        } else {
            Back::Close
        }
    }
    pub fn stop(&mut self) {
        if self.loading {
            self.request += 1;
            self.loading = false;
        }
    }
    pub fn clear_navigation(&mut self) {
        self.stop();
        self.history.clear();
        self.future.clear();
        self.scroll = 0;
        self.page.elements.clear();
        self.page.flow.clear();
        self.page.url.clear();
        self.page.title.clear();
        self.bounds.clear();
        self.error = None;
    }
    pub fn visited(&self) -> Vec<(String, String)> {
        self.history
            .iter()
            .map(|e| &e.page)
            .chain(std::iter::once(&self.page))
            .filter(|p| !p.url.is_empty())
            .map(|p| (p.title.clone(), p.url.clone()))
            .collect()
    }
    pub fn can_forward(&self) -> bool {
        !self.future.is_empty()
    }
    pub fn can_back(&self) -> bool {
        !self.history.is_empty()
    }
    pub fn forward(&mut self) -> bool {
        self.stop();
        if let Some(e) = self.future.pop() {
            self.history.push(Entry {
                page: std::mem::replace(&mut self.page, e.page),
                focus: self.focus,
                scroll: self.scroll,
            });
            self.focus = e.focus;
            self.scroll = e.scroll;
            self.bounds.clear();
            self.error = None;
            true
        } else {
            false
        }
    }
    pub fn move_focus(&mut self, delta: i32) {
        let next = if delta > 0 {
            (self.focus + 1..self.page.elements.len())
                .find(|&i| !matches!(self.page.elements[i], Element::Text(_)))
        } else {
            (0..self.focus)
                .rev()
                .find(|&i| !matches!(self.page.elements[i], Element::Text(_)))
        };
        let max_scroll = self.content_height.saturating_sub(216);
        if let Some(i) = next {
            if let Some(&(top, bottom)) = self.bounds.get(i) {
                if delta > 0 && top >= self.scroll + 216 {
                    self.scroll = (self.scroll + 34).min(max_scroll);
                    return;
                }
                if delta < 0 && bottom < self.scroll {
                    self.scroll = self.scroll.saturating_sub(34);
                    return;
                }
                if bottom > self.scroll + 216 {
                    self.scroll = bottom.saturating_sub(216).min(max_scroll);
                }
                if top < self.scroll {
                    self.scroll = top;
                }
            }
            self.focus = i;
        } else if delta > 0 {
            self.scroll = (self.scroll + 34).min(max_scroll);
        } else {
            self.scroll = self.scroll.saturating_sub(34);
        }
    }
    pub fn toggle_check(&mut self, index: usize) {
        let Some(Element::Check {
            radio, name, form, ..
        }) = self.page.elements.get(index).cloned()
        else {
            return;
        };
        if radio {
            for e in &mut self.page.elements {
                if let Element::Check {
                    radio: true,
                    name: n,
                    form: f,
                    checked,
                    ..
                } = e
                    && *n == name
                    && *f == form
                {
                    *checked = false;
                }
            }
        }
        if let Some(Element::Check { checked, .. }) = self.page.elements.get_mut(index) {
            *checked = if radio { true } else { !*checked };
        }
    }
    pub fn reset(&mut self, form: usize) {
        for e in &mut self.page.elements {
            match e {
                Element::Field {
                    value,
                    initial,
                    form: f,
                    ..
                } if *f == form => *value = initial.clone(),
                Element::Check {
                    checked,
                    initial,
                    form: f,
                    ..
                } if *f == form => *checked = *initial,
                Element::Select {
                    selected,
                    initial,
                    form: f,
                    ..
                } if *f == form => *selected = initial.clone(),
                _ => {}
            }
        }
    }
    pub fn form_values(&self, form: usize, submit: Option<usize>) -> Vec<(String, String)> {
        let mut out = self
            .page
            .forms
            .get(form)
            .map(|f| f.hidden.clone())
            .unwrap_or_default();
        for (i, e) in self.page.elements.iter().enumerate() {
            match e {
                Element::Field {
                    name,
                    value,
                    form: f,
                    ..
                } if *f == form && !name.is_empty() => out.push((name.clone(), value.clone())),
                Element::Check {
                    name,
                    checked: true,
                    value,
                    form: f,
                    ..
                } if *f == form && !name.is_empty() => out.push((name.clone(), value.clone())),
                Element::Select {
                    name,
                    options,
                    selected,
                    form: f,
                    ..
                } if *f == form && !name.is_empty() => {
                    for (o, s) in options.iter().zip(selected) {
                        if *s {
                            out.push((name.clone(), o.1.clone()));
                        }
                    }
                }
                Element::Submit {
                    name,
                    value,
                    form: f,
                    ..
                } if *f == form && !name.is_empty() && submit == Some(i) => {
                    out.push((name.clone(), value.clone()))
                }
                _ => {}
            }
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn page(url: &str) -> Page {
        Page {
            flow: vec![],
            url: url.into(),
            title: url.into(),
            elements: vec![Element::Text(url.into())],
            forms: vec![],
        }
    }
    #[test]
    fn stale_and_stop_preserve_history() {
        let mut b = Browser::new(page("a"));
        let x = b.start();
        let y = b.start();
        assert!(!b.complete(x, Ok(page("old"))));
        assert!(b.complete(y, Ok(page("b"))));
        b.start();
        assert_eq!(b.back(), Back::Stopped);
        assert_eq!(b.page.url, "b");
        assert_eq!(b.back(), Back::History);
        assert_eq!(b.page.url, "a");
        assert_eq!(b.back(), Back::Close);
    }
    #[test]
    fn failure_retains_page() {
        let mut b = Browser::new(page("a"));
        let id = b.start();
        b.complete(id, Err("offline".into()));
        assert_eq!(b.page.url, "a");
        assert!(b.error.is_some());
    }
    #[test]
    fn forward_restores_focus_scroll_and_new_branch_discards_future() {
        let mut b = Browser::new(page("a"));
        b.focus = 3;
        b.scroll = 48;
        let id = b.start();
        b.complete(id, Ok(page("b")));
        b.focus = 2;
        b.scroll = 90;
        assert_eq!(b.back(), Back::History);
        assert_eq!((b.focus, b.scroll), (3, 48));
        assert!(b.forward());
        assert_eq!(b.page.url, "b");
        assert_eq!((b.focus, b.scroll), (2, 90));
        b.back();
        let id = b.start();
        b.complete(id, Ok(page("c")));
        assert!(!b.can_forward());
        b.back();
        assert_eq!(b.page.url, "a");
    }
}
