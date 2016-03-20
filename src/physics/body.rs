use physics::shape;
use physics::world::{self, Vec2};

#[derive(Clone,Copy)]
pub struct Collision {
    pub point_a: Vec2,
    pub point_b: Vec2,
    pub normal_a: Vec2,
    pub normal_b: Vec2,
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Clone,Copy)]
pub struct BodyDef {
    pub density: f64,
    pub body_type: BodyType,
    pub restitution: f64,
}

impl BodyDef {
    pub fn new(body_type: BodyType) -> BodyDef {
        BodyDef {
            density: 0.0,
            body_type: body_type,
            restitution: 1.0, // was 0.0 before i fiddled with it
        }
    }
}

pub struct Body<T> {
    pub user_data: Option<T>,

    /// you probably don't want to change these two directly
    pub vel: Vec2,
    pub pos: Vec2,

    pub def: BodyDef,
    shape: Box<shape::Shape>,

    applied_forces: Vec<Vec2>,
    applied_impulses: Vec<Vec2>,
}

impl<T> Body<T> {
    pub fn new(shape: Box<shape::Shape>, def: BodyDef) -> Body<T> {
        Body {
            user_data: None,
            def: def,
            shape: shape,
            vel: Vec2::default(),
            pos: Vec2::default(),
            applied_forces: Vec::new(),
            applied_impulses: Vec::new(),
        }
    }

    pub fn body_def(&self) -> BodyDef {
        self.def
    }

    pub fn update(&mut self, dt: f64) {
        let mass = self.mass();

        let mut vel = self.vel;

        for force in &mut self.applied_forces {
            // a = F/m
            let a = force.mul(1.0 / mass);
            // v = at
            vel = vel + a.mul(dt);
        }

        for impulse in &mut self.applied_impulses {
            vel = vel + impulse.mul(1.0 / mass);
        }

        self.vel = vel;

        if self.def.body_type != BodyType::Static {
            self.pos = self.pos + self.vel.mul(dt);
        }

        self.applied_forces.clear();
        self.applied_impulses.clear(); // you forgot to put this in you dummy
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.applied_forces.push(force);
    }

    pub fn apply_impulse(&mut self, impulse: Vec2) {
        self.applied_impulses.push(impulse);
    }

    pub fn momentum(&self) -> Vec2 {
        self.vel.mul(self.mass())
    }

    pub fn borrow_shape(&self) -> &shape::Shape {
        &*self.shape
    }

    pub fn mass(&self) -> f64 {
        self.shape.mass(self.def.density)
    }

    pub fn restitution(&self) -> f64 {
        self.def.restitution
    }
}
