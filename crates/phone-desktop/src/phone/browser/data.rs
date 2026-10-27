//! Replica-only browser persistence. Offline documents never retain entered form values.
use phone_core::browser::{Element, Page};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BrowserData {
    pub bookmarks: Vec<(String, String)>,
    pub history: Vec<(String, String)>,
    pub offline: Vec<Page>,
    pub settings: BrowserSettings,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BrowserSettings {
    pub custom_home: bool,
    pub home_url: String,
    pub download_images: bool,
    pub font_size: usize,
}
impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            custom_home: false,
            home_url: "http://".into(),
            download_images: true,
            font_size: 0,
        }
    }
}
impl BrowserData {
    pub fn visit(&mut self, page: &Page) {
        if page.url.is_empty() {
            return;
        }
        self.history.retain(|entry| entry.1 != page.url);
        self.history
            .insert(0, (page.title.clone(), page.url.clone()));
        self.history.truncate(100);
    }
    pub fn save_page(&mut self, page: &Page) {
        let mut page = page.clone();
        for e in &mut page.elements {
            match e {
                Element::Field {
                    value,
                    initial,
                    password,
                    ..
                } => {
                    if *password {
                        initial.clear();
                    }
                    *value = initial.clone();
                }
                Element::Check {
                    checked, initial, ..
                } => *checked = *initial,
                Element::Select {
                    selected, initial, ..
                } => *selected = initial.clone(),
                _ => {}
            }
        }
        self.offline.retain(|p| p.url != page.url);
        self.offline.insert(0, page);
        self.offline.truncate(20);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_offline_pages_without_layout_flow_still_load() {
        let old =
            r#"{"url":"about:demo","title":"Saved","elements":[{"Text":"Old text"}],"forms":[]}"#;
        let page: Page = serde_json::from_str(old).unwrap();
        assert!(page.flow.is_empty());
        assert_eq!(page.elements, vec![Element::Text("Old text".into())]);
        let mut data = BrowserData::default();
        data.save_page(&page);
        assert_eq!(data.offline[0].title, "Saved");
    }
    #[test]
    fn offline_copy_resets_user_fields_and_history_survives_navigation() {
        let mut page = crate::web::parse(
            "https://example.com/",
            "<input name=a value='default'><input type=password value='secret'><input type=checkbox>",
        );
        if let Element::Field { value, .. } = &mut page.elements[0] {
            *value = "private edit".into();
        }
        let mut data = BrowserData::default();
        data.visit(&page);
        data.save_page(&page);
        if let Element::Field { value, .. } = &data.offline[0].elements[0] {
            assert_eq!(value, "default");
        } else {
            panic!();
        }
        if let Element::Field { value, initial, .. } = &data.offline[0].elements[1] {
            assert!(value.is_empty() && initial.is_empty());
        } else {
            panic!();
        }
        let dir =
            std::env::temp_dir().join(format!("ta1278-browser-roundtrip-{}", std::process::id()));
        crate::storage::save_browser(&dir, &data).unwrap();
        let restored = crate::storage::load_browser(&dir).unwrap();
        assert_eq!(restored.history, data.history);
        assert_eq!(restored.offline[0].elements, data.offline[0].elements);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
