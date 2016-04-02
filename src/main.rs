#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate lazy_static;
extern crate piston_window;
extern crate sdl2;
extern crate sdl2_mixer;

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

    let window: PistonWindow = WindowSettings::new("dio", [800, 600])
                                   .opengl(opengl)
                                   .exit_on_esc(true)
                                   .samples(4)
                                   .vsync(true)
                                   .build()
                                   .unwrap();

    let mut world = Box::new(engine::world::World::new(engine::world::WorldData::new(14.0, 10.0)));
    let (cx, cy) = world.data.get_centre_pos();
    let cam = interface::camera::Camera::new(cx, cy, 50.0);

    // let media_handle = media::MediaHandle::new(window.factory.clone());

    {
        let gnd = entity::Ground::new(&mut world.data, 7.0, 9.5, 7.0, 0.5);
        let gnd2 = entity::Ground::new(&mut world.data, 0.5, 5.0, 0.5, 5.0);
        world.push_entity(Rc::new(RefCell::new(Box::new(gnd))));
        world.push_entity(Rc::new(RefCell::new(Box::new(gnd2))));

        let player =
            Rc::new(RefCell::new(Box::new(entity::Player::new(&mut world.data, 4.0, 6.0, 0.35, 0.95)) as Box<entity::Entity>));
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
                if !process_event(&mut world, &val) {
                    break 'outer;
                }
            }
            Option::None => {}
        }

        render::render(&e, &cam, &mut world);
    }
}

// if returns false, exit event loop
fn process_event(world: &mut engine::world::World, event: &Event) -> bool {
    match *event {
        Event::Input(ref i) => {
            match *i {
                Input::Press(ref button) => {
                    match *button {
                        Button::Mouse(mbutton) => println!("{:?}", mbutton),
                        Button::Keyboard(key) => {
                            match key {
                                Key::Q => {
                                    return false;
                                }
                                Key::A => {
                                    let p = world.get_player().unwrap();
                                    p.borrow_mut().as_player().unwrap().set_moving_left(true);
                                }
                                Key::D => {
                                    let p = world.get_player().unwrap();
                                    p.borrow_mut().as_player().unwrap().set_moving_right(true);
                                }
                                Key::Space => {
                                    let p = world.get_player().unwrap();
                                    if p.borrow_mut().as_player().unwrap().touching_ground {
                                        p.borrow_mut().as_player().unwrap().jump(&mut world.data);
                                        p.borrow_mut().as_player().unwrap().touching_ground = false;
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Input::Release(ref button) => {
                    match *button {
                        Button::Keyboard(key) => {
                            match key {
                                Key::A => {
                                    let p = world.get_player().unwrap();
                                    p.borrow_mut().as_player().unwrap().set_moving_left(false);
                                }
                                Key::D => {
                                    let p = world.get_player().unwrap();
                                    p.borrow_mut().as_player().unwrap().set_moving_right(false);
                                }
                                Key::Space => {
                                let p = world.get_player().unwrap();
                                p.borrow_mut().as_player().unwrap().release(&mut world.data);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Event::Update(UpdateArgs{dt}) => {
            world.update(dt as f32);
        }
        _ => {}
    }

    true
}
