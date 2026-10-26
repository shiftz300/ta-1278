//! Local music library/player navigation. Audio transport is deliberately separate.
use super::*;
use phone_core::music::Repeat;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub name: String,
    pub paths: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicView {
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
        match self.music_view {
            MusicView::Root => {
                let mut labels = vec!["全部播放".into(), "歌曲".into(), "播放列表".into()];
                if self.music_started {
                    labels.push(
                        self.music
                            .tracks
                            .get(self.music.index)
                            .map_or("", |t| t.0.as_str())
                            .to_owned(),
                    );
                }
                labels
            }
            MusicView::Songs => self.music.tracks.iter().map(|t| t.0.clone()).collect(),
            MusicView::Queue => self
                .music
                .order()
                .iter()
                .map(|i| self.music.tracks[*i].0.clone())
                .collect(),
            MusicView::Playlists => self
                .local_data
                .playlists
                .iter()
                .map(|p| p.name.clone())
                .collect(),
            MusicView::AddPlaylist => std::iter::once("新增播放列表".into())
                .chain(self.local_data.playlists.iter().map(|p| p.name.clone()))
                .collect(),
            MusicView::Playlist => self
                .playlist_tracks()
                .iter()
                .map(|i| self.music.tracks[*i].0.clone())
                .collect(),
            MusicView::PlaylistMenu => ["重命名", "删除播放列表", "移除歌曲"]
                .map(str::to_owned)
                .to_vec(),
            MusicView::DeletePlaylist => vec!["取消".into(), "删除播放列表".into()],
            MusicView::Repeat => vec!["关闭".into(), "全部歌曲".into(), "当前歌曲".into()],
            MusicView::Player => vec![],
        }
    }
    pub(super) fn music_key(&mut self, key: Key) {
        if self.music_view == MusicView::Player {
            match key {
                Key::SoftLeft => {
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::Music,
                        index: 0,
                    })
                }
                Key::SoftRight if self.music_return_file => {
                    let index = self.files.selected;
                    self.go(Screen::Tools(tools::ToolPage::Files));
                    self.index = index;
                    self.music_return_file = false;
                }
                Key::SoftRight => {
                    self.music_view = MusicView::Root;
                    self.index = 0;
                }
                _ => {}
            }
            return;
        }
        let n = match self.music_view {
            MusicView::Songs => self.music.tracks.len(),
            MusicView::Queue => self.music.order().len(),
            MusicView::Playlist => self.playlist_tracks().len(),
            _ => self.music_items().len(),
        };
        match key {
            Key::Up if n > 0 => self.index = (self.index + n - 1) % n,
            Key::Down if n > 0 => self.index = (self.index + 1) % n,
            Key::SoftRight => match self.music_view {
                MusicView::Root => self.leave_app(),
                MusicView::Repeat => {
                    self.music_view = MusicView::Player;
                    self.overlay = Some(Overlay::Menu {
                        kind: MenuKind::Music,
                        index: 0,
                    });
                }
                MusicView::Queue => self.music_view = MusicView::Player,
                MusicView::Playlist => {
                    self.music_view = MusicView::Playlists;
                    self.index = self.playlist;
                }
                MusicView::AddPlaylist => {
                    self.music_view = MusicView::Player;
                    self.index = 0;
                }
                MusicView::PlaylistMenu | MusicView::DeletePlaylist => {
                    self.music_view = MusicView::Playlist;
                    self.index = self.playlist_row;
                }
                _ => {
                    self.music_view = MusicView::Root;
                    self.index = 0;
                }
            },
            Key::SoftLeft if self.music_view == MusicView::Root => {
                self.overlay = Some(Overlay::Menu {
                    kind: MenuKind::MusicLibrary,
                    index: 0,
                })
            }
            Key::SoftLeft if self.music_view == MusicView::Playlists => {
                self.open_local_text(TextTarget::PlaylistName, String::new(), false)
            }
            Key::SoftLeft if self.music_view == MusicView::Playlist => {
                self.playlist_row = self.index;
                self.music_view = MusicView::PlaylistMenu;
                self.index = 0;
            }
            Key::Center if n > 0 => match self.music_view {
                MusicView::Root => match self.index {
                    0 if self.music_loading => self.notice("正在刷新曲库，请稍候。"),
                    0 if !self.music.tracks.is_empty() => {
                        self.music.queue.clear();
                        self.play_track(0);
                    }
                    0 => self.notice("没有可播放的音乐。"),
                    1 => {
                        self.music_view = MusicView::Songs;
                        self.index = 0;
                    }
                    2 => {
                        self.music_view = MusicView::Playlists;
                        self.index = 0;
                    }
                    3 => self.music_view = MusicView::Player,
                    _ => {}
                },
                MusicView::Songs => {
                    self.music.queue.clear();
                    self.play_track(self.index);
                }
                MusicView::Queue => {
                    let order = self.music.order();
                    if let Some(i) = order.get(self.index) {
                        self.play_track(*i);
                    }
                }
                MusicView::Playlists => {
                    self.playlist = self.index;
                    self.music_view = MusicView::Playlist;
                    self.index = 0;
                }
                MusicView::Playlist => {
                    let tracks = self.playlist_tracks();
                    if let Some(i) = tracks.get(self.index) {
                        let i = *i;
                        self.music.queue = tracks;
                        self.play_track(i);
                    }
                }
                MusicView::AddPlaylist if self.index == 0 => {
                    self.open_local_text(TextTarget::PlaylistName, String::new(), false)
                }
                MusicView::AddPlaylist => {
                    let i = self.index - 1;
                    self.playlist_add(i);
                    self.music_view = MusicView::Player;
                }
                MusicView::PlaylistMenu => match self.index {
                    0 => self.open_local_text(
                        TextTarget::PlaylistName,
                        self.local_data.playlists[self.playlist].name.clone(),
                        false,
                    ),
                    1 => {
                        self.music_view = MusicView::DeletePlaylist;
                        self.index = 0;
                    }
                    2 => {
                        let available = self.playlist_tracks();
                        if let Some(index) = available.get(self.playlist_row) {
                            let path = &self.music_paths[*index];
                            self.local_data.playlists[self.playlist]
                                .paths
                                .retain(|p| p != path);
                            self.local_dirty = true;
                        }
                        self.music_view = MusicView::Playlist;
                        self.index = 0;
                    }
                    _ => {}
                },
                MusicView::DeletePlaylist => {
                    if self.index == 1 && self.playlist < self.local_data.playlists.len() {
                        self.local_data.playlists.remove(self.playlist);
                        self.local_dirty = true;
                    }
                    self.music_view = MusicView::Playlists;
                    self.index = 0;
                }
                MusicView::Repeat => {
                    self.music.repeat = [Repeat::Off, Repeat::All, Repeat::One][self.index];
                    self.music_view = MusicView::Player;
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
        if self.music_loading {
            self.notice("正在刷新曲库，请稍候。");
            return;
        }
        if index >= self.music.tracks.len() {
            return;
        }
        self.music.play(index);
        self.music_started = true;
        self.music_view = MusicView::Player;
    }
    pub(super) fn music_action(&mut self, index: usize) {
        match index {
            0 => {
                self.music_view = MusicView::Repeat;
                self.index = match self.music.repeat {
                    Repeat::Off => 0,
                    Repeat::All => 1,
                    Repeat::One => 2,
                };
            }
            1 => self.music.shuffle = !self.music.shuffle,
            2 => {
                self.music_view = MusicView::Queue;
                self.index = self
                    .music
                    .order()
                    .iter()
                    .position(|i| *i == self.music.index)
                    .unwrap_or(0);
            }
            3 => {
                self.music_view = MusicView::AddPlaylist;
                self.index = 0;
            }
            4 => {
                if let Some(path) = self.music_paths.get(self.music.index) {
                    self.preferences.ringtone = path.clone();
                    self.preferences_dirty = true;
                    self.notice("已设为铃声。");
                }
            }
            _ => self.notice("分享服务暂未接入。"),
        }
    }
    pub fn playlist_tracks(&self) -> Vec<usize> {
        self.local_data
            .playlists
            .get(self.playlist)
            .map(|list| {
                list.paths
                    .iter()
                    .filter_map(|p| self.music_index(p))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn music_index(&self, path: &str) -> Option<usize> {
        self.music_lookup.get(path).copied()
    }
    pub fn set_music_paths(&mut self, paths: Vec<String>) {
        self.music_lookup = paths
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, p)| (p, i))
            .collect();
        self.music_paths = paths;
    }
    pub fn append_music_path(&mut self, path: String) {
        self.music_lookup
            .insert(path.clone(), self.music_paths.len());
        self.music_paths.push(path);
    }
    fn playlist_add(&mut self, index: usize) {
        if let (Some(list), Some(path)) = (
            self.local_data.playlists.get_mut(index),
            self.music_paths.get(self.music.index),
        ) && list.paths.len() < 1000
            && !list.paths.contains(path)
        {
            list.paths.push(path.clone());
            self.local_dirty = true;
        }
    }
    pub(super) fn playlist_name(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty()
            || name.chars().count() > 80
            || self.local_data.playlists.iter().enumerate().any(|(i, l)| {
                l.name == name
                    && !(self.music_view == MusicView::PlaylistMenu && i == self.playlist)
            })
        {
            self.notice("列表名称为空或已存在。");
            return;
        }
        if self.music_view == MusicView::PlaylistMenu {
            self.local_data.playlists[self.playlist].name = name.into();
            self.local_dirty = true;
            self.music_view = MusicView::Playlist;
            self.index = self.playlist_row;
            return;
        }
        if self.local_data.playlists.len() >= 100 {
            self.notice("播放列表已满。");
            return;
        }
        let index = self.local_data.playlists.len();
        self.local_data.playlists.push(Playlist {
            name: name.into(),
            paths: vec![],
        });
        self.local_dirty = true;
        if self.music_view == MusicView::AddPlaylist {
            self.playlist_add(index);
            self.music_view = MusicView::Player;
        } else {
            self.index = index;
        }
    }
}
