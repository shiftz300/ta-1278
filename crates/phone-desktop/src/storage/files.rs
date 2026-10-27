//! Direct access to one user-selected local folder. No mounts or backend traits.
use std::io::{self, Read};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    /// Slash-prefixed path relative to the selected folder, never a host path.
    pub path: String,
    pub directory: bool,
    pub len: u64,
}
pub struct OpenFile {
    pub reader: std::fs::File,
    pub len: u64,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
pub fn valid_name(name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let stem = name
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end()
            .to_ascii_uppercase();
        if name.ends_with(['.', ' '])
            || name.chars().any(|c| "<>:\"|?*".contains(c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return false;
        }
    }
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name
            .chars()
            .any(|c| c == '/' || c == '\\' || c.is_control())
}
pub fn child(parent: &str, name: &str) -> String {
    format!("{}/{name}", parent.trim_end_matches('/'))
}
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

/// Directory adapter for a user-selected, trusted local directory.
/// Rejects symlinks and special files. Not an adversarial OS sandbox: concurrent
/// host renames between validation and I/O are outside this adapter's contract.
pub struct Folder {
    root: PathBuf,
    read_only: bool,
}
impl Folder {
    pub fn capacity(&self) -> Option<(u64, u64)> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new(crate::media::visual::helper().ok()?)
                .arg("folder-stats")
                .arg(&self.root)
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let [total, free]: [u64; 2] = serde_json::from_slice(&output.stdout).ok()?;
            Some((total, free.min(total)))
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }
    pub fn new(root: impl AsRef<Path>, read_only: bool) -> io::Result<Self> {
        let root = fs::canonicalize(root)?;
        if !root.is_dir() {
            return Err(invalid("Storage must be a directory"));
        }
        Ok(Self { root, read_only })
    }
    fn path(&self, relative: &str, new_leaf: bool) -> io::Result<PathBuf> {
        if !self.root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Storage unavailable",
            ));
        }
        let relative = relative
            .strip_prefix('/')
            .ok_or_else(|| invalid("Expected folder-relative path"))?;
        if !relative.is_empty() && relative.split('/').any(|p| !valid_name(p)) {
            return Err(invalid("Invalid folder-relative path"));
        }
        let mut path = self.root.clone();
        let parts: Vec<_> = Path::new(relative).components().collect();
        for (i, c) in parts.iter().enumerate() {
            let Component::Normal(name) = c else {
                return Err(invalid("Invalid relative path"));
            };
            if !name.to_str().is_some_and(valid_name) {
                return Err(invalid("Unsupported file name"));
            }
            path.push(name);
            match fs::symlink_metadata(&path) {
                Ok(m) if m.file_type().is_symlink() || (!m.is_file() && !m.is_dir()) => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Links and special files are unavailable",
                    ));
                }
                Ok(_) => {}
                Err(e)
                    if new_leaf && i + 1 == parts.len() && e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
        Ok(path)
    }
}
impl Folder {
    pub fn writable(&self, path: &str) -> bool {
        !self.read_only && self.path(path, false).is_ok()
    }
    fn write_path(&self, path: &str, new_leaf: bool) -> io::Result<PathBuf> {
        if self.read_only || path == "/" {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Read-only storage or root folder",
            ));
        }
        self.path(path, new_leaf)
    }
    fn create(&self, relative: &str, source: &mut dyn Read, limit: u64) -> io::Result<()> {
        let path = self.write_path(relative, true)?;
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut out = options.open(&path)?;
        let result = std::io::copy(&mut source.take(limit + 1), &mut out).and_then(|n| {
            if n > limit {
                Err(invalid("Source grew during copy"))
            } else {
                out.sync_all()
            }
        });
        drop(out);
        if result.is_err() {
            let _ = fs::remove_file(&path);
        }
        result
    }
    pub fn list(&self, relative: &str) -> io::Result<Vec<Entry>> {
        let mut entries = vec![];
        for item in fs::read_dir(self.path(relative, false)?)? {
            let item = item?;
            let name = item.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if !valid_name(name) || name.starts_with('.') {
                continue;
            }
            let kind = item.file_type()?;
            if !kind.is_file() && !kind.is_dir() {
                continue;
            }
            if entries.len() >= 10000 {
                return Err(invalid("Directory exceeds 10000 visible entries"));
            }
            entries.push(Entry {
                name: name.into(),
                path: child(relative, name),
                directory: kind.is_dir(),
                len: item.metadata()?.len(),
            });
        }
        entries.sort_by(|a, b| {
            b.directory
                .cmp(&a.directory)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(entries)
    }
    pub fn open(&self, relative: &str) -> io::Result<OpenFile> {
        let path = self.path(relative, false)?;
        if !path.is_file() {
            return Err(invalid("Expected a regular file"));
        }
        let file = fs::File::open(path)?;
        let len = file.metadata()?.len();
        Ok(OpenFile { reader: file, len })
    }
    pub fn mkdir(&self, relative: &str) -> io::Result<()> {
        fs::create_dir(self.write_path(relative, true)?)
    }
    pub fn remove(&self, relative: &str) -> io::Result<()> {
        let path = self.write_path(relative, false)?;
        if path.is_dir() {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    }
    /// Preflight every selected entry. Never recursively removes a directory.
    /// Host changes after preflight can still cause a partial result, reported per path.
    pub fn remove_many(&self, paths: &[String]) -> (Vec<String>, Vec<(String, String)>) {
        if paths.is_empty() || paths.len() > 1000 {
            return (
                vec![],
                vec![(String::new(), "Select 1–1000 entries".into())],
            );
        }
        for name in paths {
            let check = (|| {
                let path = self.write_path(name, false)?;
                if path.is_dir() && fs::read_dir(path)?.next().transpose()?.is_some() {
                    return Err(invalid("Folder is not empty"));
                }
                Ok(())
            })();
            if let Err(e) = check {
                return (vec![], vec![(name.clone(), e.to_string())]);
            }
        }
        let mut removed = vec![];
        let mut failed = vec![];
        for name in paths.iter().collect::<std::collections::BTreeSet<_>>() {
            match self.remove(name) {
                Ok(()) => removed.push(name.clone()),
                Err(e) => failed.push((name.clone(), e.to_string())),
            }
        }
        (removed, failed)
    }
    pub fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let src = self.write_path(from, false)?;
        let dst = self.write_path(to, true)?;
        if dst.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination already exists",
            ));
        }
        fs::rename(src, dst)
    }
    /// Bounded recursive copy. No overwrite; failed copies roll back new entries.
    /// A move uses rename on the same host filesystem, with no copy/delete fallback.
    pub fn transfer(&self, from: &str, to: &str, moving: bool) -> io::Result<()> {
        self.path(from, false)?;
        self.write_path(to, true)?;
        if from == "/" || to == from || to.starts_with(&format!("{from}/")) {
            return Err(invalid("Invalid copy/move destination"));
        }
        if moving {
            return self.rename(from, to);
        }
        let parent = from.rsplit_once('/').unwrap().0;
        let parent = if parent.is_empty() { "/" } else { parent };
        let root = self
            .list(parent)?
            .into_iter()
            .find(|e| e.path == from)
            .ok_or_else(|| invalid("Source unavailable"))?;
        let mut nodes = vec![(root, to.to_owned(), 0)];
        let mut cursor = 0;
        let mut size = 0u64;
        while cursor < nodes.len() {
            let (entry, dest, depth) = nodes[cursor].clone();
            if nodes.len() > 10000 || depth > 16 {
                return Err(invalid("Transfer exceeds entry/depth limit"));
            }
            if entry.directory {
                for child in self.list(&entry.path)? {
                    let target = format!("{dest}/{}", child.name);
                    nodes.push((child, target, depth + 1));
                }
            } else {
                size = size.saturating_add(entry.len);
                if size > 512 * 1024 * 1024 {
                    return Err(invalid("Transfer exceeds 512 MiB"));
                }
            }
            cursor += 1;
        }
        let mut created: Vec<String> = vec![];
        for (entry, dest, _) in &nodes {
            let result = if entry.directory {
                self.mkdir(dest)
            } else {
                (|| {
                    let mut source = self.open(&entry.path)?;
                    self.create(dest, &mut source.reader, entry.len)
                })()
            };
            if let Err(e) = result {
                for path in created.iter().rev() {
                    let _ = self.remove(path);
                }
                return Err(e);
            }
            created.push(dest.clone());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
