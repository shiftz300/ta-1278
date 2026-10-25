use phone_core::sms::Mailbox;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
pub fn default_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
            .join("Library/Application Support/TA1278 Replica")
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
            })
            .join("ta1278-replica")
    }
}
pub fn load(dir: &Path) -> Result<Mailbox, String> {
    let path = dir.join("messages.json");
    if !path.exists() {
        return Ok(Mailbox::default());
    }
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() > 8 * 1024 * 1024 {
        return Err("Local store exceeds 8 MiB; original retained".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Cannot read local store (retained): {e}"))
}
pub fn save(dir: &Path, mailbox: &Mailbox) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(mailbox).map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("Local store is full; delete some entries".into());
    }
    let tmp = dir.join("messages.json.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(&tmp).map_err(|e| e.to_string())?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    std::fs::rename(tmp, dir.join("messages.json")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn save_reload_and_corrupt_file_is_retained() {
        let dir = std::env::temp_dir().join(format!("ta1278-store-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut m = Mailbox::default();
        m.save(
            None,
            phone_core::sms::Message {
                to: "123".into(),
                body: "中文".into(),
            },
        );
        save(&dir, &m).unwrap();
        assert_eq!(load(&dir).unwrap().drafts, m.drafts);
        std::fs::write(dir.join("messages.json"), b"{bad").unwrap();
        assert!(load(&dir).is_err());
        assert_eq!(std::fs::read(dir.join("messages.json")).unwrap(), b"{bad");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
