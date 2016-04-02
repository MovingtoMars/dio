extern crate nphysics2d as nphysics;

use std::cell::RefCell;
use std::rc::Rc;

use engine::entity;
use engine::entity::Entity;

pub struct WorldData {
    width: f32, // metres
    height: f32, // metres
    pub physics_world: nphysics::world::World,
}

pub struct World {
    pub data: WorldData,

    player: Option<Rc<RefCell<Box<entity::Entity>>>>,
    entities: Vec<Rc<RefCell<Box<entity::Entity>>>>,
}

impl WorldData {
    pub fn new(width: f32, height: f32) -> WorldData {
        let mut physics_world = nphysics::world::World::new();
        physics_world.set_gravity(nphysics::math::Vect::new(0.0, 9.81));

        WorldData {
            width: width,
            height: height,
            physics_world: physics_world,
        }
    }

    pub fn get_dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn get_centre_pos(&self) -> (f32, f32) {
        (self.width / 2.0, self.height / 2.0)
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_height(&self) -> f32 {
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

    pub fn update(&mut self, dt: f32) {
        assert!(dt > 0.0);

        // self.data.b2world.step(dt, 5, 5);
        self.data.physics_world.step(dt);

        let data = &mut self.data;

        for e in &mut self.entities {
            e.borrow_mut().pre_update(data);
        }

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
