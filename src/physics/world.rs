
use std::fmt;
use std::ops;
use std::cell::{RefCell, RefMut, Ref};
use std::rc::Rc;

use physics;
use physics::body::{self, Body};

pub const COLLISION_BUFFER: f64 = 0.005;
pub const VELOCITY_THRESHOLD: f64 = 1.0; // collisions below this are treated inelastically

pub struct BodyHandle<T> {
    body: Rc<RefCell<Body<T>>>,
}

impl<T> Clone for BodyHandle<T> {
    fn clone(&self) -> Self {
        BodyHandle { body: self.body.clone() }
    }
}

pub struct World<T> {
    pub gravity: Vec2,

    bodies: Vec<BodyHandle<T>>,

    collision_callback: Option<fn(body::Collision, &mut Body<T>, &mut Body<T>)>,
}

impl<T> World<T> {
    pub fn new(gravity: Vec2) -> World<T> {
        World {
            gravity: gravity,
            bodies: Vec::new(),
            collision_callback: None,
        }
    }

    pub fn set_collision_callback(&mut self,
                                  callback: Option<fn(body::Collision, &mut Body<T>, &mut Body<T>)>) {
        self.collision_callback = callback;
    }

    pub fn add_body(&mut self, body: Body<T>) -> BodyHandle<T> {
        let handle = BodyHandle { body: Rc::new(RefCell::new(body)) };
        self.bodies.push(handle.clone());
        handle
    }

    pub fn get_body<'a>(&'a self, handle: &'a BodyHandle<T>) -> Ref<Body<T>> {
        handle.body.borrow()
    }

    pub fn get_body_mut<'a>(&'a self, handle: &'a BodyHandle<T>) -> RefMut<Body<T>> {
        handle.body.borrow_mut()
    }

    pub fn update(&mut self, dt: f64) {
        for body in &mut self.bodies {
            let mut body = body.body.borrow_mut();
            if body.body_def().body_type == physics::body::BodyType::Dynamic {
                body.apply_force(self.gravity);
            }
            body.update(dt);
        }

        if self.bodies.len() > 1 {
            for i in 0..(self.bodies.len() - 1) {
                use std::borrow::BorrowMut;
                let (mut a, b) = self.bodies.split_at_mut(i + 1);
                let alen = a.len();

                let h1 = &mut a[alen - 1];
                for h2 in &mut b.iter() {
                    let mut b1 = (*h1.body).borrow_mut();
                    let mut b2 = (*h2.body).borrow_mut();

                    let collision = check_body_collision(&mut *b1, &mut *b2);
                    match collision {
                        Some(c) => {
                            solve_collision(c, &mut *b1, &mut *b2);

                            match self.collision_callback {
                                Some(func) => {
                                    func(c, &mut *b1, &mut *b2);
                                }
                                None => {}
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

fn get_collision_impulses<T, U>(collision: body::Collision,
                                b1: &mut Body<T>,
                                b2: &mut Body<U>)
                                -> (Vec2, Vec2) {
    let _ = collision;

    let relative_speed = (b1.vel - b2.vel).norm();

    let mut collision_restitution = (b1.restitution() + b2.restitution()) / 2.0;
    if relative_speed<VELOCITY_THRESHOLD {
        collision_restitution = 0.0;
    }

    let mut momentum1 = b1.momentum();
    let mut momentum2 = b2.momentum();

    let total_momentum: Vec2;
    if !b1.is_static() && !b2.is_static() {
        total_momentum = momentum1 + momentum2;
    } else {
        total_momentum = Vec2::new(0.0, 0.0);
        if b1.is_static() {
            momentum1 = Vec2::new(0.0, 0.0);
        }
        if b2.is_static() {
            momentum2 = Vec2::new(0.0, 0.0);
        }
    }

    let final_velocity = total_momentum.mul(1.0 / (b1.mass() + b2.mass()));

    let deformation_impulse1 = (final_velocity.mul(b1.mass()) - momentum1).projection_onto(collision.normal_a);
    let deformation_impulse2 = (final_velocity.mul(b2.mass()) - momentum2).projection_onto(collision.normal_a);
    let restoration_impulse1 = deformation_impulse1.mul(collision_restitution);
    let restoration_impulse2 = deformation_impulse2.mul(collision_restitution);

    (deformation_impulse1 + restoration_impulse1,
     deformation_impulse2 + restoration_impulse2)
}

fn apply_collision_position_correction<T, U>(collision: body::Collision,
                                             b1: &mut Body<T>,
                                             b2: &mut Body<U>) {
    let (x1, y1, x2, y2) = b1.borrow_shape().bounds(b1.pos); // we are restricted to Rect shapes
    let (x3, y3, x4, y4) = b2.borrow_shape().bounds(b2.pos);
    let _ = (x1, x2, x3, x4, y1, y2, y3, y4);
    let center_point = collision.point_a;
    let correction_vector = collision.normal_a;
    let penetration_vector = center_point - collision.corner_a;
    let displacement = correction_vector.scale_to(penetration_vector);
        if !b1.is_static() && !b2.is_static() {
            b1.pos = b1.pos + displacement;
            b2.pos = b2.pos - displacement;
        } else {
            if b2.is_static() && !b1.is_static() {
                b1.pos = b1.pos + displacement.mul(2.0);
            }
            if b1.is_static() && !b2.is_static() {
                b2.pos = b2.pos - displacement.mul(2.0);
            }
        }
}

fn solve_collision<T, U>(collision: body::Collision, b1: &mut Body<T>, b2: &mut Body<U>) {
    let (impulse1, impulse2) = get_collision_impulses(collision, b1, b2);

    apply_collision_position_correction(collision, b1, b2);

    b1.apply_impulse(impulse1);
    b2.apply_impulse(impulse2);
}

/// This function is called every time World updates. Note that this function will be called a maximum of one time for every possible pair of bodies, on each iteration.
fn check_body_collision<T, U>(b1: &mut Body<T>, b2: &mut Body<U>) -> Option<body::Collision> {
    b1.borrow_shape().collides_with(b1.pos, b2.borrow_shape(), b2.pos)
}

#[derive(Clone,Copy,Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x: x, y: y }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn norm(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn mul(self, scalar: f64) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    pub fn coords(self) -> (f64, f64) {
        (self.x, self.y)
    }

    pub fn abs(self) -> Vec2 {
        Vec2 {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    pub fn align_quadrant(self, align: Vec2) ->  Vec2 {
        let reduced_x = align.x/align.x.abs();
        let reduced_y = align.y/align.y.abs();
        Vec2 {
            x: self.x*reduced_x,
            y: self.y*reduced_y,
        }
    }

    pub fn unit(self) -> Vec2 {
        self.mul(1.0/self.norm())
    }

    pub fn scale_to(self, vector: Vec2) -> Vec2 {
        let aligned_vector = vector.align_quadrant(self);
        let scale_x = aligned_vector.x/self.x;
        let scale_y = aligned_vector.y/self.y;
        let scale = scale_x.min(scale_y);
        self.mul(scale)
    }

    pub fn projection_onto(self, vector: Vec2) -> Vec2 {
        vector.mul(self.dot(vector)/vector.norm())
    }

    pub fn orthogonalise(self, vector: Vec2) -> Vec2 {
        self - self.projection_onto(vector)
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Self) -> Self {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}


impl ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Self) -> Self {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
