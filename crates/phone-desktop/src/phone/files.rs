//! File-manager interaction. Host I/O is handled by the storage service.
use super::*;
use phone_vfs::Entry;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileView {
    #[default]
    Browse,
    Options,
    Delete,
    Details,
}
pub enum FileRequest {
    List(String),
    Enter(String),
    Parent,
    Mkdir(String),
    Rename(String, String),
    Remove(String),
    Play(String),
}
pub struct Files {
    pub path: String,
    pub entries: Vec<Entry>,
    pub view: FileView,
    pub selected: usize,
    pub writable: bool,
    pub pending: Option<FileRequest>,
    pub history: Vec<(String, usize)>,
}
impl Default for Files {
    fn default() -> Self {
        Self {
            path: "/".into(),
            entries: vec![],
            view: FileView::Browse,
            selected: 0,
            writable: false,
            pending: None,
            history: vec![],
        }
    }
}
impl Phone {
    pub fn file_labels(&self) -> Vec<String> {
        match self.files.view {
            FileView::Options => ["打开", "详情", "重命名", "删除", "新建文件夹", "刷新"]
                .map(str::to_owned)
                .to_vec(),
            _ => self
                .files
                .entries
                .iter()
                .map(|e| {
                    if e.directory {
                        format!("{} /", e.name)
                    } else {
                        e.name.clone()
                    }
                })
                .collect(),
        }
    }
    pub(super) fn files_key(&mut self, key: Key) {
        if self.files.pending.is_some() {
            return;
        }
        if self.files.view == FileView::Delete {
            match key {
                Key::Center | Key::SoftLeft => {
                    if let Some(e) = self.files.entries.get(self.files.selected) {
                        self.files.pending = Some(FileRequest::Remove(e.path.clone()));
                    }
                    self.files.view = FileView::Browse;
                    self.index = self.files.selected;
                }
                Key::SoftRight | Key::End => {
                    self.files.view = FileView::Browse;
                    self.index = self.files.selected;
                }
                _ => {}
            }
            return;
        }
        if self.files.view == FileView::Details {
            if matches!(key, Key::Center | Key::SoftRight | Key::SoftLeft) {
                self.files.view = FileView::Browse;
                self.index = self.files.selected;
            }
            return;
        }
        let n = self.file_labels().len();
        match key {
            Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
            Key::Down if n > 0 => self.index = (self.index + 1) % n,
            Key::SoftRight => {
                if self.files.view == FileView::Options {
                    self.files.view = FileView::Browse;
                    self.index = self.files.selected;
                } else if !self.files.history.is_empty() {
                    self.files.pending = Some(FileRequest::Parent);
                } else {
                    self.leave_app();
                }
            }
            Key::SoftLeft if self.files.view == FileView::Browse => {
                self.files.selected = self.index;
                self.files.view = FileView::Options;
                self.index = 0;
            }
            Key::Center | Key::SoftLeft => {
                if self.files.view == FileView::Options {
                    let action = self.index;
                    self.index = self.files.selected;
                    self.files.view = FileView::Browse;
                    match action {
                        0 => self.file_open(),
                        1 if self.files.entries.get(self.index).is_some() => {
                            self.files.view = FileView::Details
                        }
                        2..=4 if !self.files.writable => self.notice("此存储为只读。"),
                        2 => {
                            if let Some(e) = self.files.entries.get(self.index) {
                                self.open_local_text(
                                    TextTarget::FileName(false),
                                    e.name.clone(),
                                    false,
                                );
                            }
                        }
                        3 if self.files.entries.get(self.index).is_some() => {
                            self.files.view = FileView::Delete
                        }
                        4 => self.open_local_text(TextTarget::FileName(true), String::new(), false),
                        5 => self.files.pending = Some(FileRequest::List(self.files.path.clone())),
                        _ => {}
                    }
                } else {
                    self.file_open();
                }
            }
            _ => {}
        }
    }
    fn file_open(&mut self) {
        let Some(e) = self.files.entries.get(self.index) else {
            return;
        };
        if e.directory {
            self.files.pending = Some(FileRequest::Enter(e.path.clone()));
        } else {
            self.files.pending = Some(FileRequest::Play(e.path.clone()));
        }
    }
    pub(super) fn file_name(&mut self, new: bool, name: String) {
        if !phone_vfs::valid_name(&name) || name.starts_with('.') {
            self.notice("文件名无效。");
            return;
        }
        let target = format!("{}/{}", self.files.path, name);
        self.files.pending = if new {
            Some(FileRequest::Mkdir(target))
        } else {
            self.files
                .entries
                .get(self.files.selected)
                .map(|e| FileRequest::Rename(e.path.clone(), target))
        };
    }
}
