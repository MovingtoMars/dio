#![allow(dead_code)]

extern crate piston_window;

use std::cell::RefCell;
use std::rc::Rc;

use piston_window::*;

mod engine;
mod render;
mod interface;
mod physics;

fn main() {
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

    {
        let gnd = engine::entity::Ground::new(&mut world.data, 7.0, 9.5, 7.0, 0.5);
        world.push_entity(Rc::new(RefCell::new(Box::new(gnd))));

        let player = Rc::new(RefCell::new(Box::new(engine::entity::Player::new(&mut world.data, 4.0, 6.0, 0.4, 0.95)) as Box<engine::entity::Entity>));
        world.push_entity(player.clone());
        world.set_player(Option::Some(player));
    }

    'outer: for e in window {
        match e.event {
            Option::Some(ref val) => {
                if !process_event(&mut world, &val) {
                    break 'outer;
                }
            },
            Option::None => {},
        }

        render::render(&e, &cam, &mut world);
    }
}

// if returns false, exit event loop
fn process_event(world: &mut engine::world::World, event: &Event) -> bool {
    match *event {
        Event::Input(ref i) => match *i {
            Input::Press(ref button) => match *button {
                Button::Mouse(mbutton) => println!("{:?}", mbutton),
                Button::Keyboard(key) => {
                    match key {
                        Key::Q => {
                            return false;
                        }
                        Key::Left => {
                            let p = world.get_player().unwrap();
                            p.borrow_mut().as_player().unwrap().set_moving_left(true);
                        }
                        Key::Right => {
                            let p = world.get_player().unwrap();
                            p.borrow_mut().as_player().unwrap().set_moving_right(true);
                        },
                        _ => {},
                    }
                },
                _ => {},
            },
            Input::Release(ref button) => match *button {
                Button::Keyboard(key) => {
                    match key {
                        Key::Left => {
                            let p = world.get_player().unwrap();
                            p.borrow_mut().as_player().unwrap().set_moving_left(false);
                        }
                        Key::Right => {
                            let p = world.get_player().unwrap();
                            p.borrow_mut().as_player().unwrap().set_moving_right(false);
                        },
                        _ => {},
                    }
                },
                _ => {},
            },
            _ => {},
        },
        Event::Update(UpdateArgs{dt}) => {
            world.update(dt);
        },
        _ => {},
    }

    true
}
