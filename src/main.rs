#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// TODO fix player jumping on bullets

extern crate chan;
extern crate gfx_device_gl;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics2d as nphysics;
extern crate num;
extern crate piston_window;
extern crate rand;
extern crate rodio;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
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

use engine::*;

use levels::*;

use interface::camera::Camera;

use std::collections::HashSet;

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
        .exit_on_esc(true)
        .samples(4)
        .vsync(true)
        .build()
        .unwrap();

    // let level = levels::Level {
    //     name: String::from("Test Level"),
    //     player_start_pos: (-3.0, 1.0),
    //     entities: vec![
    //         LevelEntity::Ground {
    //             rect: Rect::new(0.0, 4.5, 7.0, 0.5),
    //         },
    //         LevelEntity::Ground {
    //             rect: Rect::new(-6.5, 0.0, 0.5, 5.0),
    //         },
    //         LevelEntity::Crate {
    //             rect: Rect::new(-2.0, 3.5, 0.5, 0.5),
    //             material: CrateMaterial::Steel,
    //         },
    //         LevelEntity::Crate {
    //             rect: Rect::new(-2.0, 2.5, 0.5, 0.5),
    //             material: CrateMaterial::Wood,
    //         },
    //         LevelEntity::Enemy {
    //             rect: Rect::new(2.0, 2.5, PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT),
    //         },
    //     ],
    // };

    let media_handle = media::MediaHandle::new(window.factory.clone());

    let level = Level::load(&media_handle, "default.level.json").unwrap();

    let mut world = level.to_world();

    let mut cam = Camera::new(0.0, 0.0, INIT_WIN_WIDTH, INIT_WIN_HEIGHT, 50.0);

    let mut fonts = render::Fonts::new(&media_handle);
    level.save(&media_handle, "default.level.json").unwrap();

    window.set_ups(60);

    let mut keys_down = HashSet::new();

    'outer: while let Some(e) = window.next() {
        let mut stats = stats_handler.get();
        if !process_event(
            &mut world,
            &mut window,
            &mut cam,
            &e,
            &mut stats,
            &mut fonts,
            &mut keys_down,
        ) {
            break 'outer;
        }

        if keys_down.contains(&Key::Space) || keys_down.contains(&Key::W) {
            world.set_player_jumping(true);
        } else {
            world.set_player_jumping(false);
        }

        stats_handler.set(stats);
    }

    stats_handler.finish();
}

pub const KNIFE_INIT_SPEED: N = 14.0;

fn spawn_knife(world: &mut World, cam: &mut Camera) {
    let (kx, ky) = cam.screen_to_pos(cam.mouse_x, cam.mouse_y);

    let physics = world.physics_thread_link();
    let pos = physics
        .lock()
        .unwrap()
        .get_position(world.player_rigid_body_id());
    let px = pos.translation.vector.x;
    let py = pos.translation.vector.y;

    let sx = if kx < px {
        px - PLAYER_HALF_WIDTH * 1.6
    } else {
        px + PLAYER_HALF_WIDTH * 1.6
    };
    let sy = py - PLAYER_HALF_HEIGHT * 0.32;

    let vel = Vector::new(kx - sx, ky - sy).normalize() * KNIFE_INIT_SPEED;

    world.player_throw_knife(sx, sy, vel);
}

// if returns false, exit event loop
fn process_event(
    world: &mut World,
    window: &mut piston_window::PistonWindow,
    cam: &mut Camera,
    event: &Input,
    stats: &mut stat::Stats,
    fonts: &mut render::Fonts,
    keys_down: &mut HashSet<Key>,
) -> bool {
    if let &Input::Update(UpdateArgs { dt }) = event {
        world.tick(dt as N);

        let win_draw_size = window.draw_size();
        cam.set_window_dimensions(win_draw_size.width, win_draw_size.height);
        let physics = world.physics_thread_link();
        let pos = physics
            .lock()
            .unwrap()
            .get_position(world.player_rigid_body_id());
        let px = pos.translation.vector.x;
        let py = pos.translation.vector.y;
        cam.set_pos_smooth(px, py);

        stats.total_game_time += dt;
        return true;
    }

    match *event {
        Input::Render(_) => {
            render::render(window, cam, world, event, fonts);
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
                keys_down.insert(key);

                match key {
                    Key::Q => return false,
                    Key::A => world.set_player_moving_left(true),
                    Key::D => world.set_player_moving_right(true),
                    Key::C => world.set_player_picking_up(true),
                    Key::E => {
                        world.new_bullet(Vector::new(0.0, 1.5), 0.08, Vector::new(20.0, 0.0)); // XXX
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        Input::Release(ref button) => match *button {
            Button::Keyboard(key) => {
                keys_down.remove(&key);

                match key {
                    Key::A => world.set_player_moving_left(false),
                    Key::D => world.set_player_moving_right(false),
                    Key::C => world.set_player_picking_up(false),
                    Key::F => if world.stop_time(5.0) {
                        stats.num_time_stops += 1;
                    },
                    _ => {}
                }
            }
            _ => {}
        },
        _ => {}
    }

    true
}
