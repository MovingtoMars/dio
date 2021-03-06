use super::*;

use std::sync::{Arc, Mutex};
use std::thread;
use std::mem::uninitialized;
use std::collections::HashMap;

use ncollide::shape::{Ball, Cuboid, ShapeHandle};
use nphysics;
use nphysics::math::{AngularInertia, Isometry, Orientation, Point, Rotation, Translation, Vector};
use nphysics::volumetric::Volumetric;
use num::Zero;

use chan;
use specs::{self, Component, Entity, Join};

pub type N = f32;
pub type RigidBody = nphysics::object::RigidBody<N>;
pub type RigidBodyHandle = nphysics::object::RigidBodyHandle<N>;

pub const BODY_MARGIN: N = 0.04;

pub const PLAYER_HALF_WIDTH: N = 0.35;
pub const PLAYER_HALF_HEIGHT: N = 0.85;

// TODO event system: entities aren't really added until events processed

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SensorID(u32);

struct Counter {
    next: u32,
}

impl Counter {
    pub fn new() -> Self {
        Counter { next: 1 }
    }

    pub fn next(&mut self) -> u32 {
        let x = self.next;
        self.next += 1;
        x
    }
}

pub struct World {
    specs_world: specs::World,
    physics_thread: thread::JoinHandle<()>,
    physics_thread_link: Arc<Mutex<PhysicsThreadLink>>,
    next_rigid_body_id: Counter,
    next_sensor_id: Counter,
    player: Entity,

    time_stop_remaining: Option<N>,
    normal_gravity: Vector<N>,
}

impl World {
    pub fn new(x: N, y: N) -> Self {
        let mut specs_world = specs::World::new();

        register_components(&mut specs_world);

        let (physics_thread_sender, recv) = chan::sync(0);
        let (send, physics_thread_receiver) = chan::sync(0);

        let gravity = nphysics::math::Vector::new(0.0, 9.81);
        let physics_thread = thread::spawn(move || physics_thread_inner(gravity, recv, send));

        let mut world = World {
            specs_world,
            next_rigid_body_id: Counter::new(),
            next_sensor_id: Counter::new(),
            physics_thread,
            physics_thread_link: Arc::new(Mutex::new(PhysicsThreadLink {
                send: physics_thread_sender,
                recv: physics_thread_receiver,
            })),
            player: unsafe { uninitialized() },
            time_stop_remaining: None,
            normal_gravity: gravity,
        };

        world.player = world.new_player(x, y);

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

    pub fn clone_player_component(&self) -> Player {
        self.specs_world
            .read::<Player>()
            .get(self.player)
            .unwrap()
            .clone()
    }

    pub fn entities(&self) -> specs::Fetch<specs::EntitiesRes> {
        self.specs_world.entities()
    }

    pub fn tick(&mut self, time: N) {
        assert!(time > 0.0);

        if self.time_stop_remaining.is_some() {
            let body_id = self.player_rigid_body_id();
            let physics = self.physics_thread_link.lock().unwrap();
            let inv_mass = physics.get_inv_mass(body_id);
            physics.apply_central_impulse(body_id, self.normal_gravity * (1.0 / inv_mass) * time);
        }

        self.physics_thread_link.lock().unwrap().step(time);
        let contacts = self.physics_thread_link.lock().unwrap().get_contacts();

        let mut contact_map = HashMap::new();

        for contact in contacts {
            contact_map
                .entry(contact.obj1.rigid_body_id)
                .or_insert_with(|| Vec::new())
                .push(contact.clone());
            contact_map
                .entry(contact.obj2.rigid_body_id)
                .or_insert_with(|| Vec::new())
                .push(contact.flip());
        }

        self.specs_world.maintain();

        let events = Arc::new(Mutex::new(Vec::new()));

        let context = SystemContext {
            time,
            physics_thread_link: self.physics_thread_link.clone(),
            time_is_stopped: self.time_stop_remaining.is_some(),
            contact_map,
            events: events.clone(),
            player: self.player,
        };
        self.specs_world.add_resource(context.clone());

        let mut dispatcher = register_systems(specs::DispatcherBuilder::new()).build();
        dispatcher.dispatch(&mut self.specs_world.res);

        // self.specs_world.maintain();

        for event in &*events.lock().unwrap() {
            self.run_event(event);
        }

        if let Some(t) = self.time_stop_remaining {
            if time >= t {
                self.start_time();
            } else {
                self.time_stop_remaining = Some(t - time);
            }
        }
    }

    pub fn run_event(&mut self, event: &Event) {
        match *event {
            Event::SpawnParticle {
                rect,
                velocity,
                ttl,
            } => {
                self.new_particle(rect, velocity, ttl);
            }
        }
    }

    /// Returns true if sucessfully stops time, false otherwise.
    pub fn stop_time(&mut self, dur: N) -> bool {
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

    pub fn time_stop_remaining(&self) -> Option<N> {
        self.time_stop_remaining
    }

    fn new_rigid_body_id(&mut self) -> RigidBodyID {
        RigidBodyID::new(self.next_rigid_body_id.next())
    }

    fn new_sensor_id(&mut self) -> SensorID {
        SensorID(self.next_sensor_id.next())
    }

    pub fn new_ground(&mut self, rect: Rect) -> Entity {
        let Rect { x, y, hw, hh } = rect;
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let renderable = Renderable::new(x, y, 0.0).with(RenderItem::rectangle(
            0.0,
            0.0,
            hw * 2.0,
            hh * 2.0,
            0.0,
            [0.0, 1.0, 0.0, 1.0],
        ));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            shape: ShapeHandle::new(shape),
            mass_properties: None,
            restitution: 0.2,
            friction: 0.3,
            translation: Vector::new(x, y),
            collision_groups_kind: CollisionGroupsKind::GenericStatic,
            ccd: None,
        };
        self.physics_thread_link.lock().unwrap().send.send(message);

        entity
    }

    pub fn new_particle(&mut self, rect: Rect, velocity: Vector<N>, ttl: N) -> Entity {
        let Rect { x, y, hw, hh } = rect;
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let renderable = Renderable::new(x, y, 0.0).with(RenderItem::rectangle(
            0.0,
            0.0,
            hw * 2.0,
            hh * 2.0,
            0.0,
            [1.0, 0.0, 0.0, 1.0],
        ));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimedRemove(ttl))
            .with(TimeStopStore::new())
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            shape: ShapeHandle::new(shape),
            mass_properties: Some((1400.0, Point::new(0.0, 0.0), AngularInertia::new(1.0))),
            restitution: 0.0,
            friction: 0.5,
            translation: Vector::new(x, y),
            collision_groups_kind: CollisionGroupsKind::Particle,
            ccd: None,
        };
        self.physics_thread_link.lock().unwrap().send.send(message);
        self.physics_thread_link
            .lock()
            .unwrap()
            .set_lin_vel(id, velocity);

