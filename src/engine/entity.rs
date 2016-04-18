use std::ops::Deref;
use std::boxed::Box;
use std::sync::Arc;

use piston_window::*;
use engine::world::*;
use interface::camera::Camera;
use render;

use nalgebra::{self, Rotation, Rot2, Vec1, RotationTo, Norm, Mat1};
use ncollide_entities::shape::Cuboid;
use ncollide_math::Scalar;
use nphysics;
use nphysics::math::{Vector, Orientation, Point};
use nphysics::object::{RigidBody, RigidBodyHandle};
use num::Zero;

// NOTE: For all objects that have their velocities changes manually, make sure to turn off the deactivation threshold.

// Note that the following conventions are used with entities:
// x, y: the *centre* position of the entity
// hw, hh: the half-width and half-height of the entity

const BODY_MARGIN: f32 = 0.04;

pub struct TimeRigidBodyHandle<T: Scalar> {
    handle: RigidBodyHandle<T>,

    pub saved_lin_vel: Option<Vector<T>>,
    pub saved_ang_vel: Option<Orientation<T>>,
}

impl<T: Scalar> TimeRigidBodyHandle<T> {
    fn new(handle: RigidBodyHandle<T>) -> TimeRigidBodyHandle<T> {
        TimeRigidBodyHandle {
            handle: handle,
            saved_lin_vel: None,
            saved_ang_vel: None,
        }
    }

    // called when time is stopped to gradually slow down body
    pub fn update_saved_vel(&mut self, dt: f32) {
        assert!(self.saved_ang_vel.is_none() == self.saved_lin_vel.is_none());

        // use zero values if this body was created during time stop
        let saved_lin_vel = self.saved_lin_vel.unwrap_or(Vector::zero());
        let saved_ang_vel = self.saved_ang_vel.unwrap_or(Orientation::zero());

        let mut handle = self.handle.borrow_mut();

        let init_lin_vel = handle.lin_vel();
        let init_ang_vel = handle.ang_vel();

        let ratio: T = nalgebra::cast(0.01f64.powf(dt as f64));
        let new_lin_vel = init_lin_vel * ratio;
        let new_ang_vel = init_ang_vel * ratio;

        // TODO +=
        self.saved_lin_vel = Some(saved_lin_vel + init_lin_vel - new_lin_vel);
        self.saved_ang_vel = Some(saved_ang_vel + init_ang_vel - new_ang_vel);

        handle.set_lin_vel(new_lin_vel);
        handle.set_ang_vel(new_ang_vel);
    }

    pub fn save_vel(&mut self) {
        assert!(self.saved_lin_vel.is_none());
        assert!(self.saved_ang_vel.is_none());

        let mut handle = self.handle.borrow_mut();

        self.saved_lin_vel = Some(handle.lin_vel());
        self.saved_ang_vel = Some(handle.ang_vel());

        handle.set_lin_vel(Vector::zero());
        handle.set_ang_vel(Orientation::zero());
    }

    pub fn restore_vel(&mut self) {
        assert!(self.saved_ang_vel.is_none() == self.saved_lin_vel.is_none());

        // use zero values if this body was created during time stop
        let saved_lin_vel = self.saved_lin_vel.unwrap_or(Vector::zero());
        let saved_ang_vel = self.saved_ang_vel.unwrap_or(Orientation::zero());

        let mut handle = self.handle.borrow_mut();

        let cur_lin_vel = handle.lin_vel();
        let cur_ang_vel = handle.ang_vel();

        if self.saved_lin_vel.is_some() && !handle.is_active() {
            handle.activate(nalgebra::Bounded::max_value());
        }
        handle.set_lin_vel(cur_lin_vel + saved_lin_vel);
        handle.set_ang_vel(cur_ang_vel + saved_ang_vel);

        self.saved_lin_vel = None;
        self.saved_ang_vel = None;
    }
}

impl<T: Scalar> Deref for TimeRigidBodyHandle<T> {
    type Target = RigidBodyHandle<T>;

    fn deref(&self) -> &RigidBodyHandle<T> {
        &self.handle
    }
}

// TODO: get_aabb() and change get_bounding_box()
pub trait Entity {
    fn get_body_handle_mut(&mut self) -> &mut TimeRigidBodyHandle<f32>;
    fn get_body_handle(&self) -> &TimeRigidBodyHandle<f32>;
    fn get_bounding_box(&self) -> (f32, f32, f32, f32);

    fn render(&self, physics_world: &nphysics::world::World<f32>, win: &PistonWindow, cam: &Camera);
    fn update(&mut self, world: &mut WorldData, dt: f32);

    fn pre_update(&mut self, world_data: &mut WorldData) {}
    fn on_stop_time(&mut self, world: &mut WorldData) {}
    fn on_start_time(&mut self, world: &mut WorldData) {}

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::None
    }

    fn get_rotation(&self) -> f32 {
        self.get_body_handle().borrow().position().rotation.rotation().x
    }

    fn get_centre(&self) -> (f32, f32) {
        let trans = self.get_body_handle().borrow().position().translation;
        (trans.x, trans.y)
    }
}

