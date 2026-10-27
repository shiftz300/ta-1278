//! Derived geometry returned by read-only painting and applied before input.
use crate::shell::Shell;
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
    pub fn apply(self, shell: &mut Shell) {
        if let Some(rows) = self.editor {
            shell.editor_lines = rows;
        }
        if let Some(lines) = self.notice_lines {
            shell.notice_lines = lines;
        }
        if let Some(index) = self.index {
            shell.index = index;
        }
        if let Some(browser) = self.browser {
            shell.browsing.model.content_height = browser.height;
            shell.browsing.model.bounds = browser.bounds;
            shell.browsing.model.scroll = browser.scroll;
        }
        if let Some((lines, scroll)) = self.reader {
            shell.messaging.ui.lines = lines;
            shell.messaging.ui.scroll = scroll;
        }
        if let Some((max_scroll, scroll, reveal)) = self.conversation {
            shell.messaging.chat.max_scroll = max_scroll;
            shell.messaging.chat.scroll = scroll;
            shell.messaging.chat.reveal = reveal;
        }
    }
}
