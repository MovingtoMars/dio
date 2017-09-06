use super::*;

use std::sync::{Arc, Mutex};
use std::thread;
use std::mem::uninitialized;

use ncollide::shape::{Cuboid, ShapeHandle};
use nphysics;
use nphysics::math::{AngularInertia, Orientation, Point, Rotation, Vector};
use nphysics::volumetric::Volumetric;
use num::Zero;

use chan;
use specs::{self, Component, Entity, Join};

pub type N = f32;
pub type RigidBody = nphysics::object::RigidBody<N>;
pub type RigidBodyHandle = nphysics::object::RigidBodyHandle<N>;

pub const BODY_MARGIN: f32 = 0.04;

// TODO event system: entities aren't really added until events processed

pub struct World {
    specs_world: specs::World,
    physics_thread: thread::JoinHandle<()>,
    physics_thread_link: Arc<Mutex<PhysicsThreadLink>>,
    next_rigid_body_id: u32,
    player: Entity,

    time_stop_remaining: Option<f32>,
    normal_gravity: Vector<N>,
}

impl World {
    pub fn new(x: f32, y: f32, hw: f32, hh: f32) -> Self {
        let mut specs_world = specs::World::new();

        macro_rules! register_components {
            ($world:expr, $($comp:ty),* $(,)*) => {
                $($world.register::<$comp>());+
            }
        }

        register_components! {
            specs_world,
            RigidBodyID,
            Renderable,
            Player,
            TimeStopStore,
        }

        let (physics_thread_sender, recv) = chan::sync(0);
        let (send, physics_thread_receiver) = chan::sync(0);

        let gravity = nphysics::math::Vector::new(0.0, 9.81);
        let physics_thread = thread::spawn(move || physics_thread_inner(gravity, recv, send));

        let mut world = World {
            specs_world,
            next_rigid_body_id: 0,
            physics_thread,
            physics_thread_link: Arc::new(Mutex::new(PhysicsThreadLink {
                send: physics_thread_sender,
                recv: physics_thread_receiver,
            })),
            player: unsafe { uninitialized() },
            time_stop_remaining: None,
            normal_gravity: gravity,
        };

        world.player = world.new_player(x, y, hw, hh);

        world
    }

    pub fn physics_thread_link(&self) -> Arc<Mutex<PhysicsThreadLink>> {
        self.physics_thread_link.clone()
    }

    pub fn player_entity(&self) -> Entity {
        self.player
    }

    pub fn player_rigid_body_id(&self) -> RigidBodyID {
        let idc = self.read_component::<RigidBodyID>();
        *idc.get(self.player).unwrap()
    }

    pub fn read_component<T: Component>(&self) -> specs::ReadStorage<T> {
        self.specs_world.read::<T>()
    }

    pub fn tick(&mut self, time: f32) {
        assert!(time > 0.0);

        if self.time_stop_remaining.is_some() {
            let body_id = self.player_rigid_body_id();
            let physics = self.physics_thread_link.lock().unwrap();
            let inv_mass = physics.get_inv_mass(body_id);
            physics.apply_central_impulse(body_id, self.normal_gravity * (1.0 / inv_mass) * time);
        }

        self.physics_thread_link.lock().unwrap().step(time);

        self.specs_world.maintain();

        let context = SystemContext {
            time,
            physics_thread_link: self.physics_thread_link.clone(),
            time_is_stopped: self.time_stop_remaining.is_some(),
        };
        self.specs_world.add_resource(context.clone());

        let mut dispatcher = register_systems(specs::DispatcherBuilder::new()).build();
        dispatcher.dispatch(&mut self.specs_world.res);

        self.specs_world.maintain();

        if let Some(t) = self.time_stop_remaining {
            if time >= t {
                self.start_time();
            } else {
                self.time_stop_remaining = Some(t - time);
            }
        }
    }

