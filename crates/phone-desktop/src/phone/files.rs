//! File-manager interaction. Host I/O is handled by the storage service.
use super::*;
use crate::local_files::Entry;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileView {
    #[default]
    Storage,
    StorageOptions,
    StorageDetails,
    Browse,
    Options,
    DeleteSelect,
    DeleteOptions,
    DeleteConfirm,
    Details,
}
pub enum FileRequest {
    Storage,
    List(String),
    Enter(String),
    Parent,
    Mkdir(String),
    Rename(String, String),
    RemoveMany(Vec<String>),
    Play(String),
    Transfer(String, String, bool),
}
impl FileRequest {
    pub fn mutating(&self) -> bool {
        matches!(
            self,
            Self::Mkdir(_) | Self::Rename(..) | Self::RemoveMany(_) | Self::Transfer(..)
        )
    }
}
pub struct Transfer {
    pub source: String,
    pub moving: bool,
    path: String,
    index: usize,
    history: Vec<(String, usize)>,
}
pub struct Files {
    pub capacity: Option<(u64, u64)>,
    pub return_to: Option<(Screen, usize)>,
    pub transfer: Option<Transfer>,
    pub busy: bool,
    pub path: String,
    pub entries: Vec<Entry>,
    pub view: FileView,
    pub selected: usize,
    pub marked: std::collections::BTreeSet<String>,
    pub delete_index: usize,
    pub delete_kind: Option<bool>,
    pub writable: bool,
    pub pending: Option<FileRequest>,
    pub history: Vec<(String, usize)>,
}
impl Default for Files {
    fn default() -> Self {
        Self {
            capacity: None,
            return_to: None,
            transfer: None,
            busy: false,
            path: "/".into(),
            entries: vec![],
            view: FileView::Storage,
            selected: 0,
            marked: Default::default(),
            delete_index: 0,
            delete_kind: None,
            writable: false,
            pending: None,
            history: vec![],
        }
    }
}
impl Phone {
    pub fn file_action_enabled(&self, index: usize) -> bool {
        let selected = self.files.entries.get(self.files.selected).is_some();
        match index {
            0 => !self.files.entries.is_empty() && self.files.writable,
            1..=4 => selected && self.files.writable,
            5 => selected,
            6 => self.files.writable,
            _ => false,
        }
    }
    pub fn file_labels(&self) -> Vec<String> {
        if self.files.view == FileView::Options {
            let folder = self
                .files
                .entries
                .get(self.files.selected)
                .is_none_or(|e| e.directory);
            if folder {
                [
                    "删除",
                    "删文件夹",
                    "移动文件夹",
                    "复制文件夹",
                    "重命名文件夹",
                    "详情",
                    "增加文件夹",
                ]
            } else {
                [
                    "删除",
                    "删除文件",
                    "移动文件",
                    "复制文件",
                    "重命名文件",
                    "详情",
                    "增加文件夹",
                ]
            }
            .map(str::to_owned)
            .to_vec()
        } else {
            self.files.entries.iter().map(|e| e.name.clone()).collect()
        }
    }
    pub(super) fn files_key(&mut self, key: Key) {
        if self.files.pending.is_some() || self.files.busy {
            return;
        }
        if self.file_delete_key(key) {
            return;
        }
        match self.files.view {
            FileView::Storage => {
                match key {
                    Key::Center => self.files.pending = Some(FileRequest::List("/".into())),
                    Key::SoftLeft if self.files.transfer.is_some() => self.file_transfer_here("/"),
                    Key::SoftLeft => {
                        self.files.view = FileView::StorageOptions;
                        self.index = 0;
                    }
                    Key::SoftRight if self.files.transfer.is_some() => {
                        self.finish_file_transfer();
                        self.files.pending = Some(FileRequest::List(self.files.path.clone()));
                    }
                    Key::SoftRight => self.leave_app(),
                    _ => {}
                }
                return;
            }
            FileView::StorageOptions => {
                match key {
                    Key::Up | Key::Down => self.index ^= 1,
                    Key::Center | Key::SoftLeft if self.index == 0 => {
                        self.files.view = FileView::StorageDetails
                    }
                    Key::SoftRight => {
                        self.files.view = FileView::Storage;
                        self.index = 0;
                    }
                    _ => {} // Format intentionally unavailable for a mapped host folder.
                }
                return;
            }
            FileView::StorageDetails => {
                if matches!(key, Key::SoftRight | Key::Center) {
                    self.files.view = FileView::Storage;
                    self.index = 0;
                }
                return;
            }
            _ => {}
        }
        if self.files.transfer.is_some() {
            let n = self.files.entries.len();
            match key {
                Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
                Key::Down if n > 0 => self.index = (self.index + 1) % n,
                Key::Center => self.file_open(),
                Key::SoftLeft => self.file_transfer_here(&self.files.path.clone()),
                Key::SoftRight if !self.files.history.is_empty() => {
                    self.files.pending = Some(FileRequest::Parent)
                }
                Key::SoftRight => {
                    self.files.view = FileView::Storage;
                    self.index = 0;
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
                } else if self.files.path == "/" {
                    self.files.view = FileView::Storage;
                    self.index = 0;
                } else {
                    if let Some((screen, index)) = self.files.return_to.take() {
                        self.go(screen);
                        self.index = index;
                    } else {
                        self.leave_app();
                    }
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
                    if !self.file_action_enabled(action) {
                        return;
                    }
                    self.index = self.files.selected;
                    self.files.view = FileView::Browse;
                    match action {
                        0 | 1 => {
                            self.files.delete_kind = if action == 1 {
                                self.files.entries.get(self.index).map(|e| e.directory)
                            } else {
                                None
                            };
                            self.files.marked.clear();
                            self.files.delete_index = 0;
                            self.files.view = FileView::DeleteSelect;
                            self.index = 0;
                        }
                        2 | 3 => {
                            if let Some(e) = self.files.entries.get(self.index) {
                                self.files.transfer = Some(Transfer {
                                    source: e.path.clone(),
                                    moving: action == 2,
                                    path: self.files.path.clone(),
                                    index: self.index,
                                    history: std::mem::take(&mut self.files.history),
                                });
                                self.files.view = FileView::Storage;
                                self.index = 0;
                            }
                        }
                        4 => {
                            if let Some(e) = self.files.entries.get(self.index) {
                                self.open_local_text(
                                    TextTarget::FileName(false),
                                    e.name.clone(),
                                    false,
                                );
                            }
                        }
                        5 => self.files.view = FileView::Details,
                        6 => {
                            let name = (1..=10001)
                                .map(|n| format!("文件夹 {n}"))
                                .find(|name| !self.files.entries.iter().any(|e| &e.name == name))
                                .unwrap();
                            self.open_local_text(TextTarget::FileName(true), name, false);
                        }
                        _ => {}
                    }
                } else {
                    self.file_open();
                }
            }
            _ => {}
        }
    }
    fn file_transfer_here(&mut self, parent: &str) {
        if let Some(t) = &self.files.transfer {
            let name = t.source.rsplit('/').next().unwrap_or("");
            self.files.pending = Some(FileRequest::Transfer(
                t.source.clone(),
                crate::local_files::child(parent, name),
                t.moving,
            ));
        }
    }
    pub fn finish_file_transfer(&mut self) {
        if let Some(t) = self.files.transfer.take() {
            self.files.path = t.path;
            self.files.history = t.history;
            self.files.selected = t.index;
            if self.screen == Screen::Tools(tools::ToolPage::Files) {
                self.index = t.index;
            }
            self.files.view = FileView::Browse;
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
        if !crate::local_files::valid_name(&name) || name.starts_with('.') {
            self.notice("文件名无效。");
            return;
        }
        let target = crate::local_files::child(&self.files.path, &name);
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
