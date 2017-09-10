use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::Path;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use serde_json;

use media;

#[derive(Debug)]
pub enum LevelError {
    IoError(io::Error),
    SerdeError(serde_json::Error),
}

impl Display for LevelError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self, f)
    }
}

impl StdError for LevelError {
    fn description(&self) -> &str {
        "level error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            LevelError::SerdeError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for LevelError {
    fn from(err: serde_json::Error) -> LevelError {
        LevelError::SerdeError(From::from(err))
    }
}

impl From<io::Error> for LevelError {
    fn from(err: io::Error) -> LevelError {
        LevelError::IoError(From::from(err))
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub player_start_pos: (f32, f32),
}

impl Level {
    pub fn load(media_handle: &media::MediaHandle, path: &str) -> Result<Level, LevelError> {
        let mut full_path = media_handle.base_path.clone();
        full_path.push(&Path::new(path));

        let mut file = OpenOptions::new().read(true).open(full_path)?;

        let mut text = String::new();
        file.read_to_string(&mut text)?;
        let level: Level = serde_json::from_str(&text)?;

        println!("Loaded level `{}`", level.name);

        Ok(level)
    }

    pub fn save(&self, media_handle: &media::MediaHandle, path: &str) -> Result<(), LevelError> {
        println!("saving...");
        let mut full_path = media_handle.base_path.clone();
        full_path.push(&Path::new(path));

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(full_path)?;

        let text = serde_json::to_string(self)?;
        file.write_all(text.as_ref())?;

        println!("Saved level `{}`", self.name);

        Ok(())
    }
}
