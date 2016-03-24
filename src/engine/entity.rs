extern crate piston_window;

use std::rc::Rc;
use std::cell::RefCell;
use std::boxed::Box;

use physics;

use piston_window::*;
use engine::world::*;
use interface::camera::Camera;
use render;

// Note that the following conventions are used with entities:
// x, y: the *centre* position of the entity
// hw, hh: the half-width and half-height of the entity

pub type PhysicsWorld = physics::world::World<Rc<RefCell<Box<Entity>>>>;

pub trait Entity {
    fn get_body_handle(&mut self) -> &physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>>;
    fn get_centre(&self, physics_world: &PhysicsWorld) -> (f64, f64);
    fn get_bounding_box(&self, physics_world: &PhysicsWorld) -> (f64, f64, f64, f64);

    fn render(&self, physics_world: &PhysicsWorld, win: &PistonWindow, cam: &Camera);
    fn update(&mut self, world: &mut WorldData, dt: f64);

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::None
    }
}

pub struct Ground {
    body_handle: physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>>,
    hw: f64,
    hh: f64,
}

impl Ground {
    pub fn new(world_data: &mut WorldData, x: f64, y: f64, hw: f64, hh: f64) -> Ground {
        // TODO static

        let def = physics::body::BodyDef::new(physics::body::BodyType::Static);
        let shape = physics::shape::Rect::new(hw, hh);
        let mut body = physics::body::Body::new(Box::new(shape), def);
        body.pos = physics::world::Vec2 { x: x, y: y };
        let handle = world_data.physics_world.add_body(body);

        Ground {
            body_handle: handle,
            hw: hw,
            hh: hh,
        }
    }
}

impl Entity for Ground {
    fn render(&self, physics_world: &PhysicsWorld, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box(physics_world);
        render::fill_rectangle(win, cam, [0.0, 1.0, 0.0, 1.0], x, y, w, h);
    }

    fn get_body_handle(&mut self) -> &physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>> {
        &mut self.body_handle
    }

    fn get_centre(&self, physics_world: &PhysicsWorld) -> (f64, f64) {
        let trans = physics_world.get_body(&self.body_handle).pos;
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self, physics_world: &PhysicsWorld) -> (f64, f64, f64, f64) {
        let (cx, cy) = self.get_centre(physics_world);
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, _: &mut WorldData, _: f64) {}
}

#[derive(Clone,Copy)]
pub enum CrateMaterial {
    Steel,
    Wood,
}

impl CrateMaterial {
    pub fn density(self) -> f64 {
        match self {
            CrateMaterial::Steel => 8000.0,
            CrateMaterial::Wood => 700.0,
        }
    }

    pub fn restitution(self) -> f64 {
        match self {
            CrateMaterial::Steel => 0.6,
            CrateMaterial::Wood => 0.4,
        }
    }
}

pub struct Crate {
    body_handle: physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>>,
    hw: f64,
    hh: f64,
    material: CrateMaterial,
}

impl Crate {
    pub fn new(world_data: &mut WorldData, mat: CrateMaterial, x: f64, y: f64, hw: f64, hh: f64) -> Crate {
        let mut def = physics::body::BodyDef::new(physics::body::BodyType::Dynamic);
        def.density = mat.density();
        def.restitution = mat.restitution();
        let shape = physics::shape::Rect::new(hw, hh);
        let mut body = physics::body::Body::new(Box::new(shape), def);
        body.pos = physics::world::Vec2 { x: x, y: y };
        let handle = world_data.physics_world.add_body(body);

        Crate {
            body_handle: handle,
            hw: hw,
            hh: hh,
            material: mat,
        }
    }
}

impl Entity for Crate {
    fn render(&self, physics_world: &PhysicsWorld, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box(physics_world);

        let (c1, c2) = match self.material {
            CrateMaterial::Steel => ([0.2, 0.2, 0.2, 1.0], [0.3, 0.3, 0.3, 1.0]),
            CrateMaterial::Wood => ([0.4, 0.2, 0.0, 1.0], [0.6, 0.3, 0.0, 1.0]),
        };

        render::fill_rectangle(win, cam, c1, x, y, w, h);
        render::fill_rectangle(win, cam, c2, x + w * 0.1, y + h * 0.1, w * 0.8, h * 0.8);
    }

    fn get_body_handle(&mut self) -> &physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>> {
        &mut self.body_handle
    }

    fn get_centre(&self, physics_world: &PhysicsWorld) -> (f64, f64) {
        let trans = physics_world.get_body(&self.body_handle).pos;
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self, physics_world: &PhysicsWorld) -> (f64, f64, f64, f64) {
        let (cx, cy) = self.get_centre(physics_world);
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, world_data: &mut WorldData, _: f64) {}
}

pub struct Player {
    body_handle: physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>>,
    hw: f64,
    hh: f64,

    moving_right: bool,
    moving_left: bool,
}

impl Player {
    pub fn new(world_data: &mut WorldData, x: f64, y: f64, hw: f64, hh: f64) -> Player {
        let mut def = physics::body::BodyDef::new(physics::body::BodyType::Dynamic);
        def.density = 1000.0;
        let shape = physics::shape::Rect::new(hw, hh);
        let mut body = physics::body::Body::new(Box::new(shape), def);
        body.pos = physics::world::Vec2 { x: x, y: y };
        let handle = world_data.physics_world.add_body(body);

        Player {
            body_handle: handle,
            hw: hw,
            hh: hh,
            moving_right: false,
            moving_left: false,
        }
    }

    pub fn set_moving_right(&mut self, moving: bool) {
        self.moving_right = moving;
    }

    pub fn set_moving_left(&mut self, moving: bool) {
        self.moving_left = moving;
    }
}

const USAIN_BOLT_MAX_SPEED: f64 = 12.4;
const PLAYER_MAX_SPEED: f64 = USAIN_BOLT_MAX_SPEED * 0.5;
const PLAYER_ACCELERATION: f64 = 1.5;

impl Entity for Player {
    fn render(&self, physics_world: &PhysicsWorld, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box(physics_world);
        render::fill_rectangle(win, cam, [1.0, 0.8, 0.1, 1.0], x, y, w, h);
    }

    fn get_body_handle(&mut self) -> &physics::world::BodyHandle<Rc<RefCell<Box<Entity>>>> {
        &mut self.body_handle
    }

    fn get_centre(&self, physics_world: &PhysicsWorld) -> (f64, f64) {
        let trans = physics_world.get_body(&self.body_handle).pos;
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self, physics_world: &PhysicsWorld) -> (f64, f64, f64, f64) {
        let (cx, cy) = self.get_centre(physics_world);
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, world_data: &mut WorldData, _: f64) {
        let mut body = world_data.physics_world.get_body_mut(&self.body_handle);

        let mut vel = body.vel;

        let touching_ground = true; // TODO

        if touching_ground {
            if self.moving_right == self.moving_left {
                let neg = vel.x < 0.0;
                vel.x = (vel.x.abs() - PLAYER_ACCELERATION).max(0.0);
                if neg {
                    vel.x = -vel.x;
                }
            } else {
                if self.moving_left {
                    vel.x = (vel.x - PLAYER_ACCELERATION).max(-PLAYER_MAX_SPEED);
                } else if self.moving_right {
                    vel.x = (vel.x + PLAYER_ACCELERATION).min(PLAYER_MAX_SPEED);
                }

            }
        }

        body.vel = vel;
    }

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::Some(self)
    }
}
