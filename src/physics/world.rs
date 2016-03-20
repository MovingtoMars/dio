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

    pub fn set_collision_callback(&mut self, callback: Option<fn(body::Collision, &mut Body<T>, &mut Body<T>)>) {
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
                let (mut a, mut b) = self.bodies.split_at_mut(i + 1);
                let alen = a.len();
                let h1 = &mut a[alen - 1];
                let h2 = &mut b[0];

                use std::borrow::BorrowMut;
                let mut b1 = (*h1.body).borrow_mut();
                let mut b2 = (*h2.body).borrow_mut();

                let collision = check_body_collision(&mut *b1, &mut *b2);
                match collision {
                    Some(c) => {
                        match self.collision_callback {
                            Some(func) => {
                                func(c, &mut *b1, &mut *b2);
                            },
                            None => {},
                        }
                    },
                    None => {},
                }
            }
        }
    }
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