pub struct Ground {
    body_handle: TimeRigidBodyHandle<f32>,
    hw: f32,
    hh: f32,
}

impl Ground {
    pub fn new(world_data: &mut WorldData, x: f32, y: f32, hw: f32, hh: f32) -> Ground {
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new_static(shape, 0.2, 0.3);
        body.set_margin(BODY_MARGIN);
        body.append_translation(&Vector::new(x, y));
        let handle = world_data.physics_world.add_body(body);

        Ground {
            body_handle: TimeRigidBodyHandle::new(handle),
            hw: hw,
            hh: hh,
        }
    }
}

impl Entity for Ground {
    fn render(&self, physics_world: &nphysics::world::World<f32>, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();
        render::fill_rectangle(win,
                               cam,
                               [0.0, 1.0, 0.0, 1.0],
                               x,
                               y,
                               w,
                               h,
                               self.get_rotation());
    }

    fn get_body_handle_mut(&mut self) -> &mut TimeRigidBodyHandle<f32> {
        &mut self.body_handle
    }

    fn get_body_handle(&self) -> &TimeRigidBodyHandle<f32> {
        &self.body_handle
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre();
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, _: &mut WorldData, _: f32) {}
}

#[derive(Clone,Copy)]
pub enum CrateMaterial {
    Steel,
    Wood,
}

impl CrateMaterial {
    pub fn density(self) -> f32 {
        match self {
            CrateMaterial::Steel => 8000.0,
            CrateMaterial::Wood => 7000.0,
        }
    }

    pub fn restitution(self) -> f32 {
        match self {
            CrateMaterial::Steel => 0.6,
            CrateMaterial::Wood => 0.4,
        }
    }
}

pub struct Crate {
    body_handle: TimeRigidBodyHandle<f32>,
    hw: f32,
    hh: f32,
    material: CrateMaterial,
}

impl Crate {
    pub fn new(world_data: &mut WorldData, mat: CrateMaterial, x: f32, y: f32, hw: f32, hh: f32) -> Crate {
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new_dynamic(shape, mat.density(), mat.restitution(), 0.6);
        body.set_margin(BODY_MARGIN);
        body.append_translation(&Vector::new(x, y));

        let handle = world_data.physics_world.add_body(body);

        Crate {
            body_handle: TimeRigidBodyHandle::new(handle),
            hw: hw,
            hh: hh,
            material: mat,
        }
    }
}

impl Entity for Crate {
    fn render(&self, physics_world: &nphysics::world::World<f32>, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();

        let (c1, c2) = match self.material {
            CrateMaterial::Steel => ([0.2, 0.2, 0.2, 1.0], [0.3, 0.3, 0.3, 1.0]),
            CrateMaterial::Wood => ([0.4, 0.2, 0.0, 1.0], [0.6, 0.3, 0.0, 1.0]),
        };

        render::fill_rectangle(win, cam, c1, x, y, w, h, self.get_rotation());
        render::fill_rectangle(win,
                               cam,
                               c2,
                               x + w * 0.1,
                               y + h * 0.1,
                               w * 0.8,
                               h * 0.8,
                               self.get_rotation());
    }

    fn get_body_handle_mut(&mut self) -> &mut TimeRigidBodyHandle<f32> {
        &mut self.body_handle
    }

    fn get_body_handle(&self) -> &TimeRigidBodyHandle<f32> {
        &self.body_handle
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre();
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, world_data: &mut WorldData, _: f32) {}
}

pub struct Player {
    body_handle: TimeRigidBodyHandle<f32>,
    hw: f32,
    hh: f32,

    moving_right: bool,
    moving_left: bool,
    pub touching_ground: bool,
    pub release_jump: bool,
}

impl Player {
    pub fn new(world_data: &mut WorldData, x: f32, y: f32, hw: f32, hh: f32) -> Player {
        let density = 500.0;

        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new(Arc::new(Box::new(shape)),
                                      Some((density, Point::new(0.0, 0.0), Mat1::new(100000000000.0))),
                                      0.2,
                                      0.1);
        body.set_margin(BODY_MARGIN);
        body.append_translation(&Vector::new(x, y));
        body.set_deactivation_threshold(None);

        let handle = world_data.physics_world.add_body(body);

        Player {
            body_handle: TimeRigidBodyHandle::new(handle),
            hw: hw,
            hh: hh,
            moving_right: false,
            moving_left: false,
            touching_ground: false,
            release_jump: true,
        }
    }

    pub fn set_moving_right(&mut self, moving: bool) {
        self.moving_right = moving;
    }

    pub fn set_moving_left(&mut self, moving: bool) {
        self.moving_left = moving;
    }

    pub fn jump(&mut self, world_data: &mut WorldData) {
        let mut body = self.body_handle.borrow_mut();
        let mut lvel = body.lin_vel();
        lvel.y = -6.0;
        body.set_lin_vel(lvel);
        // body.on_ground = false;
    }

