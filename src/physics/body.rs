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
    /// Static bodies cannot move. Applying forces/impulses to them will have no effect.
    Static,
    /// Kinematic bodies can move, but they are not affected by forces.
    /// The only way to change their motion is to manually alter the velocity.
    Kinematic,
    /// Dynamic bodies are affected by forces.
    Dynamic,
}

#[derive(Clone,Copy)]
pub struct BodyDef {
    pub density: f64,
    pub body_type: BodyType,
    pub restitution: f64,
    pub friction: f64,
}

impl BodyDef {
    pub fn new(body_type: BodyType) -> BodyDef {
        BodyDef {
            density: 1.0,
            body_type: body_type,
            restitution: 0.5,
            friction: 0.6,
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
    applied_friction: Vec<Vec2>,

    prev_net_force: Vec2, // TODO: when sleeping is implemented, make sure to set this to 0
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
            applied_friction: Vec::new(),
            prev_net_force: Vec2::new(0.0, 0.0),
        }
    }

    pub fn body_def(&self) -> BodyDef {
        self.def
    }

    pub fn update(&mut self, dt: f64) {
        let mass = self.mass();

        if self.def.body_type == BodyType::Dynamic {
            let mut net_force = self.current_net_force(dt);
            let required_impulse = self.momentum();
            for f in &mut self.applied_friction {
                let component_momentum = required_impulse.orthogonalise(*f);
                let unit_component_momentum = component_momentum.unit();
                let resultant_norm = f.mul(1.0 / dt).norm().min(required_impulse.norm());
                net_force = net_force - unit_component_momentum.mul(resultant_norm);
            }

            let a = net_force.mul(1.0 / mass);
            self.vel = self.vel + a.mul(dt);

            self.prev_net_force = net_force;
        }

        if self.def.body_type != BodyType::Static {
            self.pos = self.pos + self.vel.mul(dt);
        }

        self.applied_forces.clear();
        self.applied_impulses.clear();
        self.applied_friction.clear();
    }

    pub fn current_net_force(&mut self, dt: f64) -> Vec2 {
        let mut net_force = Vec2::new(0.0, 0.0);

        for force in &mut self.applied_forces {
            net_force = net_force + *force;
        }

        for impulse in &mut self.applied_impulses {
            net_force = net_force + impulse.mul(1.0 / dt);
        }

        net_force
    }

    pub fn prev_net_force(&self) -> Vec2 {
        self.prev_net_force
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.applied_forces.push(force);
    }

    pub fn apply_impulse(&mut self, impulse: Vec2) {
        self.applied_impulses.push(impulse);
    }

    pub fn apply_friction(&mut self, friction: Vec2) {
        self.applied_friction.push(friction);
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
    pub fn is_static(&self) -> bool {
        self.def.body_type!=BodyType::Dynamic
    }
}
