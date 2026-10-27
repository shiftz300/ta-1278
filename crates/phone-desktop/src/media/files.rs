//! File I/O, bounded transfers and path-reference updates.
use super::*;
impl Media {
    pub(super) fn finish_file_jobs(&mut self, shell: &mut Shell) {
        while let Ok(result) = self.transfers.try_recv() {
            shell.files.busy = false;
            match result {
                FileResult::Transfer(TransferResult {
                    result,
                    from,
                    to,
                    moving,
                }) => {
                    if let Err(e) = result {
                        shell.notice(format!("复制/移动失败：{e}"));
                    } else {
                        shell.finish_file_transfer();
                        self.prepare_refresh(shell);
                        if moving {
                            self.remap_links(shell, &from, &to);
                        }
                    }
                    if let Err(e) = self.list(shell, shell.files.path.clone()) {
                        shell.notice(e);
                    }
                }
                FileResult::Remove { removed, failed } => {
                    if !removed.is_empty() {
                        self.prepare_refresh(shell);
                        if let Some(saved) = &mut shell.playback.resume
                            && saved
                                .path
                                .as_ref()
                                .is_some_and(|p| removed.iter().any(|r| path_contains(r, p)))
                        {
                            saved.path = None;
                            saved.position = 0.;
                        }
                    }
                    shell.files.marked.retain(|p| !removed.contains(p));
                    if let Err(e) = self.list(shell, shell.files.path.clone()) {
                        shell.notice(e);
                    }
                    if let Some((_, error)) = failed.first() {
                        shell.files.view = FileView::DeleteSelect;
                        shell.notice(format!("已删除 {} 项；未完成：{}", removed.len(), error));
                    } else {
                        shell.files.marked.clear();
                    }
                }
            }
        }
    }
    pub(super) fn list(&self, shell: &mut Shell, path: String) -> Result<(), String> {
        let mut entries = self.files.list(&path).map_err(|e| e.to_string())?;
        if shell.files.transfer.is_some() {
            entries.retain(|e| e.directory);
        }
        shell.files.writable = self.files.writable(&path);
        shell.files.path = path;
        shell.files.entries = entries;
        shell.files.view = FileView::Browse;
        if shell.screen == Screen::Tools(crate::shell::tools::ToolPage::Files) {
            shell.index = shell.index.min(shell.files.entries.len().saturating_sub(1));
        }
        Ok(())
    }
    pub(super) fn file_request(
        &mut self,
        shell: &mut Shell,
        request: FileRequest,
    ) -> Result<(), String> {
        if !shell.features.files {
            return Err("文件功能已停用。".into());
        }
        match request {
            FileRequest::Storage => {
                shell.files.capacity = self.files.capacity();
                shell.files.view = FileView::Storage;
                Ok(())
            }
            FileRequest::Transfer(from, to, moving) => {
                shell.files.busy = true;
                let files = self.files.clone();
                let tx = self.transfer_results.clone();
                thread::spawn(move || {
                    let result = files
                        .transfer(&from, &to, moving)
                        .map_err(|e| e.to_string());
                    let _ = tx.send(FileResult::Transfer(TransferResult {
                        result,
                        from,
                        to,
                        moving,
                    }));
                });
                Ok(())
            }
            FileRequest::RemoveMany(paths) => {
                shell.files.busy = true;
                let files = self.files.clone();
                let tx = self.transfer_results.clone();
                thread::spawn(move || {
                    let (removed, failed) = files.remove_many(&paths);
                    let _ = tx.send(FileResult::Remove { removed, failed });
                });
                Ok(())
            }
            FileRequest::List(path) => self.list(shell, path),
            FileRequest::Enter(path) => {
                let previous = (shell.files.path.clone(), shell.index);
                self.list(shell, path)?;
                shell.files.history.push(previous);
                shell.index = 0;
                Ok(())
            }
            FileRequest::Parent => {
                if let Some((path, index)) = shell.files.history.last().cloned() {
                    self.list(shell, path)?;
                    shell.files.history.pop();
                    shell.index = index.min(shell.files.entries.len().saturating_sub(1));
                }
                Ok(())
            }
            FileRequest::Play(path) => {
                use crate::shell::media::{Purpose, Request};
                if Purpose::Pictures.accepts(&path) || Purpose::Videos.accepts(&path) {
                    let purpose = if Purpose::Videos.accepts(&path) {
                        Purpose::Videos
                    } else {
                        Purpose::Pictures
                    };
                    shell.open_media(purpose);
                    if shell.screen != Screen::Media {
                        return Err("此功能已停用。".into());
                    }
                    shell.media.ui.selected = path.clone();
                    shell.media.ui.viewing = true;
                    shell.media.ui.direct = true;
                    shell.media.ui.entries = shell
                        .files
                        .entries
                        .iter()
                        .filter(|e| !e.directory && purpose.accepts(&e.path))
                        .cloned()
                        .collect();
                    shell.media.ui.pending = vec![Request::Open(path)];
                    return Ok(());
                }
                if self.scanning || shell.playback.loading {
                    return Err("正在刷新曲库，请稍候。".into());
                }
                if !shell.features.music {
                    return Err("音乐功能已停用。".into());
                }
                let track = probe(&self.files, self.decoder.as_ref(), &path)?;
                let index = if let Some(i) = self.tracks.iter().position(|t| t.path == path) {
                    i
                } else {
                    shell
                        .playback
                        .model
                        .tracks
                        .push((track.title.clone(), track.seconds));
                    self.tracks.push(track);
                    shell.append_music_path(path.clone());
                    self.tracks.len() - 1
                };
                shell.playback.resume = None;
                shell.files.selected = shell.index;
                shell.go(Screen::Music);
                shell.playback.model.queue.clear();
                shell.play_track(index);
                shell.playback.return_file = true;
                Ok(())
            }
            FileRequest::Mkdir(path) => {
                self.files.mkdir(&path).map_err(|e| e.to_string())?;
                self.list(shell, shell.files.path.clone())
            }
            FileRequest::Rename(from, to) => {
                self.files.rename(&from, &to).map_err(|e| e.to_string())?;
                self.prepare_refresh(shell);
                self.remap_links(shell, &from, &to);
                self.list(shell, shell.files.path.clone())
            }
        }
    }
    pub(super) fn remap_links(&mut self, shell: &mut Shell, from: &str, to: &str) {
        let remap = |path: &mut String| {
            if path_contains(from, path) {
                *path = format!("{to}{}", &path[from.len()..]);
                true
            } else {
                false
            }
        };
        if let Some(saved) = &mut shell.playback.resume {
            if let Some(path) = &mut saved.path {
                remap(path);
            }
            for path in &mut saved.queue {
                remap(path);
            }
        }
        let mut changed = false;
        for row in &mut shell.contacts.records {
            changed |= remap(&mut row[0]);
            changed |= remap(&mut row[7]);
        }
        for event in &mut shell.organizer.events {
            changed |= remap(&mut event.fields[4]);
        }
        for alarm in &mut shell.organizer.alarms {
            changed |= remap(&mut alarm.fields[3]);
        }
        for list in &mut shell.playback.playlists {
            for path in &mut list.paths {
                changed |= remap(path);
            }
        }
        shell.records_dirty |= changed;
        if let Some(path) = &mut shell.settings.preferences.wallpaper {
            shell.settings.dirty |= remap(path);
        }
        shell.settings.dirty |= remap(&mut shell.settings.preferences.ringtone);
        shell.media.pictures.clear();
    }
    pub(super) fn prepare_refresh(&mut self, shell: &mut Shell) {
        if shell.playback.resume.is_none() {
            shell.playback.resume = Some(shell.playback.snapshot());
        }
        self.audio.stop();
        shell.playback.model.playing = false;
        shell.playback.loading = shell.features.music;
        shell.playback.refresh = shell.features.music;
    }
}