    pub fn release(&mut self, world_data: &mut WorldData) {
        let mut body = self.body_handle.borrow_mut();

        let mut lvel = body.lin_vel();

        if lvel.y < 0.0 && self.release_jump {
            lvel.y *= 0.45;
            body.set_lin_vel(lvel);
            self.release_jump = false;
        }
    }
}

const USAIN_BOLT_MAX_SPEED: f32 = 12.4;
const PLAYER_MAX_SPEED: f32 = USAIN_BOLT_MAX_SPEED * 0.5;
const PLAYER_ACCELERATION: f32 = PLAYER_MAX_SPEED * 2.5;

impl Entity for Player {
    fn render(&self, physics_world: &nphysics::world::World<f32>, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();
        render::fill_rectangle(win,
                               cam,
                               [1.0, 0.8, 0.1, 1.0],
                               x,
                               y,
                               w,
                               h,
                               self.get_rotation());
    }

    fn get_body_handle_mut(&mut self) -> &mut TimeRigidBodyHandle<f32> {
        &mut self.body_handle
    }

    fn get_body_handle(&self) -> &TimeRigidBodyHandle<f32> {
        &self.body_handle
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre();
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn pre_update(&mut self, world_data: &mut WorldData) {
        let mut body = self.body_handle.borrow_mut();
        body.clear_linear_force();
    }

    fn update(&mut self, world_data: &mut WorldData, dt: f32) {
        let mut body = self.body_handle.borrow_mut();

        let mut lvel = body.lin_vel();

        // if body.on_ground {
        self.touching_ground = true;
        self.release_jump = true;
        // }

        let mass = 1.0 / body.inv_mass();
        let lin_force = mass * PLAYER_ACCELERATION;

        // if self.touching_ground // why??????
        {
            if self.moving_right == self.moving_left {
                let neg = lvel.x < 0.0;
                lvel.x = (lvel.x.abs() - PLAYER_ACCELERATION * dt).max(0.0);
                if neg {
                    lvel.x = -lvel.x;
                }
            } else {
                if self.moving_left {
                    if lvel.norm() < PLAYER_MAX_SPEED {
                        body.append_lin_force(Vector::new(-lin_force, 0.0));
                    }
                    // lvel.x = (lvel.x - PLAYER_ACCELERATION).max(-PLAYER_MAX_SPEED);
                } else if self.moving_right {
                    if lvel.norm() < PLAYER_MAX_SPEED {
                        body.append_lin_force(Vector::new(lin_force, 0.0));
                    }
                    // lvel.x = (lvel.x + PLAYER_ACCELERATION).min(PLAYER_MAX_SPEED);
                }

            }
        }

        body.set_lin_vel(lvel);

        let pos = *body.position();

        let zero_rot = Rot2::new(Vec1::new(0.0));
        let delta_rot = pos.rotation.rotation_to(&zero_rot);
        body.set_rotation(Vec1::new(0.0));
    }

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::Some(self)
    }
}

pub const KNIFE_INIT_SPEED: f32 = 14.0;

pub struct Knife {
    body_handle: TimeRigidBodyHandle<f32>,
    hw: f32,
    hh: f32,
}

impl Knife {
    // Knife's rotation is set to same direction as velocity
    pub fn new(world_data: &mut WorldData, x: f32, y: f32, velocity: Vector<f32>) -> Knife {
        let hw = 0.15;
        let hh = 0.05;
        let density = 500.0;

        // use angle = acos(v.u / (||v|| ||u||)), where u = [1, 0].
        let rot = if velocity.norm() > 0.0 {
            let angle = (velocity.x / velocity.norm()).acos();
            if velocity.y < 0.0 {
                -angle
            } else {
                angle
            }
        } else {
            0.0
        };

        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new_dynamic(shape, density, 0.2, 0.1);
        body.set_margin(BODY_MARGIN);
        body.append_translation(&Vector::new(x, y));
        body.set_deactivation_threshold(None);
        body.set_lin_vel(velocity);
        body.set_rotation(Vec1::new(rot));

        let handle = world_data.physics_world.add_body(body);

        Knife {
            body_handle: TimeRigidBodyHandle::new(handle),
            hw: hw,
            hh: hh,
        }
    }
}

impl Entity for Knife {
    fn render(&self, physics_world: &nphysics::world::World<f32>, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();
        render::fill_rectangle(win,
                               cam,
                               [0.3, 0.3, 0.3, 1.0],
                               x,
                               y,
                               w,
                               h,
                               self.get_rotation());
    }

    fn get_body_handle_mut(&mut self) -> &mut TimeRigidBodyHandle<f32> {
        &mut self.body_handle
    }

    fn get_body_handle(&self) -> &TimeRigidBodyHandle<f32> {
        &self.body_handle
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre();
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, world_data: &mut WorldData, dt: f32) {}
}
