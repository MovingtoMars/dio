#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate lazy_static;

extern crate piston_window;
extern crate sdl2;
extern crate sdl2_mixer;
extern crate nphysics2d as nphysics;
extern crate nalgebra;

use std::cell::RefCell;
use std::rc::Rc;

use piston_window::*;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG, INIT_OGG, AUDIO_S16LSB};

mod engine;
mod render;
mod interface;
mod media;
mod audio;

use engine::entity;
use engine::entity::Entity;
use engine::world::*;
use engine::entity::Player;

use interface::camera::Camera;

use nphysics::math::Vector;

use nalgebra::Norm;

const INIT_WIN_WIDTH: u32 = 800;
const INIT_WIN_HEIGHT: u32 = 600;

fn main() {
    let sdl = sdl2::init().unwrap();
    sdl.audio().unwrap();
    // let mut timer = sdl.timer().unwrap();
    sdl2_mixer::init(INIT_MP3 | INIT_FLAC | INIT_MOD | INIT_FLUIDSYNTH | INIT_MODPLUG | INIT_OGG).unwrap();
    let frequency = 44100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = 2; // Stereo
    let chunk_size = 1024;
    sdl2_mixer::open_audio(frequency, format, channels, chunk_size).unwrap();

    audio::init();

    let opengl = OpenGL::V2_1;

    let window: PistonWindow = WindowSettings::new("dio", [INIT_WIN_WIDTH, INIT_WIN_HEIGHT])
                                   .opengl(opengl)
                                   .exit_on_esc(true)
                                   .samples(4)
                                   .vsync(true)
                                   .build()
                                   .unwrap();

    let mut world = Box::new(World::new(WorldData::new(14.0, 10.0)));
    let (cx, cy) = world.data.get_centre_pos();
    let mut cam = Camera::new(cx, cy, INIT_WIN_WIDTH, INIT_WIN_HEIGHT, 50.0);

    // let media_handle = media::MediaHandle::new(window.factory.clone());

    {
        let gnd = entity::Ground::new(&mut world.data, 7.0, 9.5, 7.0, 0.5);
        let gnd2 = entity::Ground::new(&mut world.data, 0.5, 5.0, 0.5, 5.0);
        world.push_entity(Rc::new(RefCell::new(Box::new(gnd))));
        world.push_entity(Rc::new(RefCell::new(Box::new(gnd2))));

        let player =
            Rc::new(RefCell::new(Box::new(Player::new(&mut world.data, 4.0, 6.0, 0.35, 0.95)) as Box<entity::Entity>));
        world.push_entity(player.clone());
        let block =
            Rc::new(RefCell::new(Box::new(entity::Crate::new(&mut world.data, entity::CrateMaterial::Wood, 5.0, 7.5, 0.5, 0.5)) as Box<entity::Entity>));
        let block2 =
            Rc::new(RefCell::new(Box::new(entity::Crate::new(&mut world.data, entity::CrateMaterial::Steel, 5.0, 8.5, 0.5, 0.5)) as Box<entity::Entity>));
        world.push_entity(block);
        world.push_entity(block2);
        world.set_player(Option::Some(player));
    }

    'outer: for e in window {
        match e.event {
            Option::Some(ref val) => {
                if !process_event(&mut world, &mut cam, &val) {
                    break 'outer;
                }
            }
            Option::None => {}
        }

        render::render(&e, &cam, &mut world);
    }
}

fn spawn_knife(world: &mut World, cam: &mut Camera, player: &mut Player) {
    let (kx, ky) = cam.screen_to_pos(cam.mouse_x, cam.mouse_y);
    let (px, py) = player.get_centre();

    let (_, _, w, h) = player.get_bounding_box();

    let sx = if kx < px {
        px - w * 0.8
    } else {
        px + w * 0.8
    };
    let sy = py - h * 0.16;

    let vel = Vector::new(kx - sx, ky - sy).normalize() * entity::KNIFE_INIT_SPEED;

    let knife = entity::Knife::new(&mut world.data, sx, sy, vel);
    world.push_entity(Rc::new(RefCell::new(Box::new(knife))));
}

// if returns false, exit event loop
fn process_event(world: &mut World, cam: &mut Camera, event: &Event) -> bool {
    if let &Event::Update(UpdateArgs{dt}) = event {
        world.update(dt as f32);
        return true;
    }

    match *event {
        Event::Input(ref i) => match *i {
            Input::Resize(w, h) => {
                cam.win_w = w;
                cam.win_h = h;
            },
            Input::Move(ref motion) => match *motion {
                Motion::MouseCursor(x, y) => {
                    cam.mouse_x = x;
                    cam.mouse_y = y;
                },
                _ => {},
            },
            Input::Press(ref button) => match *button {
                Button::Mouse(mbutton) => {
                    if mbutton == MouseButton::Left {
                        world.with_player(|world, p| spawn_knife(world, cam, p));
                    }
                },
                Button::Keyboard(key) => match key {
                    Key::Q => return false,
                    Key::A => world.with_player(|_, p| p.set_moving_left(true)),
                    Key::D => world.with_player(|_, p| p.set_moving_right(true)),
                    Key::Space => {
                        world.with_player(|world, p| if p.touching_ground {
                            p.jump(&mut world.data);
                            p.touching_ground = false;
                        });
                    }
                    _ => {}
                },
                _ => {}
            },
            Input::Release(ref button) => match *button {
                Button::Keyboard(key) => match key {
                    Key::A => world.with_player(|_, p| p.set_moving_left(false)),
                    Key::D => world.with_player(|_, p| p.set_moving_right(false)),
                    Key::Space => world.with_player(|world, p| p.release(&mut world.data)),
                    Key::T => world.stop_time(5.0),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }

    true
}