    /// Returns true if sucessfully stops time, false otherwise.
    pub fn stop_time(&mut self, dur: f32) -> bool {
        if self.time_stop_remaining.is_some() {
            return false;
        }

        println!("[stop time]");

        self.time_stop_remaining = Some(dur);

        let physics = self.physics_thread_link.lock().unwrap();
        physics.set_gravity(Vector::zero());

        let mut time_stop_storec = self.specs_world.write::<TimeStopStore>();
        let rigid_body_idc = self.read_component::<RigidBodyID>();

        for (&body_id, store) in (&rigid_body_idc, &mut time_stop_storec).join() {
            assert!(store.saved_lin_vel.is_none());
            assert!(store.saved_ang_vel.is_none());

            // XXX what behavious do we want?
            // store.saved_lin_vel = Some(physics.get_lin_vel(body_id));
            // store.saved_ang_vel = Some(physics.get_ang_vel(body_id));
            //
            // physics.set_lin_vel(body_id, Vector::zero());
            // physics.set_ang_vel(body_id, Orientation::zero());
        }

        true
    }

    pub fn start_time(&mut self) {
        println!("[start time]");

        self.time_stop_remaining = None;

        let physics = self.physics_thread_link.lock().unwrap();
        physics.set_gravity(self.normal_gravity);



        let mut time_stop_storec = self.specs_world.write::<TimeStopStore>();
        let rigid_body_idc = self.read_component::<RigidBodyID>();

        for (&body_id, store) in (&rigid_body_idc, &mut time_stop_storec).join() {
            assert!(store.saved_ang_vel.is_none() == store.saved_lin_vel.is_none());

            // use zero values if this body was created during time stop
            let saved_lin_vel = store.saved_lin_vel.unwrap_or(Vector::zero());
            let saved_ang_vel = store.saved_ang_vel.unwrap_or(Orientation::zero());

            let cur_lin_vel = physics.get_lin_vel(body_id);
            let cur_ang_vel = physics.get_ang_vel(body_id);

            // XXX
            // if store.saved_lin_vel.is_some() && !handle.is_active() {
            // handle.activate(na::Bounded::max_value());
            // }

            physics.set_lin_vel(body_id, cur_lin_vel + saved_lin_vel);
            physics.set_ang_vel(body_id, cur_ang_vel + saved_ang_vel);

            store.saved_lin_vel = None;
            store.saved_ang_vel = None;
        }
    }

    fn new_rigid_body_id(&mut self) -> RigidBodyID {
        let body_id = self.next_rigid_body_id;
        self.next_rigid_body_id += 1;
        RigidBodyID::new(body_id)
    }

    pub fn new_ground(&mut self, x: f32, y: f32, hw: f32, hh: f32) -> Entity {
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            shape: ShapeHandle::new(shape),
            mass_properties: None,
            restitution: 0.2,
            friction: 0.3,
            translation: Vector::new(x, y),
        };

        self.physics_thread_link.lock().unwrap().send.send(message);

        let renderable = Renderable {
            color: [0.0, 1.0, 0.0, 1.0],
            w: hw * 2.0,
            h: hh * 2.0,
            x,
            y,
            ..Renderable::default()
        };

