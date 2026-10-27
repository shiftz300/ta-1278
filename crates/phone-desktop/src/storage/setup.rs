//! First-launch folder choice, kept outside the replica's phone UI.
use crate::{
    persistence::{self, Directory, Record},
    storage::files::Folder,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize)]
struct Selection {
    directory: Option<PathBuf>,
}

fn validate(data: &Path, chosen: &Path, read_only: bool) -> Result<(PathBuf, Folder), String> {
    let chosen = chosen
        .canonicalize()
        .map_err(|e| format!("Cannot open storage folder: {e}"))?;
    let data = data.canonicalize().map_err(|e| e.to_string())?;
    if data.starts_with(&chosen) {
        return Err("请选择独立的文件夹，不要选择包含程序数据的目录。".into());
    }
    let folder = Folder::new(&chosen, read_only).map_err(|e| e.to_string())?;
    Ok((chosen, folder))
}

pub fn select(
    data: &Path,
    requested: Option<PathBuf>,
    read_only: bool,
) -> Result<Option<Folder>, String> {
    resolve(data, requested, read_only, choose)
}

fn resolve(
    data: &Path,
    requested: Option<PathBuf>,
    read_only: bool,
    choose: impl FnOnce(&Path) -> Result<Option<PathBuf>, String>,
) -> Result<Option<Folder>, String> {
    let mut store = Directory::new(data);
    let saved: Selection = persistence::read(&mut store, Record::Mapping)?;
    if requested.is_none()
        && let Some(path) = &saved.directory
    {
        // Never silently replace unavailable storage with an empty directory.
        return validate(data, path, read_only).map(|(_, folder)| Some(folder));
    }
    let chosen = if let Some(path) = requested {
        Some(path)
    } else {
        choose(&data.join("files"))?
    };
    let Some(chosen) = chosen else {
        return Ok(None);
    };
    let (directory, folder) = validate(data, &chosen, read_only)?;
    persistence::write(
        &mut store,
        Record::Mapping,
        &Selection {
            directory: Some(directory),
        },
    )?;
    Ok(Some(folder))
}

#[cfg(target_os = "macos")]
fn choose(initial: &Path) -> Result<Option<PathBuf>, String> {
    let result = std::process::Command::new(crate::media::visual::helper()?)
        .arg("choose-folder")
        .arg(initial)
        .output()
        .map_err(|e| e.to_string())?;
    if result.status.code() == Some(2) {
        return Ok(None);
    }
    if !result.status.success() {
        return Err(format!(
            "Cannot choose storage folder: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    serde_json::from_slice::<PathBuf>(&result.stdout)
        .map(Some)
        .map_err(|e| e.to_string())
}
#[cfg(all(feature = "desktop", any(target_os = "windows", target_os = "linux")))]
fn choose(initial: &Path) -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title("Choose phone storage folder")
        .set_directory(initial)
        .pick_folder())
}
#[cfg(all(
    not(target_os = "macos"),
    not(all(feature = "desktop", any(target_os = "windows", target_os = "linux")))
))]
fn choose(_: &Path) -> Result<Option<PathBuf>, String> {
    Err("Choose a phone storage folder with --storage-dir DIRECTORY on first launch.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_choice_cancel_restart_read_only_and_missing_directory() {
        let root =
            std::env::temp_dir().join(format!("ta1278-folder-choice-{}", std::process::id()));
        let data = root.join("data");
        let media = root.join("media");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        assert!(resolve(&data, None, false, |_| Ok(None)).unwrap().is_none());
        assert!(!data.join("storage.json").exists());
        let folder = resolve(&data, None, false, |_| Ok(Some(media.clone())))
            .unwrap()
            .unwrap();
        folder.mkdir("/Music").unwrap();
        assert!(media.join("Music").is_dir());
        let restored = resolve(&data, None, true, |_| panic!("Must not ask twice"))
            .unwrap()
            .unwrap();
        assert!(!restored.writable("/"));
        assert!(restored.mkdir("/Denied").is_err());
        assert!(resolve(&data, Some(root.clone()), false, |_| unreachable!()).is_err());
        std::fs::rename(&media, root.join("moved")).unwrap();
        assert!(
            resolve(&data, None, false, |_| panic!(
                "Must not silently replace storage"
            ))
            .is_err()
        );
        assert!(!media.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
