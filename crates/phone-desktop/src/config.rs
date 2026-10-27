//! Host capabilities, separate from the phone's own settings.
mod apps;
pub use apps::Apps;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Features {
    pub music: bool,
    pub files: bool,
    pub network: bool,
    pub javascript: bool,
    pub pictures: bool,
    pub video: bool,
    pub sounds: bool,
}
impl Default for Features {
    fn default() -> Self {
        Self {
            music: true,
            files: true,
            network: true,
            javascript: true,
            pictures: true,
            video: true,
            sounds: true,
        }
    }
}
impl Features {
    pub fn disable(&mut self, names: &str) -> Result<(), String> {
        for name in names.split(',') {
            match name {
                "music" => self.music = false,
                "files" => self.files = false,
                "network" => self.network = false,
                "javascript" => self.javascript = false,
                "pictures" => self.pictures = false,
                "video" => self.video = false,
                "sounds" => self.sounds = false,
                _ => {
                    return Err(format!(
                        "Unknown feature: {name}. Expected music, files, network, javascript, pictures, video or sounds"
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub directory: Option<PathBuf>,
    pub read_only: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuntimeConfig {
    pub version: u32,
    pub apps: Apps,
    pub capabilities: Features,
    pub storage: StorageConfig,
}
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            version: 1,
            apps: Default::default(),
            capabilities: Default::default(),
            storage: Default::default(),
        }
    }
}
impl RuntimeConfig {
    pub fn read(path: &Path, required: bool) -> Result<Self, String> {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if !required && e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(e) => return Err(format!("Cannot read config {}: {e}", path.display())),
        };
        let mut bytes = vec![];
        file.take(65537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65536 {
            return Err("Config exceeds 64 KiB".into());
        }
        let mut config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Invalid config {}: {e}", path.display()))?;
        if config.version != 1 {
            return Err(format!("Unsupported config version {}", config.version));
        }
        if let Some(dir) = &mut config.storage.directory {
            if dir.as_os_str().is_empty() {
                return Err("Storage directory must not be empty".into());
            }
            if dir.is_relative() {
                *dir = path.parent().unwrap_or(Path::new(".")).join(&*dir);
            }
        }
        Ok(config)
    }
    pub fn effective_features(&self) -> Features {
        let mut f = self.capabilities;
        f.music &= self.apps.music;
        f.network &= self.apps.browser;
        f.pictures &= self.apps.pictures;
        f.video &= self.apps.video;
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_defaults_keep_apps_and_unknown_or_core_app_flags_are_rejected() {
        let c: RuntimeConfig = serde_json::from_str("{}").unwrap();
        assert!(c.apps.browser && c.apps.music && c.apps.alarms && c.apps.utilities);
        assert!(c.capabilities.javascript);
        let mut restricted = c.capabilities;
        restricted.disable("javascript").unwrap();
        assert!(!restricted.javascript && restricted.network);
        for bad in [
            r#"{"apps":{"messages":false}}"#,
            r#"{"apps":{"broser":false}}"#,
            r#"{"capabilities":{"snd":false}}"#,
            r#"{"extra":1}"#,
        ] {
            assert!(serde_json::from_str::<RuntimeConfig>(bad).is_err());
        }
        let c: RuntimeConfig = serde_json::from_str(
            r#"{"apps":{"music":false,"browser":false},"capabilities":{"sounds":false}}"#,
        )
        .unwrap();
        let f = c.effective_features();
        assert!(!f.music && !f.network && !f.sounds && f.files);
    }
    #[test]
    fn config_file_is_bounded_resolves_relative_storage_and_never_rewrites_errors() {
        let root = std::env::temp_dir().join(format!("ta1278-config-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("config.json");
        assert!(RuntimeConfig::read(&root.join("missing"), false).is_ok());
        assert!(RuntimeConfig::read(&root.join("missing"), true).is_err());
        std::fs::write(
            &path,
            r#"{"storage":{"directory":"phone","read_only":true}}"#,
        )
        .unwrap();
        let c = RuntimeConfig::read(&path, true).unwrap();
        assert_eq!(c.storage.directory, Some(root.join("phone")));
        assert!(c.storage.read_only);
        for content in [
            r#"{"version":2}"#.to_owned(),
            " ".repeat(65537),
            r#"{"storage":{"directory":""}}"#.into(),
            "{".into(),
        ] {
            std::fs::write(&path, &content).unwrap();
            assert!(RuntimeConfig::read(&path, true).is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
