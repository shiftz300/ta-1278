//! Local music library/player navigation. Audio transport is deliberately separate.
mod state;
use super::*;
use phone_core::music::Repeat;
pub use state::{SavedMusic, State};
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub name: String,
    pub paths: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MusicView {
    #[default]
    Root,
    Songs,
    Playlists,
    Player,
    Repeat,
    Queue,
    Playlist,
    AddPlaylist,
    PlaylistMenu,
    DeletePlaylist,
}
impl Phone {
    pub fn music_items(&self) -> Vec<String> {
        match self.playback.view {
            MusicView::Root => {
                let mut labels = vec!["全部播放".into(), "歌曲".into(), "播放列表".into()];
                if self.playback.started {
                    labels.push(
                        self.playback
                            .model
                            .tracks
                            .get(self.playback.model.index)
                            .map_or("", |t| t.0.as_str())
                            .to_owned(),
                    );
                }
                labels
            }
            MusicView::Songs => self
                .playback
                .model
                .tracks
                .iter()
                .map(|t| t.0.clone())
                .collect(),
            MusicView::Queue => self
                .playback
                .model
                .order()
                .iter()
                .map(|i| self.playback.model.tracks[*i].0.clone())
                .collect(),
            MusicView::Playlists => self
                .playback
                .playlists
                .iter()
                .map(|p| p.name.clone())
                .collect(),
            MusicView::AddPlaylist => std::iter::once("新增播放列表".into())
                .chain(self.playback.playlists.iter().map(|p| p.name.clone()))
                .collect(),
            MusicView::Playlist => self
                .playlist_tracks()
                .iter()
                .map(|i| self.playback.model.tracks[*i].0.clone())
                .collect(),
            MusicView::PlaylistMenu => ["重命名", "删除播放列表", "移除歌曲"]
                .map(str::to_owned)
                .to_vec(),
            MusicView::DeletePlaylist => vec!["取消".into(), "删除播放列表".into()],
            MusicView::Repeat => vec!["关闭".into(), "全部歌曲".into(), "当前歌曲".into()],
            MusicView::Player => vec![],
        }
    }
    pub fn music_wait_time(&self) -> u64 {
        self.last_tick.unwrap_or(0)
    }
    pub fn music_refresh_visible(&self) -> bool {
        self.playback.view == MusicView::Root
            && !self.playback.wait_dismissed
            && (self.playback.refresh || self.playback.loading)
    }
    pub(super) fn music_key(&mut self, key: Key) {
        if self.music_refresh_visible() {
            // Close the wait page to the library without discarding the pending scan. Home also
            // remains available through the global End route.
            if key == Key::SoftRight {
                self.playback.wait_dismissed = true;
            }
            return;
        }
        if self.playback.view == MusicView::Player {
            match key {
                Key::SoftLeft => {
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::Music,
                        index: 0,
                    })
                }
                Key::SoftRight if self.playback.return_file => {
                    let index = self.files.selected;
                    self.go(Screen::Tools(tools::ToolPage::Files));
                    self.index = index;
                    self.playback.return_file = false;
                }
                Key::SoftRight => {
                    self.playback.view = MusicView::Root;
                    self.index = 0;
                }
                _ => {}
            }
            return;
        }
        let n = match self.playback.view {
            MusicView::Songs => self.playback.model.tracks.len(),
            MusicView::Queue => self.playback.model.order().len(),
            MusicView::Playlist => self.playlist_tracks().len(),
            _ => self.music_items().len(),
        };
        match key {
            Key::Up | Key::Down => {
                vertical(&mut self.index, n, key, Boundary::Wrap);
            }
            Key::SoftRight => match self.playback.view {
                MusicView::Root => self.leave_app(),
                MusicView::Repeat => {
                    self.playback.view = MusicView::Player;
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::Music,
                        index: 0,
                    });
                }
                MusicView::Queue => self.playback.view = MusicView::Player,
                MusicView::Playlist => {
                    self.playback.view = MusicView::Playlists;
                    self.index = self.playback.playlist;
                }
                MusicView::AddPlaylist => {
                    self.playback.view = MusicView::Player;
                    self.index = 0;
                }
                MusicView::PlaylistMenu | MusicView::DeletePlaylist => {
                    self.playback.view = MusicView::Playlist;
                    self.index = self.playback.row;
                }
                _ => {
                    self.playback.view = MusicView::Root;
                    self.index = 0;
                }
            },
            Key::SoftLeft if self.playback.view == MusicView::Root => {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::MusicLibrary,
                    index: 0,
                })
            }
            Key::SoftLeft if self.playback.view == MusicView::Playlists => {
                self.open_local_text(TextTarget::PlaylistName, String::new(), false)
            }
            Key::SoftLeft if self.playback.view == MusicView::Playlist => {
                self.playback.row = self.index;
                self.playback.view = MusicView::PlaylistMenu;
                self.index = 0;
            }
            Key::Center if n > 0 => match self.playback.view {
                MusicView::Root => match self.index {
                    0 if self.playback.loading || self.playback.refresh => {
                        self.notice("正在刷新曲库，请稍候。")
                    }
                    0 if !self.playback.model.tracks.is_empty() => {
                        self.playback.model.queue.clear();
                        self.play_track(0);
                    }
                    0 => self.notice("没有可播放的音乐。"),
                    1 => {
                        self.playback.view = MusicView::Songs;
                        self.index = 0;
                    }
                    2 => {
                        self.playback.view = MusicView::Playlists;
                        self.index = 0;
                    }
                    3 => self.playback.view = MusicView::Player,
                    _ => {}
                },
                MusicView::Songs => {
                    self.playback.model.queue.clear();
                    self.play_track(self.index);
                }
                MusicView::Queue => {
                    let order = self.playback.model.order();
                    if let Some(i) = order.get(self.index) {
                        self.play_track(*i);
                    }
                }
                MusicView::Playlists => {
                    self.playback.playlist = self.index;
                    self.playback.view = MusicView::Playlist;
                    self.index = 0;
                }
                MusicView::Playlist => {
                    let tracks = self.playlist_tracks();
                    if let Some(i) = tracks.get(self.index) {
                        let i = *i;
                        self.playback.model.queue = tracks;
                        self.play_track(i);
                    }
                }
                MusicView::AddPlaylist if self.index == 0 => {
                    self.open_local_text(TextTarget::PlaylistName, String::new(), false)
                }
                MusicView::AddPlaylist => {
                    let i = self.index - 1;
                    self.playlist_add(i);
                    self.playback.view = MusicView::Player;
                }
                MusicView::PlaylistMenu => match self.index {
                    0 => self.open_local_text(
                        TextTarget::PlaylistName,
                        self.playback.playlists[self.playback.playlist].name.clone(),
                        false,
                    ),
                    1 => {
                        self.playback.view = MusicView::DeletePlaylist;
                        self.index = 0;
                    }
                    2 => {
                        let available = self.playlist_tracks();
                        if let Some(index) = available.get(self.playback.row) {
                            let path = &self.playback.paths[*index];
                            self.playback.playlists[self.playback.playlist]
                                .paths
                                .retain(|p| p != path);
                            self.records_dirty = true;
                        }
                        self.playback.view = MusicView::Playlist;
                        self.index = 0;
                    }
                    _ => {}
                },
                MusicView::DeletePlaylist => {
                    if self.index == 1 && self.playback.playlist < self.playback.playlists.len() {
                        self.playback.playlists.remove(self.playback.playlist);
                        self.records_dirty = true;
                    }
                    self.playback.view = MusicView::Playlists;
                    self.index = 0;
                }
                MusicView::Repeat => {
                    self.playback.model.repeat =
                        [Repeat::Off, Repeat::All, Repeat::One][self.index];
                    self.playback.view = MusicView::Player;
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::Music,
                        index: 0,
                    });
                }
                _ => {}
            },
            _ => {}
        }
    }
    pub fn play_track(&mut self, index: usize) {
        if !self.features.music {
            self.notice("此功能已停用。");
            return;
        }
        if self.playback.loading || self.playback.refresh {
            self.notice("正在刷新曲库，请稍候。");
            return;
        }
        if index >= self.playback.model.tracks.len() {
            return;
        }
        self.playback.model.play(index);
        self.playback.started = true;
        self.playback.view = MusicView::Player;
    }
    pub(super) fn music_action(&mut self, index: usize) {
        match index {
            0 => {
                self.playback.view = MusicView::Repeat;
                self.index = match self.playback.model.repeat {
                    Repeat::Off => 0,
                    Repeat::All => 1,
                    Repeat::One => 2,
                };
            }
            1 => self.playback.model.shuffle = !self.playback.model.shuffle,
            2 => {
                self.playback.view = MusicView::Queue;
                self.index = self
                    .playback
                    .model
                    .order()
                    .iter()
                    .position(|i| *i == self.playback.model.index)
                    .unwrap_or(0);
            }
            3 => {
                self.playback.view = MusicView::AddPlaylist;
                self.index = 0;
            }
            4 => {
                if let Some(path) = self.playback.paths.get(self.playback.model.index) {
                    self.settings.preferences.ringtone = path.clone();
                    self.settings.dirty = true;
                    self.notice("已设为铃声。");
                }
            }
            _ => self.notice("分享服务暂未接入。"),
        }
    }
    pub fn playlist_tracks(&self) -> Vec<usize> {
        self.playback
            .playlists
            .get(self.playback.playlist)
            .map(|list| {
                list.paths
                    .iter()
                    .filter_map(|p| self.music_index(p))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn music_index(&self, path: &str) -> Option<usize> {
        self.playback.lookup.get(path).copied()
    }
    pub fn set_music_paths(&mut self, paths: Vec<String>) {
        self.playback.lookup = paths
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, p)| (p, i))
            .collect();
        self.playback.paths = paths;
    }
    pub fn append_music_path(&mut self, path: String) {
        self.playback
            .lookup
            .insert(path.clone(), self.playback.paths.len());
        self.playback.paths.push(path);
    }
    fn playlist_add(&mut self, index: usize) {
        if let (Some(list), Some(path)) = (
            self.playback.playlists.get_mut(index),
            self.playback.paths.get(self.playback.model.index),
        ) && list.paths.len() < 1000
            && !list.paths.contains(path)
        {
            list.paths.push(path.clone());
            self.records_dirty = true;
        }
    }
    pub(super) fn playlist_name(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty()
            || name.chars().count() > 80
            || self.playback.playlists.iter().enumerate().any(|(i, l)| {
                l.name == name
                    && !(self.playback.view == MusicView::PlaylistMenu
                        && i == self.playback.playlist)
            })
        {
            self.notice("列表名称为空或已存在。");
            return;
        }
        if self.playback.view == MusicView::PlaylistMenu {
            self.playback.playlists[self.playback.playlist].name = name.into();
            self.records_dirty = true;
            self.playback.view = MusicView::Playlist;
            self.index = self.playback.row;
            return;
        }
        if self.playback.playlists.len() >= 100 {
            self.notice("播放列表已满。");
            return;
        }
        let index = self.playback.playlists.len();
        self.playback.playlists.push(Playlist {
            name: name.into(),
            paths: vec![],
        });
        self.records_dirty = true;
        if self.playback.view == MusicView::AddPlaylist {
            self.playlist_add(index);
            self.playback.view = MusicView::Player;
        } else {
            self.index = index;
        }
    }
}
