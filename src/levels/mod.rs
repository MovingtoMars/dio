use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use serde_json;

use nphysics::math::Vector;

use media;
use engine::*;

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
    pub player_start_pos: (N, N),
    pub entities: Vec<LevelEntity>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LevelVector {
    x: f32,
    y: f32,
}

impl LevelVector {
    pub fn to_vector(self) -> Vector<N> {
        Vector::new(self.x as N, self.y as N)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LevelEntity {
    Ground { rect: Rect },
    Crate { rect: Rect, material: CrateMaterial },
    Enemy { rect: Rect },
    Bullet {
        pos: LevelVector,
        radius: N,
        velocity: LevelVector,
    },
}

impl Level {
    pub fn to_world(&self) -> World {
        let (px, py) = self.player_start_pos;
        let mut world = World::new(px, py);

        for e in &self.entities {
            match *e {
                LevelEntity::Ground { rect } => {
                    world.new_ground(rect);
                }
                LevelEntity::Crate { rect, material } => {
                    world.new_crate(rect, material);
                }
                LevelEntity::Enemy { rect } => {
                    world.new_enemy(rect);
                }
                LevelEntity::Bullet {
                    pos,
                    radius,
                    velocity,
                } => {
                    world.new_bullet(pos.to_vector(), radius, velocity.to_vector());
                }
            }
        }

        world
    }

    pub fn load(media_handle: &media::MediaHandle, path: &str) -> Result<Level, LevelError> {
        let mut full_path = media_handle.base_path.clone();
        full_path.push("levels/");
        full_path.push(path);

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
        full_path.push("levels/");
        full_path.push(path);

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(full_path)?;

        let text = serde_json::to_string_pretty(self)?;
        writeln!(file, "{}", text)?;

        println!("Saved level `{}`", self.name);

        Ok(())
    }
}
