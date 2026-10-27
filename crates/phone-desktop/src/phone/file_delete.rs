//! Native-shaped checkbox selection; confirmation and bounded deletion are host policies.
use super::*;
use files::{FileRequest, FileView};
impl Phone {
    pub fn delete_entries(&self) -> Vec<&crate::local_files::Entry> {
        self.files
            .entries
            .iter()
            .filter(|e| {
                self.files
                    .delete_kind
                    .is_none_or(|directory| e.directory == directory)
            })
            .collect()
    }
    pub(super) fn file_delete_key(&mut self, key: Key) -> bool {
        match self.files.view {
            FileView::DeleteSelect => {
                let entries = self.delete_entries();
                let n = entries.len();
                let path = entries.get(self.index).map(|e| e.path.clone());
                match key {
                    Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
                    Key::Down if n > 0 => self.index = (self.index + 1) % n,
                    Key::Center => {
                        if let Some(path) = path
                            && !self.files.marked.remove(&path)
                        {
                            self.files.marked.insert(path);
                        }
                    }
                    Key::SoftLeft => {
                        self.files.delete_index = self.index;
                        self.files.view = FileView::DeleteOptions;
                        self.index = 0;
                    }
                    Key::SoftRight => {
                        self.files.marked.clear();
                        self.files.view = FileView::Browse;
                        self.index = self
                            .files
                            .selected
                            .min(self.files.entries.len().saturating_sub(1));
                    }
                    _ => {}
                }
            }
            FileView::DeleteOptions => match key {
                Key::Up | Key::Down => self.index ^= 1,
                Key::Center if self.index == 0 && !self.files.marked.is_empty() => {
                    self.files.view = FileView::DeleteConfirm;
                }
                Key::Center if self.index == 1 => {
                    let paths: Vec<_> = self
                        .delete_entries()
                        .iter()
                        .map(|e| e.path.clone())
                        .collect();
                    if paths.iter().all(|p| self.files.marked.contains(p)) {
                        self.files.marked.clear();
                    } else {
                        self.files.marked = paths.into_iter().collect();
                    }
                    self.files.view = FileView::DeleteSelect;
                    self.index = self.files.delete_index;
                }
                Key::SoftRight => {
                    self.files.view = FileView::DeleteSelect;
                    self.index = self.files.delete_index;
                }
                _ => {}
            },
            FileView::DeleteConfirm => match key {
                Key::Center if self.files.writable && !self.files.marked.is_empty() => {
                    self.files.pending = Some(FileRequest::RemoveMany(
                        self.files.marked.iter().cloned().collect(),
                    ));
                }
                Key::SoftRight => {
                    self.files.view = FileView::DeleteSelect;
                    self.index = self.files.delete_index;
                }
                _ => {}
            },
            _ => return false,
        }
        true
    }
}
