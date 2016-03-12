extern crate core;
extern crate box2d;

use engine::entity;
use engine::entity::Entity;

use self::box2d::b2;

pub struct WorldData {
    width: f32, // metres
    height: f32, // metres
    pub b2world: b2::World,
}

pub struct World {
    entities: Vec<Box<entity::Entity>>,
    pub data: WorldData,
}

impl WorldData {
    pub fn new(width: f32, height: f32) -> WorldData {
        WorldData{
            width: width,
            height: height,
            b2world: b2::World::new(&b2::Vec2{x: 0.0, y: -9.81}),
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
        World{
            data: data,
            entities: Vec::new(),
        }
    }

    pub fn get_entities_ref(&self) -> &Vec<Box<entity::Entity>> {
        &self.entities
    }

    pub fn push_entity(&mut self, e: Box<entity::Entity>) {
        self.entities.push(e);
    }

    pub fn update(&mut self, dt: f32) {
        //let mut world = RefCell::new(self);
        let data = &mut self.data;
        for e in &mut self.entities {
            e.update(data, dt);
        }
    }
}
