//! Host capabilities, separate from the phone's own settings.
#[derive(Clone, Copy, Debug)]
pub struct Features {
    pub music: bool,
    pub files: bool,
    pub network: bool,
}
impl Default for Features {
    fn default() -> Self {
        Self {
            music: true,
            files: true,
            network: true,
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
                _ => {
                    return Err(format!(
                        "Unknown feature: {name}. Expected music, files or network"
                    ));
                }
            }
        }
        Ok(())
    }
}
