//! Derived geometry returned by read-only painting and applied before input.
use crate::phone::Phone;
use phone_core::editor::VisualRow;

#[derive(Default)]
pub struct Layout {
    pub editor: Option<Vec<VisualRow>>,
    pub notice_lines: Option<usize>,
    pub index: Option<usize>,
    pub browser: Option<Browser>,
    pub reader: Option<(usize, usize)>,
    pub conversation: Option<(i32, i32, bool)>,
}
pub struct Browser {
    pub height: usize,
    pub bounds: Vec<(usize, usize)>,
    pub scroll: usize,
}
impl Layout {
    pub fn apply(self, phone: &mut Phone) {
        if let Some(rows) = self.editor {
            phone.editor_lines = rows;
        }
        if let Some(lines) = self.notice_lines {
            phone.notice_lines = lines;
        }
        if let Some(index) = self.index {
            phone.index = index;
        }
        if let Some(browser) = self.browser {
            phone.browsing.model.content_height = browser.height;
            phone.browsing.model.bounds = browser.bounds;
            phone.browsing.model.scroll = browser.scroll;
        }
        if let Some((lines, scroll)) = self.reader {
            phone.messaging.ui.lines = lines;
            phone.messaging.ui.scroll = scroll;
        }
        if let Some((max_scroll, scroll, reveal)) = self.conversation {
            phone.messaging.chat.max_scroll = max_scroll;
            phone.messaging.chat.scroll = scroll;
            phone.messaging.chat.reveal = reveal;
        }
    }
}
