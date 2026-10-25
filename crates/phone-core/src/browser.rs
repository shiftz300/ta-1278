//! Document and navigation state; the desktop adapter supplies parsed HTTP results.
#[derive(Clone, Debug, PartialEq)]
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
        form: usize,
    },
    Check {
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
#[derive(Clone, Debug)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub hidden: Vec<(String, String)>,
}
#[derive(Clone, Debug)]
pub struct Page {
    pub url: String,
    pub title: String,
    pub elements: Vec<Element>,
    pub forms: Vec<Form>,
}
#[derive(Clone, Debug)]
struct Entry {
    page: Page,
    focus: usize,
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
                if self.page.url == page.url {
                    self.page = page;
                } else {
                    self.history.push(Entry {
                        page: std::mem::replace(&mut self.page, page),
                        focus: self.focus,
                    });
                }
                if self.history.len() > 32 {
                    self.history.remove(0);
                }
                self.focus = 0;
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
            self.page = e.page;
            self.focus = e.focus;
            self.error = None;
            Back::History
        } else {
            Back::Close
        }
    }
    pub fn move_focus(&mut self, delta: i32) {
        self.focus = (self.focus as i32 + delta)
            .clamp(0, self.page.elements.len().saturating_sub(1) as i32)
            as usize;
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
}
