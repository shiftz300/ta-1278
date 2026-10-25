//! In-process phone volumes. Paths are virtual; callers never receive host paths.
//! This is not an OS mount or a firmware filesystem emulator.
mod directory;
pub use directory::Directory;
use std::collections::BTreeMap;
use std::io::{self, Read, Seek};

pub trait ReadSeek: Read + Seek + Send + Sync {}
impl<T: Read + Seek + Send + Sync> ReadSeek for T {}
pub struct OpenFile {
    pub reader: Box<dyn ReadSeek>,
    pub len: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub len: u64,
}
pub trait Volume: Send + Sync {
    fn list(&self, path: &str) -> io::Result<Vec<Entry>>;
    fn open(&self, path: &str) -> io::Result<OpenFile>;
    fn mkdir(&self, path: &str) -> io::Result<()>;
    fn rename(&self, from: &str, to: &str) -> io::Result<()>;
    /// Directories must be empty. Never recursively erase a tree.
    fn remove(&self, path: &str) -> io::Result<()>;
}
struct Mount {
    label: String,
    read_only: bool,
    volume: Box<dyn Volume>,
}
#[derive(Default)]
pub struct Vfs {
    mounts: BTreeMap<String, Mount>,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name
            .chars()
            .any(|c| c == '/' || c == '\\' || c.is_control())
}
fn components(path: &str) -> io::Result<Vec<&str>> {
    if !path.starts_with('/') {
        return Err(invalid("Expected an absolute virtual path"));
    }
    if path == "/" {
        return Ok(vec![]);
    }
    let parts: Vec<_> = path[1..].split('/').collect();
    if parts.iter().any(|p| !valid_name(p)) {
        return Err(invalid("Invalid virtual path"));
    }
    Ok(parts)
}
impl Vfs {
    pub fn mount(
        &mut self,
        name: &str,
        label: &str,
        volume: Box<dyn Volume>,
        read_only: bool,
    ) -> io::Result<()> {
        if !valid_name(name) || self.mounts.contains_key(name) {
            return Err(invalid("Invalid or occupied mount"));
        }
        self.mounts.insert(
            name.into(),
            Mount {
                label: label.into(),
                read_only,
                volume,
            },
        );
        Ok(())
    }
    pub fn unmount(&mut self, name: &str) -> bool {
        self.mounts.remove(name).is_some()
    }
    fn resolve(&self, path: &str, write: bool) -> io::Result<(&Mount, String)> {
        let parts = components(path)?;
        let name = parts.first().ok_or_else(|| invalid("Volume required"))?;
        let mount = self
            .mounts
            .get(*name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Volume not mounted"))?;
        if write && (mount.read_only || parts.len() == 1) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Read-only storage or mount root",
            ));
        }
        Ok((mount, parts[1..].join("/")))
    }
    pub fn list(&self, path: &str) -> io::Result<Vec<Entry>> {
        if path == "/" {
            return Ok(self
                .mounts
                .iter()
                .map(|(name, m)| Entry {
                    name: m.label.clone(),
                    path: format!("/{name}"),
                    directory: true,
                    len: 0,
                })
                .collect());
        }
        let (m, relative) = self.resolve(path, false)?;
        let mut entries = m.volume.list(&relative)?;
        for entry in &mut entries {
            if !valid_name(&entry.name) {
                return Err(invalid("Backend returned invalid name"));
            }
            entry.path = format!("{path}/{}", entry.name);
        }
        entries.sort_by(|a, b| {
            b.directory
                .cmp(&a.directory)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(entries)
    }
    pub fn open(&self, path: &str) -> io::Result<OpenFile> {
        let (m, p) = self.resolve(path, false)?;
        m.volume.open(&p)
    }
    pub fn writable(&self, path: &str) -> bool {
        self.resolve(path, false).is_ok_and(|(m, _)| !m.read_only)
    }
    pub fn mkdir(&self, path: &str) -> io::Result<()> {
        let (m, p) = self.resolve(path, true)?;
        m.volume.mkdir(&p)
    }
    pub fn remove(&self, path: &str) -> io::Result<()> {
        let (m, p) = self.resolve(path, true)?;
        m.volume.remove(&p)
    }
    pub fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let (a, src) = self.resolve(from, true)?;
        let (b, dst) = self.resolve(to, true)?;
        if !std::ptr::eq(a, b) {
            return Err(invalid("Cross-volume moves are not supported"));
        }
        a.volume.rename(&src, &dst)
    }
}
#[cfg(test)]
mod tests;
