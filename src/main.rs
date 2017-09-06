#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate chan;
extern crate gfx_device_gl;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics2d as nphysics;
extern crate num;
extern crate piston_window;
extern crate rodio;
extern crate rustc_serialize;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate specs;

use piston_window::*;
use nphysics::math::Vector;

mod engine;
mod render;
mod interface;
mod media;
mod audio;
mod stat;
mod levels;

use engine::{CrateMaterial, World};

use interface::camera::Camera;

const INIT_WIN_WIDTH: u32 = 800;
const INIT_WIN_HEIGHT: u32 = 600;

fn main() {
    audio::init();

    let opengl = OpenGL::V2_1;

    let mut stats_handler = stat::Handler::new();
    let mut stats = stats_handler.get();
    stats.num_startups += 1;
    stats_handler.set(stats);

    let mut window: PistonWindow = WindowSettings::new("dio", [INIT_WIN_WIDTH, INIT_WIN_HEIGHT])
                                //    .opengl(opengl)
                                   .exit_on_esc(true)
                                   .samples(4)
                                   .vsync(true)
                                   .build()
                                   .unwrap();

    let mut world = World::new(-3.0, 1.0, PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT);

    world.new_ground(0.0, 4.5, 7.0, 0.5);
    world.new_ground(-6.5, 0.0, 0.5, 5.0);
    world.new_crate(-2.0, 3.5, 0.5, 0.5, CrateMaterial::Steel);
    world.new_crate(-2.0, 2.5, 0.5, 0.5, CrateMaterial::Wood);

    let mut cam = Camera::new(0.0, 0.0, INIT_WIN_WIDTH, INIT_WIN_HEIGHT, 50.0);

    let media_handle = media::MediaHandle::new(window.factory.clone());
    //
    // (&levels::Level{
    //     name: String::from("Test Level"),
    //     player_start_pos: (1.0, 2.0),
    // }).save(&media_handle, "default.level.json").unwrap();

    window.set_ups(60);

    'outer: while let Some(e) = window.next() {
        let mut stats = stats_handler.get();
        if !process_event(&mut world, &mut window, &mut cam, &e, &mut stats) {
            break 'outer;
        }

        stats_handler.set(stats);
    }

    stats_handler.finish();
}

pub const KNIFE_INIT_SPEED: f32 = 14.0;
const PLAYER_HALF_WIDTH: f32 = 0.35;
const PLAYER_HALF_HEIGHT: f32 = 0.95;

fn spawn_knife(world: &mut World, cam: &mut Camera) {
    let (kx, ky) = cam.screen_to_pos(cam.mouse_x, cam.mouse_y);

    let physics = world.physics_thread_link();
    let (px, py) = physics
        .lock()
        .unwrap()
        .get_position(world.player_rigid_body_id());

    let sx = if kx < px {
        px - PLAYER_HALF_WIDTH * 1.6
    } else {
        px + PLAYER_HALF_WIDTH * 1.6
    };
    let sy = py - PLAYER_HALF_HEIGHT * 0.32;

    let vel = Vector::new(kx - sx, ky - sy).normalize() * KNIFE_INIT_SPEED;

    world.new_knife(sx, sy, vel);
}

// if returns false, exit event loop
fn process_event(world: &mut World, window: &mut piston_window::PistonWindow, cam: &mut Camera, event: &Input, stats: &mut stat::Stats) -> bool {
    if let &Input::Update(UpdateArgs { dt }) = event {
        world.tick(dt as f32);
        stats.total_game_time += dt;
        return true;
    }

    match *event {
        Input::Render(_) => {
            render::render(window, cam, world, event);
        }
        Input::Resize(w, h) => {
            cam.win_w = w;
            cam.win_h = h;
        }
        Input::Move(ref motion) => match *motion {
            Motion::MouseCursor(x, y) => {
                cam.mouse_x = x;
                cam.mouse_y = y;
            }
            _ => {}
        },
        Input::Press(ref button) => match *button {
            Button::Mouse(mbutton) => {
                stats.num_clicks += 1;
                if mbutton == MouseButton::Left {
                    stats.num_knives_spawned += 1;
                    spawn_knife(world, cam);
                }
            }
            Button::Keyboard(key) => {
                stats.num_key_presses += 1;
                match key {
                    Key::Q => return false,
                    Key::A => world.set_player_moving_left(true),
                    Key::D => world.set_player_moving_right(true),
                    Key::Space | Key::W => world.set_player_jumping(true),
                    _ => {}
                }
            }
            _ => {}
        },
        Input::Release(ref button) => match *button {
            Button::Keyboard(key) => match key {
                Key::A => world.set_player_moving_left(false),
                Key::D => world.set_player_moving_right(false),
                Key::Space | Key::W => world.set_player_jumping(false),
                Key::T => if world.stop_time(5.0) {
                    stats.num_time_stops += 1;
                },
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }

    true
}
