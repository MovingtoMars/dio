mod image;
pub use self::image::*;

use std::path::PathBuf;

use gfx_device_gl::*;
use piston_window::*;

pub struct MediaHandle {
    pub base_path: PathBuf,

    factory: Factory,
}

impl MediaHandle {
    pub fn new(factory: Factory) -> MediaHandle {
        MediaHandle {
            base_path: PathBuf::from("media/"),
            factory: factory,
        }
    }

    pub fn factory_clone(&self) -> Factory {
        self.factory.clone()
    }
}


pub struct FontHandle {
    pub glyphs: Glyphs,
}

impl FontHandle {
    pub fn new(media: &MediaHandle, font_path: &str) -> Self {
        let mut path = media.base_path.clone();
        path.push("fonts/");
        path.push(font_path);

        FontHandle {
            glyphs: Glyphs::new(path, media.factory.clone()).unwrap(),
        }
    }
}
