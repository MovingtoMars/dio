extern crate piston_window;
extern crate ncollide_entities;
extern crate nphysics2d as nphysics;
extern crate nalgebra;

use piston_window::*;
use engine::world::*;
use interface::camera::Camera;
use render;

use self::nalgebra::{Rotation, Rot2, Vec1, RotationTo, Norm};
use self::ncollide_entities::shape::Cuboid;
use self::nphysics::math::Vect;
use self::nphysics::object::{RigidBody, RigidBodyHandle};

// NOTE: For all objects that have their velocities changes manually, make sure to turn off the deactivation threshold.

// Note that the following conventions are used with entities:
// x, y: the *centre* position of the entity
// hw, hh: the half-width and half-height of the entity

const BODY_MARGIN: f32 = 0.04;

pub trait Entity {
    fn get_body_handle(&mut self) -> &RigidBodyHandle;
    fn get_centre(&self) -> (f32, f32);
    fn get_bounding_box(&self) -> (f32, f32, f32, f32);

    fn render(&self, physics_world: &nphysics::world::World, win: &PistonWindow, cam: &Camera);
    fn pre_update(&mut self, world_data: &mut WorldData);
    fn update(&mut self, world: &mut WorldData, dt: f32);

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::None
    }
}

pub struct Ground {
    body_handle: RigidBodyHandle,
    hw: f32,
    hh: f32,
}

impl Ground {
    pub fn new(world_data: &mut WorldData, x: f32, y: f32, hw: f32, hh: f32) -> Ground {
        let shape = Cuboid::new(Vect::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new_static(shape, 0.2, 0.3);
        body.append_translation(&Vect::new(x, y));
        let handle = world_data.physics_world.add_body(body);

        Ground {
            body_handle: handle,
            hw: hw,
            hh: hh,
        }
    }
}

impl Entity for Ground {
    fn render(&self, physics_world: &nphysics::world::World, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();
        render::fill_rectangle(win, cam, [0.0, 1.0, 0.0, 1.0], x, y, w, h, self.body_handle.borrow_mut().position().rotation.rotation().x);
    }

    fn get_body_handle(&mut self) -> &RigidBodyHandle {
        &mut self.body_handle
    }

    fn get_centre(&self) -> (f32, f32) {
        let trans = self.body_handle.borrow().position().translation;
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre();
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn pre_update(&mut self, world_data: &mut WorldData) {}
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
    body_handle: RigidBodyHandle,
    hw: f32,
    hh: f32,
    material: CrateMaterial,
}

impl Crate {
    pub fn new(world_data: &mut WorldData, mat: CrateMaterial, x: f32, y: f32, hw: f32, hh: f32) -> Crate {
        let shape = Cuboid::new(Vect::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new_dynamic(shape, mat.density(), mat.restitution(), 0.6);
        body.append_translation(&Vect::new(x, y));

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
    fn render(&self, physics_world: &nphysics::world::World, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();

        let (c1, c2) = match self.material {
            CrateMaterial::Steel => ([0.2, 0.2, 0.2, 1.0], [0.3, 0.3, 0.3, 1.0]),
            CrateMaterial::Wood => ([0.4, 0.2, 0.0, 1.0], [0.6, 0.3, 0.0, 1.0]),
        };

        render::fill_rectangle(win, cam, c1, x, y, w, h, self.body_handle.borrow_mut().position().rotation.rotation().x);
        render::fill_rectangle(win, cam, c2, x + w * 0.1, y + h * 0.1, w * 0.8, h * 0.8, self.body_handle.borrow_mut().position().rotation.rotation().x);
    }

    fn get_body_handle(&mut self) -> &RigidBodyHandle {
        &mut self.body_handle
    }

    fn get_centre(&self) -> (f32, f32) {
        let trans = self.body_handle.borrow().position().translation;
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre();
        (cx - self.hw, cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn pre_update(&mut self, world_data: &mut WorldData) {}
    fn update(&mut self, world_data: &mut WorldData, _: f32) {}
}

pub struct Player {
    body_handle: RigidBodyHandle,
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

        let shape = Cuboid::new(Vect::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let mut body = RigidBody::new_dynamic(shape, density, 0.2, 0.1);
        body.append_translation(&Vect::new(x, y));
        body.set_deactivation_threshold(None);

        let handle = world_data.physics_world.add_body(body);

        Player {
            body_handle: handle,
            hw: hw,
            hh: hh,
            moving_right: false,
            moving_left: false,
            touching_ground: false,
            release_jump:true,
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
        //body.on_ground = false;
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
    fn render(&self, physics_world: &nphysics::world::World, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box();
        render::fill_rectangle(win, cam, [1.0, 0.8, 0.1, 1.0], x, y, w, h, self.body_handle.borrow_mut().position().rotation.rotation().x);
    }

    fn get_body_handle(&mut self) -> &RigidBodyHandle {
        &mut self.body_handle
    }

    fn get_centre(&self) -> (f32, f32) {
        let trans = self.body_handle.borrow().position().translation;
        (trans.x, trans.y)
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

        //if body.on_ground {
            self.touching_ground = true;
            self.release_jump = true;
        //}

        let mass = 1.0 / body.inv_mass();
        let lin_force = mass * PLAYER_ACCELERATION;

        //if self.touching_ground // why??????
        {
            if self.moving_right == self.moving_left {
                let neg = lvel.x < 0.0;
                lvel.x = (lvel.x.abs() - PLAYER_ACCELERATION).max(0.0);
                if neg {
                    lvel.x = -lvel.x;
                }
            } else {
                if self.moving_left {
                    if lvel.norm() < PLAYER_MAX_SPEED {
                        body.append_lin_force(Vect::new(-lin_force, 0.0));
                    }
                    //lvel.x = (lvel.x - PLAYER_ACCELERATION).max(-PLAYER_MAX_SPEED);
                } else if self.moving_right {
                    if lvel.norm() < PLAYER_MAX_SPEED {
                        body.append_lin_force(Vect::new(lin_force, 0.0));
                    }
                    //lvel.x = (lvel.x + PLAYER_ACCELERATION).min(PLAYER_MAX_SPEED);
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
