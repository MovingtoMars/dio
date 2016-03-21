extern crate gfx_device_gl;

pub mod image;

use std::path::PathBuf;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

use self::gfx_device_gl::*;

pub struct MediaHandle {
    pub base_path: PathBuf,

    factory: Rc<RefCell<Factory>>,
}

impl MediaHandle {
    pub fn new(factory: Rc<RefCell<Factory>>) -> MediaHandle {
        MediaHandle {
            base_path: PathBuf::from("media/"),
            factory: factory,
        }
    }

    pub fn borrow_factory_mut(&self) -> RefMut<Factory> {
        (*self.factory).borrow_mut()
    }
}
