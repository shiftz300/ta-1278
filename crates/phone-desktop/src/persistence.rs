//! Host-owned storage. Runtime alone captures, validates and acknowledges app data.
use crate::storage;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Record {
    Snapshot,
    Messages,
    Preferences,
    Organizer,
    Browser,
    Snake,
    Utilities,
    Input,
    Music,
    Mapping,
}
impl Record {
    pub fn name(self) -> &'static str {
        match self {
            Self::Snapshot => "state.json",
            Self::Messages => "messages.json",
            Self::Preferences => "preferences.json",
            Self::Organizer => "organizer.json",
            Self::Browser => "browser.json",
            Self::Snake => "snake.json",
            Self::Utilities => "utilities.json",
            Self::Input => "input-native.json",
            Self::Music => "music.json",
            Self::Mapping => "storage.json",
        }
    }
    pub fn limit(self) -> usize {
        match self {
            Self::Snapshot => 32 * 1024 * 1024,
            Self::Messages | Self::Organizer => 8 * 1024 * 1024,
            Self::Browser => 16 * 1024 * 1024,
            Self::Input => 2 * 1024 * 1024,
            Self::Utilities => 128 * 1024,
            Self::Preferences | Self::Music | Self::Mapping => 65536,
            Self::Snake => 4096,
        }
    }
}

/// A successful write must durably replace that record. Messages and their
/// service cursor are one record; never acknowledge a failed write.
pub trait Store {
    fn read(&mut self, record: Record) -> Result<Option<Vec<u8>>, String>;
    fn write(&mut self, record: Record, bytes: &[u8]) -> Result<(), String>;
}

/// Existing JSON profile layout, with atomic replacement and one backup per file.
/// `open` holds the existing instance lock for the lifetime of this Store.
pub struct Directory {
    path: PathBuf,
    _lock: Option<storage::atomic::DataLock>,
}
impl Directory {
    /// Use when the caller already owns the profile transaction/lock.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            _lock: None,
        }
    }
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let lock = storage::atomic::DataLock::acquire(&path)?;
        Ok(Self {
            path,
            _lock: Some(lock),
        })
    }
}
impl Store for Directory {
    fn read(&mut self, record: Record) -> Result<Option<Vec<u8>>, String> {
        storage::atomic::read(&self.path.join(record.name()), record.limit() as u64)
    }
    fn write(&mut self, record: Record, bytes: &[u8]) -> Result<(), String> {
        storage::atomic::write(&self.path, record.name(), bytes, record.limit())
    }
}

#[derive(Clone, Copy, Default)]
pub enum Resume {
    /// Embedded checkpoint: exclude time spent outside the host.
    #[default]
    Paused,
    /// Standalone profile: preserve the existing wall-clock catch-up behavior.
    WallClock,
}

pub(crate) fn read<T: serde::de::DeserializeOwned + Default>(
    store: &mut dyn Store,
    record: Record,
) -> Result<T, String> {
    match store.read(record)? {
        Some(bytes) if bytes.len() <= record.limit() => serde_json::from_slice(&bytes)
            .map_err(|e| format!("Cannot read {} (retained): {e}", record.name())),
        Some(_) => Err(format!(
            "{} exceeds limit; original retained",
            record.name()
        )),
        None => Ok(T::default()),
    }
}
pub(crate) fn write(
    store: &mut dyn Store,
    record: Record,
    value: &impl serde::Serialize,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if bytes.len() > record.limit() {
        return Err(format!(
            "{} exceeds limit; original retained",
            record.name()
        ));
    }
    store.write(record, &bytes)
}
