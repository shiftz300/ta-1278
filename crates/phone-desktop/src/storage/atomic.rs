//! Shared bounded JSON I/O. An incomplete write never replaces the current file.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub fn read(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Cannot open {}: {e}", path.display())),
    };
    if file.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err(format!(
            "{} exceeds limit; original retained",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Store grew beyond its limit; original retained".into());
    }
    Ok(Some(bytes))
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn replace(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let (path, mut file) = loop {
        let path = dir.join(format!(
            ".{name}.{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match options.open(&path) {
            Ok(file) => break (path, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    };
    let temporary = Temporary(path);
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temporary.0, dir.join(name)).map_err(|e| e.to_string())?;
    // After rename the new file is committed. Unsupported directory fsync must
    // not report a failed transaction and roll back already committed SMS UI.
    #[cfg(unix)]
    if let Ok(directory) = File::open(dir) {
        let _ = directory.sync_all();
    }
    Ok(())
}
pub fn write(dir: &Path, name: &str, bytes: &[u8], limit: usize) -> Result<(), String> {
    if bytes.len() > limit {
        return Err(format!("{name} exceeds limit; original retained"));
    }
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if let Some(previous) = read(&dir.join(name), limit as u64)? {
        // Preserve an externally corrupted current file rather than overwriting
        // it with an old in-memory copy. Recovery is an explicit user decision.
        serde_json::from_slice::<serde_json::Value>(&previous)
            .map_err(|e| format!("Cannot replace damaged {name}; original retained: {e}"))?;
        if previous == bytes {
            return Ok(());
        }
        replace(dir, &format!("{name}.bak"), &previous)?;
    }
    replace(dir, name, bytes)
}

/// The OS releases this lock after a normal exit or crash. The lock file can stay.
/// Hold it for the lifetime of an application data directory session.
#[cfg(any(feature = "desktop", test))]
pub struct DataLock {
    _file: File,
}
#[cfg(any(feature = "desktop", test))]
impl DataLock {
    pub fn acquire(dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(dir.join(".instance.lock"))
            .map_err(|e| e.to_string())?;
        file.try_lock().map_err(|e| {
            format!("Cannot lock this data directory; another instance may be using it: {e}")
        })?;
        Ok(Self { _file: file })
    }
}
