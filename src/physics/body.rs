use physics::shape;
use physics::world;

#[derive(Clone,Copy)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Clone,Copy)]
pub struct BodyDef {
    pub density: f64,
    pub body_type: BodyType,
}

impl BodyDef {
    pub fn new(body_type: BodyType) -> BodyDef {
        BodyDef{
            density: 0.0,
            body_type: body_type,
        }
    }
}

pub struct Body<T> {
    pub user_data: Option<T>,

    /// you probably don't want to change these two directly
    pub vel: world::Vec2,
    pub pos: world::Vec2,

    def: BodyDef,
    shape: Box<shape::Shape>,
}

impl<T> Body<T> {
    pub fn new(shape: Box<shape::Shape>, def: BodyDef) -> Body<T> {
        Body{
            user_data: None,
            def: def,
            shape: shape,
            vel: world::Vec2::default(),
            pos: world::Vec2::default(),
        }
    }
}

/// This function is called every time World updates. Note that this function will be called a maximum of one time for every possible pair of bodies, on each iteration.
pub fn check_body_collision<T, U>(b1: &mut Body<T>, b2: &mut Body<U>) -> bool {
    let _ = (b1, b2);
    false
}