        self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .build()
    }

    // Make sure to set world.player to the returned entity!
    fn new_player(&mut self, x: f32, y: f32, hw: f32, hh: f32) -> Entity {
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let density = 500.0;

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            shape: ShapeHandle::new(shape),
            mass_properties: Some((
                density,
                Point::new(0.0, 0.0),
                AngularInertia::new(100000000000.0),
            )),
            restitution: 0.2,
            friction: 0.1,
            translation: Vector::new(x, y),
        };

        let player = Player::new();

        self.physics_thread_link.lock().unwrap().send.send(message);

        let renderable = Renderable {
            color: [1.0, 0.8, 0.1, 1.0],
            w: hw * 2.0,
            h: hh * 2.0,
            x,
            y,
            ..Renderable::default()
        };

        self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(player)
            .build()
    }

    pub fn new_crate(&mut self, x: f32, y: f32, hw: f32, hh: f32, material: CrateMaterial) -> Entity {
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            mass_properties: Some(shape.mass_properties(material.density())),
            shape: ShapeHandle::new(shape),
            restitution: material.restitution(),
            friction: 0.6,
            translation: Vector::new(x, y),
        };

        self.physics_thread_link.lock().unwrap().send.send(message);

        let renderable = Renderable {
            color: material.color().1,
            w: hw * 2.0,
            h: hh * 2.0,
            x,
            y,
            ..Renderable::default()
        };

        self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimeStopStore::new())
            .build()
    }

    pub fn new_knife(&mut self, x: f32, y: f32, velocity: Vector<N>) -> Entity {
        let hw = 0.15;
        let hh = 0.05;
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let density = 500.0;

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            mass_properties: Some(shape.mass_properties(density)),
            shape: ShapeHandle::new(shape),
            restitution: 0.2,
            friction: 0.1,
            translation: Vector::new(x, y),
        };

        let physics = self.physics_thread_link.lock().unwrap();
        physics.send.send(message);
        physics.set_lin_vel(id, velocity);
        use num::Complex;
        let rot = Rotation::from_complex(Complex {
            re: velocity.x,
            im: velocity.y,
        });
        physics.set_rotation(id, rot);

        let renderable = Renderable {
            color: [0.3, 0.3, 0.3, 1.0],
            x,
            y,
            w: hw * 2.0,
            h: hh * 2.0,
            rotation: rot.angle(),
            ..Renderable::default()
        };

        self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimeStopStore::new())
            .build()
    }


    pub fn set_player_moving_left(&mut self, x: bool) {
        self.specs_world
            .write::<Player>()
            .get_mut(self.player)
            .unwrap()
            .moving_left = x;
    }

    pub fn set_player_moving_right(&mut self, x: bool) {
        self.specs_world
            .write::<Player>()
            .get_mut(self.player)
            .unwrap()
            .moving_right = x;
    }

    pub fn set_player_jumping(&mut self, jumping: bool) {
        let mut playerc = self.specs_world.write::<Player>();
        let player = playerc.get_mut(self.player).unwrap();
        let idc = self.read_component::<RigidBodyID>();
        let &body_id = idc.get(self.player).unwrap();

        let physics = self.physics_thread_link.lock().unwrap();

        if jumping {
            player.touching_ground = true;
            if player.touching_ground {
                // player.jump(&mut world.data);
                player.touching_ground = false;

                let mut lvel = physics.get_lin_vel(body_id);
                lvel.y = -6.0;
                physics.set_lin_vel(body_id, lvel);
                // body.on_ground = false;
            }
        } else {
            // let mut lvel = physics.get_lin_vel(body_id);
            //
            // if lvel.y < 0.0 && self.release_jump {
            //     lvel.y *= 0.45;
            //     physics.set_body_lin_vel(body_id, lvel);
            //     self.release_jump = false;
            // }
        }
    }
}

#[derive(Clone, Copy)]
pub enum CrateMaterial {
    Steel,
    Wood,
}

impl CrateMaterial {
    pub fn density(self) -> N {
        match self {
            CrateMaterial::Steel => 8000.0,
            CrateMaterial::Wood => 7000.0,
        }
    }

    pub fn restitution(self) -> N {
        match self {
            CrateMaterial::Steel => 0.6,
            CrateMaterial::Wood => 0.4,
        }
    }

    pub fn color(self) -> ([f32; 4], [f32; 4]) {
        match self {
            CrateMaterial::Steel => ([0.2, 0.2, 0.2, 1.0], [0.3, 0.3, 0.3, 1.0]),
            CrateMaterial::Wood => ([0.4, 0.2, 0.0, 1.0], [0.6, 0.3, 0.0, 1.0]),
        }
    }
}
