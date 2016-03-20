use std::fmt;
use std::ops;
use std::cell::{RefCell, RefMut, Ref};
use std::rc::Rc;

use physics::body::Body;

pub struct BodyHandle<T> {
    body: Rc<RefCell<Body<T>>>,
}

impl<T> Clone for BodyHandle<T> {
    fn clone(&self) -> Self {
        BodyHandle{body: self.body.clone()}
    }
}

pub struct World<T> {
    pub gravity: Vec2,

    bodies: Vec<BodyHandle<T>>,

    collision_callback: Option<fn(&mut Body<T>, &mut Body<T>)>,
}

impl<T> World<T> {
    pub fn new(gravity: Vec2) -> World<T> {
        World{
            gravity: gravity,
            bodies: Vec::new(),
            collision_callback: None,
        }
    }

    pub fn set_collision_callback(&mut self, callback: Option<fn(&mut Body<T>, &mut Body<T>)>) {
        self.collision_callback = callback;
    }

    pub fn add_body(&mut self, body: Body<T>) -> BodyHandle<T> {
        let handle = BodyHandle{body: Rc::new(RefCell::new(body))};
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

    }
}

#[derive(Clone,Copy,Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Vec2 {
        Vec2{x: x, y: y}
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn norm(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn mul(self, scalar: f64) -> Vec2 {
        Vec2{x: self.x * scalar, y: self.y * scalar}
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
        Vec2{x: self.x + other.x, y: self.y + other.y}
    }
}


impl ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Self) -> Self {
        Vec2{x: self.x - other.x, y: self.y - other.y}
    }
}
