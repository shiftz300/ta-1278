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
    Transfer(String, String, bool),
}
pub struct Files {
    pub return_to: Option<(Screen, usize)>,
    pub clipboard: Option<(String, bool)>,
    pub busy: bool,
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
            return_to: None,
            clipboard: None,
            busy: false,
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
    pub fn file_action_enabled(&self, index: usize) -> bool {
        let selected = self.files.entries.get(self.files.selected).is_some();
        let writable = self.files.writable && self.files.path != "/";
        match index {
            0 | 1 => selected,
            2 | 3 | 7 => selected && writable,
            4 => writable,
            5 => true,
            6 => selected && self.files.path != "/",
            8 => self.files.clipboard.is_some() && writable,
            _ => false,
        }
    }
    pub fn file_labels(&self) -> Vec<String> {
        match self.files.view {
            FileView::Options => [
                "打开",
                "详情",
                "重命名",
                "删除",
                "新建文件夹",
                "刷新",
                "复制",
                "移动",
                "粘贴",
            ]
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
        if self.files.pending.is_some() || self.files.busy {
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
                        6 | 7 => {
                            if let Some(e) = self.files.entries.get(self.index) {
                                self.files.clipboard = Some((e.path.clone(), action == 7));
                            }
                        }
                        8 => {
                            if let Some((from, moving)) = self.files.clipboard.clone() {
                                let name = from.rsplit('/').next().unwrap_or("");
                                self.files.pending = Some(FileRequest::Transfer(
                                    from.clone(),
                                    format!("{}/{name}", self.files.path),
                                    moving,
                                ));
                            }
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
