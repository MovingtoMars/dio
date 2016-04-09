use std::path::PathBuf;

use piston_window::*;
use gfx_device_gl::Resources;

use media::MediaHandle;

pub struct ImageHandle {
    path: PathBuf,
    texture: Texture<Resources>,
}

impl ImageHandle {
    pub fn new(handle: &MediaHandle, image_path: &str) -> Result<ImageHandle, String> {
        let mut path = handle.base_path.clone();
        path.push(image_path);

        let tex = try!(Texture::from_path(&mut *handle.borrow_factory_mut(),
                                          path.as_path(),
                                          Flip::None,
                                          &TextureSettings::new()));

        Ok(ImageHandle {
            path: path,
            texture: tex,
        })
    }

    pub fn borrow_texture(&self) -> &Texture<Resources> {
        &self.texture
    }
}