        entity
    }

    // Make sure to set world.player to the returned entity!
    fn new_player(&mut self, x: N, y: N) -> Entity {
        let hw = PLAYER_HALF_WIDTH;
        let hh = PLAYER_HALF_HEIGHT;

        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();
        let sensor_id = self.new_sensor_id();

        let density = 500.0;

        let player = Player::new(sensor_id, 6);

        let renderable = Renderable::new(x, y, 0.0)
            .with(RenderItem::rectangle(
                0.0,
                0.0,
                hw * 2.0,
                hh * 2.0,
                0.0,
                [1.0, 0.8, 0.1, 1.0],
            ))
            .with(RenderItem::info(0.0, -hh * 1.3, 0.0, [0.0, 0.0, 0.0, 1.0]));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(player)
            .with(Hitpoints::new(5))
            .with(Name("Player".into()))
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            shape: ShapeHandle::new(shape),
            mass_properties: Some((
                density,
                Point::new(0.0, 0.0),
                AngularInertia::new(100000000000.0),
            )),
            restitution: 0.2,
            friction: 0.1,
            translation: Vector::new(x, y),
            collision_groups_kind: CollisionGroupsKind::Player,
            ccd: None,
        };


        let sensor_height = 0.03;
        let sensor_shape = Cuboid::new(Vector::new(hw * 0.90, sensor_height));
        let rel_pos = Isometry::from_parts(
            Translation::from_vector(Vector::new(0.0, hh + sensor_height)),
            Rotation::from_angle(0.0),
        );

        {
            let physics = self.physics_thread_link.lock().unwrap();
            physics.send.send(message);
            physics.add_sensor(
                sensor_id,
                ShapeHandle::new(sensor_shape),
                Some(id),
                Some(rel_pos),
            );
        }

        entity
    }

    pub fn new_crate(&mut self, rect: Rect, material: CrateMaterial) -> Entity {
        let Rect { x, y, hw, hh } = rect;
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let renderable = Renderable::new(x, y, 0.0)
            .with(RenderItem::rectangle(
                0.0,
                0.0,
                hw * 2.0,
                hh * 2.0,
                0.0,
                material.color().0,
            ))
            .with(RenderItem::rectangle(
                0.0,
                0.0,
                hw * 1.6,
                hh * 1.6,
                0.0,
                material.color().1,
            ));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimeStopStore::new())
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            mass_properties: Some(shape.mass_properties(material.density())),
            shape: ShapeHandle::new(shape),
            restitution: material.restitution(),
            friction: 0.6,
            translation: Vector::new(x, y),
            collision_groups_kind: CollisionGroupsKind::GenericDynamic,
            ccd: None,
        };

        self.physics_thread_link.lock().unwrap().send.send(message);

        entity
    }

    pub fn new_enemy(&mut self, rect: Rect) -> Entity {
        let Rect { x, y, hw, hh } = rect;
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let density = 1000.0;

        let renderable = Renderable::new(x, y, 0.0)
            .with(RenderItem::rectangle(
                0.0,
                0.0,
                hw * 2.0,
                hh * 2.0,
                0.0,
                [0.0, 0.0, 1.0, 1.0],
            ))
            .with(RenderItem::info(0.0, -hh * 1.3, 0.0, [0.0, 0.0, 0.0, 1.0]));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimeStopStore::new())
            .with(Hitpoints::new(5))
            .with(BasicEnemy::new())
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            mass_properties: Some(shape.mass_properties(density)),
            shape: ShapeHandle::new(shape),
            restitution: 0.2,
            friction: 0.3,
            translation: Vector::new(x, y),
            collision_groups_kind: CollisionGroupsKind::GenericDynamic,
            ccd: None,
        };

        self.physics_thread_link.lock().unwrap().send.send(message);

        entity
    }

    pub fn new_bullet(&mut self, pos: Vector<N>, radius: N, lin_vel: Vector<N>) -> Entity {
        let shape = Ball::new(radius - BODY_MARGIN);
        let id = self.new_rigid_body_id();

        let density = 8000.0;

        let renderable = Renderable::new(pos.x, pos.y, 0.0).with(RenderItem::ellipse(
            0.0,
            0.0,
            radius * 2.0,
            radius * 2.0,
            0.0,
            [0.0, 0.0, 1.0, 1.0],
        ));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimeStopStore::new())
            .with(Bullet)
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            mass_properties: Some(shape.mass_properties(density)),
            shape: ShapeHandle::new(shape),
            restitution: 0.2,
            friction: 0.1,
            translation: pos,
            collision_groups_kind: CollisionGroupsKind::GenericDynamic,
            ccd: Some(0.04),
        };

        self.physics_thread_link.lock().unwrap().send.send(message);
        self.physics_thread_link
            .lock()
            .unwrap()
            .set_lin_vel(id, lin_vel);

        entity
    }

    pub fn player_throw_knife(&mut self, x: N, y: N, velocity: Vector<N>) -> Option<Entity> {
        {
            let mut playerc = self.specs_world.write::<Player>();
            let player = playerc.get_mut(self.player).unwrap();
            if player.num_knives() > 0 {
                player.dec_knives();
            } else {
                return None;
            }
        }

        Some(self.new_knife(x, y, velocity))
    }

    pub fn new_knife(&mut self, x: N, y: N, velocity: Vector<N>) -> Entity {
        let hw = 0.18;
        let hh = 0.08;
        let shape = Cuboid::new(Vector::new(hw - BODY_MARGIN, hh - BODY_MARGIN));
        let id = self.new_rigid_body_id();

        let density = 500.0;

        use num::Complex;
        let rot = Rotation::from_complex(Complex {
            re: velocity.x,
            im: velocity.y,
        });
        let renderable = Renderable::new(x, y, rot.angle()).with(RenderItem::rectangle(
            0.0,
            0.0,
            hw * 2.0,
            hh * 2.0,
            0.0,
            [0.3, 0.3, 0.3, 1.0],
        ));

        let entity = self.specs_world
            .create_entity()
            .with(id)
            .with(renderable)
            .with(TimeStopStore::new())
            .with(Knife {
                stuck_into_entity: None,
            })
            .build();

        let message = MessageToPhysicsThread::AddRigidBody {
            id,
            entity,
            mass_properties: Some(shape.mass_properties(density)),
            shape: ShapeHandle::new(shape),
            restitution: 0.2,
            friction: 0.1,
            translation: Vector::new(x, y),
            collision_groups_kind: CollisionGroupsKind::Knife,
            ccd: Some(0.04),
        };

        let physics = self.physics_thread_link.lock().unwrap();
        physics.send.send(message);
        physics.set_lin_vel(id, velocity);
        physics.set_rotation(id, rot);

        entity
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

    pub fn set_player_picking_up(&mut self, x: bool) {
        self.specs_world
            .write::<Player>()
            .get_mut(self.player)
            .unwrap()
            .picking_up = x;
    }

    pub fn set_player_jumping(&mut self, jumping: bool) {
        let mut playerc = self.specs_world.write::<Player>();
        let player = playerc.get_mut(self.player).unwrap();
        let idc = self.read_component::<RigidBodyID>();
        let &body_id = idc.get(self.player).unwrap();

        let physics = self.physics_thread_link.lock().unwrap();

        if jumping {
            if player.touching_ground {
                // player.jump(&mut world.data);
                player.touching_ground = false;

                let mut lvel = physics.get_lin_vel(body_id);
                lvel.y = -6.0;
                physics.set_lin_vel(body_id, lvel);
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub enum Event {
    SpawnParticle {
        rect: Rect,
        velocity: Vector<N>,
        ttl: N,
    },
}
