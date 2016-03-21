extern crate gfx_graphics;
extern crate gfx_device_gl;

use std::path::PathBuf;

use self::gfx_graphics::*;
use self::gfx_device_gl::Resources;

use media::MediaHandle;

pub struct Image {
    path: PathBuf,
    texture: Texture<Resources>,
}

impl Image {
    pub fn new(handle: &MediaHandle, image_path: &str) -> Result<Image, String> {
        let mut path = handle.base_path.clone();
        path.push(image_path);

        let tex = try!(Texture::from_path(&mut *handle.borrow_factory_mut(),
                                          path.as_path(),
                                          Flip::None,
                                          &TextureSettings::new()));

        Ok(Image {
            path: path,
            texture: tex,
        })
    }
}
