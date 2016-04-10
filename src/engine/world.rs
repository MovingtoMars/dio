use std::cell::RefCell;
use std::rc::Rc;

use engine::entity;
use engine::entity::Entity;

use nphysics;
use num::Zero;

pub struct WorldData {
    pub physics_world: nphysics::world::World<f32>,

    width: f32, // metres
    height: f32, // metres

    current_time: f32, // seconds

    time_stopped_until: Option<f32>,

    gravity: nphysics::math::Vector<f32>,
}

pub struct World {
    pub data: WorldData,

    player: Option<Rc<RefCell<Box<entity::Entity>>>>,
    entities: Vec<Rc<RefCell<Box<entity::Entity>>>>,
}

impl WorldData {
    pub fn new(width: f32, height: f32) -> WorldData {
        let mut physics_world = nphysics::world::World::new();
        let gravity = nphysics::math::Vector::new(0.0, 9.81);
        physics_world.set_gravity(gravity);

        WorldData {
            physics_world: physics_world,
            width: width,
            height: height,
            current_time: 0.0,
            time_stopped_until: None,
            gravity: gravity,
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

    pub fn get_current_time(&self) -> f32 {
        self.current_time
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

    /// Returns true if sucessfully stops time, false otherwise.
    pub fn stop_time(&mut self, dur: f32) -> bool {
        if self.data.time_stopped_until.is_some() {
            return false;
        }

        println!("[stop time]");

        let end_time = self.data.current_time + dur;
        self.data.time_stopped_until = Some(end_time);

        self.data.physics_world.set_gravity(nphysics::math::Vector::zero());

        // TODO do this at start of world update
        for e in &mut self.entities {
            let mut e = e.borrow_mut();

            e.on_stop_time(&mut self.data);

            if e.as_player().is_none() {
                e.get_body_handle_mut().save_vel();
            }
        }

        true
    }

    pub fn start_time(&mut self) {
        println!("[start time]");

        self.data.time_stopped_until = None;

        self.data.physics_world.set_gravity(self.data.gravity);

        // TODO do this at start of world update
        for e in &mut self.entities {
            let mut e = e.borrow_mut();

            e.on_start_time(&mut self.data);

            if e.as_player().is_none() {
                e.get_body_handle_mut().restore_vel();
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        assert!(dt > 0.0);

        if let Some(t) = self.data.time_stopped_until {
            if t <= self.data.current_time {
                self.start_time();
            }
        }

        if self.data.time_stopped_until.is_some() {
            self.with_player(|w, p| {
                let handle = p.get_body_handle();
                let mut handle = handle.borrow_mut();
                let inv_mass = handle.inv_mass();
                handle.apply_central_impulse(w.data.gravity * (1.0 / inv_mass) * dt);
            });
        }

        self.data.physics_world.step(dt);

        for e in &mut self.entities {
            e.borrow_mut().pre_update(&mut self.data);
        }

        for e in &mut self.entities {
            e.borrow_mut().update(&mut self.data, dt);
        }

        if self.data.time_stopped_until.is_some() {
            for e in &mut self.entities {
                let mut e = e.borrow_mut();

                if e.as_player().is_none() {
                    e.get_body_handle_mut().update_saved_vel(dt);
                }
            }
        }

        self.data.current_time += dt;
    }

    pub fn set_player(&mut self, player: Option<Rc<RefCell<Box<entity::Entity>>>>) {
        self.player = player;
    }

    pub fn get_player(&mut self) -> Option<Rc<RefCell<Box<entity::Entity>>>> {
        self.player.clone()
    }

    pub fn with_player<F>(&mut self, mut func: F)
        where F: FnMut(&mut World, &mut entity::Player)
    {
        let p1 = self.get_player().unwrap();
        let mut pb = p1.borrow_mut();
        let p = pb.as_player().unwrap();
        func(self, p);
    }
}
