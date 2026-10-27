//! Standard popup composition. Draw the parent first, then this layer once.
use super::{list, softkeys::Softkeys};
use crate::canvas::Canvas;

pub struct Popup<'a> {
    dim_parent: bool,
    footer: Softkeys<'a>,
}
impl Default for Popup<'_> {
    fn default() -> Self {
        Self {
            dim_parent: true,
            footer: Softkeys::back("选择"),
        }
    }
}
impl<'a> Popup<'a> {
    /// For a standalone menu or a parent already shaded by the outer modal layer.
    pub fn unshaded(mut self) -> Self {
        self.dim_parent = false;
        self
    }
    pub fn footer(mut self, footer: Softkeys<'a>) -> Self {
        self.footer = footer;
        self
    }
    pub fn draw<'r>(
        self,
        c: &mut Canvas,
        count: usize,
        index: usize,
        row: impl FnMut(usize) -> list::Row<'r>,
    ) {
        if self.dim_parent {
            c.dim();
        }
        list::popup(c, count, index, row);
        self.footer.draw(c);
    }
}
