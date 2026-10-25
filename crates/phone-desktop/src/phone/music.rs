//! Local music library/player navigation. Audio transport is deliberately separate.
use super::*;
use phone_core::music::Repeat;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicView {
    Root,
    Songs,
    Playlists,
    Player,
    Repeat,
    Queue,
}
impl Phone {
    pub fn music_items(&self) -> Vec<String> {
        match self.music_view {
            MusicView::Root => {
                let mut labels = vec!["播放全部".into(), "歌曲".into(), "播放列表".into()];
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
            MusicView::Songs | MusicView::Queue => {
                self.music.tracks.iter().map(|t| t.0.clone()).collect()
            }
            MusicView::Playlists => vec![],
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
                Key::SoftRight => {
                    self.music_view = MusicView::Root;
                    self.index = 0;
                }
                _ => {}
            }
            return;
        }
        let n = self.music_items().len();
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
            Key::Center if n > 0 => match self.music_view {
                MusicView::Root => match self.index {
                    0 if !self.music.tracks.is_empty() => self.play_track(0),
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
                MusicView::Songs | MusicView::Queue => self.play_track(self.index),
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
    fn play_track(&mut self, index: usize) {
        self.music.index = index;
        self.music.position = 0.;
        self.music.playing = true;
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
                self.index = self.music.index;
            }
            3 => self.notice("没有播放列表"),
            4 => self.notice("铃声服务暂未接入。"),
            _ => self.notice("分享服务暂未接入。"),
        }
    }
}
