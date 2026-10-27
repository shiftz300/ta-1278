//! State owned by this application.
use super::*;

pub struct State {
    pub model: Browser,
    pub view: BrowserView,
    pub settings_origin: BrowserView,
    pub data: browser::data::BrowserData,
    pub dirty: bool,
    pub clear: [bool; 2],
    pub pending: Option<String>,
    pub requested: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            model: Browser::new(phone_core::browser::Page {
                flow: vec![],
                url: String::new(),
                title: String::new(),
                elements: vec![],
                forms: vec![],
            }),
            view: BrowserView::Start,
            settings_origin: BrowserView::Start,
            data: Default::default(),
            dirty: false,
            clear: [false, true],
            pending: None,
            requested: String::new(),
        }
    }
}
