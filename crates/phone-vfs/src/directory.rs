use super::*;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

/// Directory adapter for a user-selected, trusted local directory.
/// Rejects symlinks and special files. Not an adversarial OS sandbox: concurrent
/// host renames between validation and I/O are outside this adapter's contract.
pub struct Directory {
    root: PathBuf,
}
impl Directory {
    pub fn new(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = fs::canonicalize(root)?;
        if !root.is_dir() {
            return Err(invalid("Mount must be a directory"));
        }
        Ok(Self { root })
    }
    fn path(&self, relative: &str, new_leaf: bool) -> io::Result<PathBuf> {
        if !self.root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Storage unavailable",
            ));
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
impl Volume for Directory {
    fn create(&self, relative: &str, source: &mut dyn Read, limit: u64) -> io::Result<()> {
        let path = self.path(relative, true)?;
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
    fn list(&self, relative: &str) -> io::Result<Vec<Entry>> {
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
                path: String::new(),
                directory: kind.is_dir(),
                len: item.metadata()?.len(),
            });
        }
        Ok(entries)
    }
    fn open(&self, relative: &str) -> io::Result<OpenFile> {
        let path = self.path(relative, false)?;
        if !path.is_file() {
            return Err(invalid("Expected a regular file"));
        }
        let file = fs::File::open(path)?;
        let len = file.metadata()?.len();
        Ok(OpenFile {
            reader: Box::new(file),
            len,
        })
    }
    fn mkdir(&self, relative: &str) -> io::Result<()> {
        fs::create_dir(self.path(relative, true)?)
    }
    fn remove(&self, relative: &str) -> io::Result<()> {
        let path = self.path(relative, false)?;
        if path.is_dir() {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        }
    }
    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let src = self.path(from, false)?;
        let dst = self.path(to, true)?;
        if dst.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination already exists",
            ));
        }
        fs::rename(src, dst)
    }
}
