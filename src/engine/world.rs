extern crate core;

use std::cell::RefCell;
use std::rc::Rc;

use engine::entity;
use engine::entity::Entity;

use physics;

pub struct WorldData {
    width: f64, // metres
    height: f64, // metres
    pub physics_world: physics::space::Space<Rc<RefCell<Box<Entity>>>>,
}

pub struct World {
    pub data: WorldData,

    player: Option<Rc<RefCell<Box<entity::Entity>>>>,
    entities: Vec<Rc<RefCell<Box<entity::Entity>>>>,
}

impl WorldData {
    pub fn new(width: f64, height: f64) -> WorldData {
        WorldData {
            width: width,
            height: height,
            physics_world: physics::space::Space::new(physics::space::Vec2::new(0.0, 9.81)),
        }
    }

    pub fn get_dimensions(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    pub fn get_centre_pos(&self) -> (f64, f64) {
        (self.width / 2.0, self.height / 2.0)
    }

    pub fn get_width(&self) -> f64 {
        self.width
    }

    pub fn get_height(&self) -> f64 {
        self.height
    }
}

impl World {
    pub fn new(data: WorldData) -> World {
        World {
            data: data,
            entities: Vec::new(),
            player: Option::None,
        }
    }

    pub fn get_entities_ref(&self) -> &Vec<Rc<RefCell<Box<entity::Entity>>>> {
        &self.entities
    }

    pub fn push_entity(&mut self, e: Rc<RefCell<Box<entity::Entity>>>) {
        self.entities.push(e);
    }

    pub fn update(&mut self, dt: f64) {
        assert!(dt > 0.0);

        // self.data.b2world.step(dt, 5, 5);
        self.data.physics_world.update(dt);

        let data = &mut self.data;
        for e in &mut self.entities {
            e.borrow_mut().update(data, dt);
        }
    }

    pub fn set_player(&mut self, player: Option<Rc<RefCell<Box<entity::Entity>>>>) {
        self.player = player;
    }

    pub fn get_player(&mut self) -> Option<Rc<RefCell<Box<entity::Entity>>>> {
        self.player.clone()
    }
}
