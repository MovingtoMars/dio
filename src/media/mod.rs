pub mod image;

use std::path::PathBuf;

use gfx_device_gl::*;

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
