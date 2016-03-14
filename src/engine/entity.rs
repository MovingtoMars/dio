extern crate piston_window;
extern crate box2d;

use self::box2d::b2;

use piston_window::*;
use engine::world::*;
use interface::camera::Camera;
use render;

// Note that the following conventions are used with entities:
// x, y: the *centre* position of the entity
// hw, hh: the half-width and half-height of the entity

pub trait Entity {
    fn get_body_handle(&mut self) -> &b2::BodyHandle;
    fn get_centre(&self, b2world: &b2::World) -> (f32, f32);
    fn get_bounding_box(&self, b2world: &b2::World) -> (f32, f32, f32, f32);

    fn render(&self, b2world: &b2::World, win: &PistonWindow, cam: &Camera);
    fn update(&mut self, world: &mut WorldData, dt: f32);

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::None
    }
}

pub struct Ground {
    body: b2::BodyHandle,
    hw: f32,
    hh: f32,
}

impl Ground {
    pub fn new(world_data: &mut WorldData, x: f32, y: f32, hw: f32, hh: f32) -> Ground {
        let mut def = b2::BodyDef::new();
        def.body_type = b2::BodyType::Static;
        def.fixed_rotation = true;
        let body = world_data.b2world.create_body(&def);

        let mut shape = b2::PolygonShape::new();
        shape.set_as_box(hw, hh);

        world_data.b2world.get_body_mut(body).create_fast_fixture(&shape, 1.0);

        world_data.b2world.get_body_mut(body).set_transform(&b2::Vec2{x: x, y: y}, 0.0);

        Ground{body: body, hw: hw, hh: hh}
    }
}

impl Entity for Ground {
    fn render(&self, b2world: &b2::World, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box(b2world);
        render::fill_rectangle(win, cam, [0.0, 1.0, 0.0, 1.0], x, y, w, h);
    }

    fn get_body_handle(&mut self) -> &b2::BodyHandle {
        &mut self.body
    }

    fn get_centre(&self, b2world: &b2::World) -> (f32, f32) {
        let trans = *b2world.get_body(self.body).position();
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self, b2world: &b2::World) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre(b2world);
        (cx - self.hw , cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, _: &mut WorldData, _: f32) {

    }
}

pub struct Player {
    body: b2::BodyHandle,
    hw: f32,
    hh: f32,

    moving_right: bool,
    moving_left: bool,
}

impl Player {
    pub fn new(world_data: &mut WorldData, x: f32, y: f32, hw: f32, hh: f32) -> Player {
        let mut def = b2::BodyDef::new();
        def.body_type = b2::BodyType::Dynamic;
        def.fixed_rotation = true;
        let body = world_data.b2world.create_body(&def);

        let mut shape = b2::PolygonShape::new();
        shape.set_as_box(hw, hh);

        world_data.b2world.get_body_mut(body).create_fast_fixture(&shape, 1.0);

        world_data.b2world.get_body_mut(body).set_transform(&b2::Vec2{x: x, y: y}, 0.0);

        Player{
            body: body,
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

const USAIN_BOLT_MAX_SPEED: f32 = 12.4;
const PLAYER_MAX_SPEED: f32 = USAIN_BOLT_MAX_SPEED;
const PLAYER_ACCELERATION: f32 = 1.5;

impl Entity for Player {
    fn render(&self, b2world: &b2::World, win: &PistonWindow, cam: &Camera) {
        let (x, y, w, h) = self.get_bounding_box(b2world);
        render::fill_rectangle(win, cam, [1.0, 0.8, 0.1, 1.0], x, y, w, h);
    }

    fn get_body_handle(&mut self) -> &b2::BodyHandle {
        &mut self.body
    }

    fn get_centre(&self, b2world: &b2::World) -> (f32, f32) {
        let trans = *b2world.get_body(self.body).position();
        (trans.x, trans.y)
    }

    fn get_bounding_box(&self, b2world: &b2::World) -> (f32, f32, f32, f32) {
        let (cx, cy) = self.get_centre(b2world);
        (cx - self.hw , cy - self.hh, self.hw * 2.0, self.hh * 2.0)
    }

    fn update(&mut self, world_data: &mut WorldData, _: f32) {
        let mut body = world_data.b2world.get_body_mut(self.body);

        let mut vel = body.linear_velocity().clone();

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

        body.set_linear_velocity(&vel);
    }

    fn as_player(&mut self) -> Option<&mut Player> {
        Option::Some(self)
    }
}
