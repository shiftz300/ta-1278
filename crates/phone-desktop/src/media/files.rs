//! File I/O, bounded transfers and path-reference updates.
use super::*;
impl Media {
    pub(super) fn list(&self, phone: &mut Phone, path: String) -> Result<(), String> {
        let entries = self.vfs.list(&path).map_err(|e| e.to_string())?;
        phone.files.writable = self.vfs.writable(&path);
        phone.files.path = path;
        phone.files.entries = entries;
        phone.files.view = FileView::Browse;
        if phone.screen == Screen::Tools(crate::phone::tools::ToolPage::Files) {
            phone.index = phone.index.min(phone.files.entries.len().saturating_sub(1));
        }
        Ok(())
    }
    pub(super) fn file_request(
        &mut self,
        phone: &mut Phone,
        request: FileRequest,
    ) -> Result<(), String> {
        if !phone.features.files {
            return Err("文件功能已停用。".into());
        }
        match request {
            FileRequest::Transfer(from, to, moving) => {
                phone.files.busy = true;
                let vfs = self.vfs.clone();
                let tx = self.transfer_results.clone();
                thread::spawn(move || {
                    let result = vfs.transfer(&from, &to, moving).map_err(|e| e.to_string());
                    let _ = tx.send(TransferResult {
                        result,
                        from,
                        to,
                        moving,
                    });
                });
                Ok(())
            }
            FileRequest::List(path) => self.list(phone, path),
            FileRequest::Enter(path) => {
                let previous = (phone.files.path.clone(), phone.index);
                self.list(phone, path)?;
                phone.files.history.push(previous);
                phone.index = 0;
                Ok(())
            }
            FileRequest::Parent => {
                if let Some((path, index)) = phone.files.history.last().cloned() {
                    self.list(phone, path)?;
                    phone.files.history.pop();
                    phone.index = index.min(phone.files.entries.len().saturating_sub(1));
                }
                Ok(())
            }
            FileRequest::Play(path) => {
                use crate::phone::media_browser::{Purpose, Request};
                if Purpose::Pictures.accepts(&path) || Purpose::Videos.accepts(&path) {
                    let purpose = if Purpose::Videos.accepts(&path) {
                        Purpose::Videos
                    } else {
                        Purpose::Pictures
                    };
                    phone.open_media(purpose);
                    if phone.screen != Screen::Media {
                        return Err("此功能已停用。".into());
                    }
                    phone.media_ui.selected = path.clone();
                    phone.media_ui.viewing = true;
                    phone.media_ui.direct = true;
                    phone.media_ui.entries = phone
                        .files
                        .entries
                        .iter()
                        .filter(|e| !e.directory && purpose.accepts(&e.path))
                        .cloned()
                        .collect();
                    phone.media_ui.pending = vec![Request::Open(path)];
                    return Ok(());
                }
                if self.scanning || phone.music_loading {
                    return Err("正在刷新曲库，请稍候。".into());
                }
                if !phone.features.music {
                    return Err("音乐功能已停用。".into());
                }
                let track = probe(&self.vfs, &path)?;
                let index = if let Some(i) = self.tracks.iter().position(|t| t.path == path) {
                    i
                } else {
                    phone
                        .music
                        .tracks
                        .push((track.title.clone(), track.seconds));
                    self.tracks.push(track);
                    phone.append_music_path(path.clone());
                    self.tracks.len() - 1
                };
                self.restore = None;
                phone.files.selected = phone.index;
                phone.go(Screen::Music);
                phone.music.queue.clear();
                phone.play_track(index);
                phone.music_return_file = true;
                Ok(())
            }
            FileRequest::Mkdir(path) => {
                self.vfs.mkdir(&path).map_err(|e| e.to_string())?;
                self.list(phone, phone.files.path.clone())
            }
            FileRequest::Remove(path) => {
                self.vfs.remove(&path).map_err(|e| e.to_string())?;
                self.prepare_refresh(phone);
                if let Some(saved) = &mut self.restore
                    && saved.path.as_ref().is_some_and(|p| path_contains(&path, p))
                {
                    saved.path = None;
                    saved.position = 0.;
                }
                self.list(phone, phone.files.path.clone())
            }
            FileRequest::Rename(from, to) => {
                self.vfs.rename(&from, &to).map_err(|e| e.to_string())?;
                self.prepare_refresh(phone);
                self.remap_links(phone, &from, &to);
                self.list(phone, phone.files.path.clone())
            }
        }
    }
    pub(super) fn remap_links(&mut self, phone: &mut Phone, from: &str, to: &str) {
        let remap = |path: &mut String| {
            if path_contains(from, path) {
                *path = format!("{to}{}", &path[from.len()..]);
                true
            } else {
                false
            }
        };
        if let Some(saved) = &mut self.restore {
            if let Some(path) = &mut saved.path {
                remap(path);
            }
            for path in &mut saved.queue {
                remap(path);
            }
        }
        let mut changed = false;
        for row in &mut phone.local_data.contacts {
            changed |= remap(&mut row[0]);
            changed |= remap(&mut row[7]);
        }
        for event in &mut phone.local_data.events {
            changed |= remap(&mut event.fields[4]);
        }
        for alarm in &mut phone.local_data.alarms {
            changed |= remap(&mut alarm.fields[3]);
        }
        for list in &mut phone.local_data.playlists {
            for path in &mut list.paths {
                changed |= remap(path);
            }
        }
        phone.local_dirty |= changed;
        if let Some(path) = &mut phone.preferences.wallpaper {
            phone.preferences_dirty |= remap(path);
        }
        phone.preferences_dirty |= remap(&mut phone.preferences.ringtone);
        phone.pictures.clear();
    }
    pub(super) fn prepare_refresh(&mut self, phone: &mut Phone) {
        if self.restore.is_none() {
            self.restore = Some(SavedMusic::capture(phone, &self.tracks));
        }
        self.audio.stop();
        phone.music.playing = false;
        phone.music_loading = phone.features.music;
        phone.music_refresh = phone.features.music;
    }
}
