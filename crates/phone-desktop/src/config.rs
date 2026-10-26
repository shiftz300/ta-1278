//! Host capabilities, separate from the phone's own settings.
#[derive(Clone, Copy, Debug)]
pub struct Features {
    pub music: bool,
    pub files: bool,
    pub network: bool,
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
                "pictures" => self.pictures = false,
                "video" => self.video = false,
                "sounds" => self.sounds = false,
                _ => {
                    return Err(format!(
                        "Unknown feature: {name}. Expected music, files, network, pictures, video or sounds"
                    ));
                }
            }
        }
        Ok(())
    }
}
