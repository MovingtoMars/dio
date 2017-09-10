// pub mod world;
// pub mod entity;

mod world;
pub use self::world::*;

mod component;
pub use self::component::*;

mod system;
pub use self::system::*;

mod physics;
pub use self::physics::*;


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Rect {
    x: N,
    y: N,
    hw: N,
    hh: N,
}

impl Rect {
    pub fn new(x: N, y: N, hw: N, hh: N) -> Self {
        Rect { x, y, hw, hh }
    }
}
