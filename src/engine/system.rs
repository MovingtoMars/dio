use super::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use specs::{self, Join};
use nphysics::math::{Isometry, Orientation, Point, Rotation, Vector};
use ncollide::query;
use num::Zero;
use na::UnitComplex;

pub type RS<'a, T> = specs::ReadStorage<'a, T>;
pub type WS<'a, T> = specs::WriteStorage<'a, T>;

#[derive(Clone)]
pub struct SystemContext {
    pub time: f32,
    pub physics_thread_link: Arc<Mutex<PhysicsThreadLink>>,
    pub time_is_stopped: bool,
    pub contact_map: HashMap<RigidBodyID, Vec<Contact>>,
    pub events: Arc<Mutex<Vec<Event>>>,
    pub player: specs::Entity,
}

impl SystemContext {
    pub fn push_event(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }

    pub fn push_events<I: IntoIterator<Item = Event>>(&self, it: I) {
        let mut events = self.events.lock().unwrap();
        for event in it.into_iter() {
            events.push(event);
        }
    }
}

pub fn register_systems<'a, 'b>(d: specs::DispatcherBuilder<'a, 'b>) -> specs::DispatcherBuilder<'a, 'b> {
    let d = d.add(
        UpdateRenderableFromRigidBodyIDSystem,
        "UpdateRenderableFromRigidBodyIDSystem",
        &[],
    );
    let d = d.add(PlayerSystem, "PlayerSystem", &[]);
    let d = d.add(TimeStopSystem, "TimeStopSystem", &[]);
    let d = d.add(KnifeSystem, "KnifeSystem", &[]);

    let d = d.add_barrier();
    let d = d.add(TimedRemoveSystem, "TimedRemoveSystem", &[]);
    let d = d.add(RemoveSystem, "RemoveSystem", &["TimedRemoveSystem"]);

    d
}

#[derive(SystemData)]
struct UpdateRenderableFromRigidBodyIDData<'a> {
    rigidbodyidc: WS<'a, RigidBodyID>, // write because we lock the physics thread link
    renderablec: WS<'a, Renderable>,

    c: specs::Fetch<'a, SystemContext>,
}

struct UpdateRenderableFromRigidBodyIDSystem;

impl<'a> specs::System<'a> for UpdateRenderableFromRigidBodyIDSystem {
    type SystemData = UpdateRenderableFromRigidBodyIDData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let physics_thread_link = data.c.physics_thread_link.lock().unwrap();

        for (&rigidbodyid, renderable) in (&data.rigidbodyidc, &mut data.renderablec).join() {
            let pos = physics_thread_link.get_position(rigidbodyid);

            renderable.x = pos.translation.vector.x;
            renderable.y = pos.translation.vector.y;
            renderable.rotation = physics_thread_link.get_rotation(rigidbodyid);
        }
    }
}

#[derive(SystemData)]
struct PlayerData<'a> {
    rigidbodyidc: WS<'a, RigidBodyID>,
    playerc: WS<'a, Player>,

    c: specs::Fetch<'a, SystemContext>,
}


const USAIN_BOLT_MAX_SPEED: f32 = 12.4;
const PLAYER_MAX_SPEED: f32 = USAIN_BOLT_MAX_SPEED * 0.5;
const PLAYER_ACCELERATION: f32 = PLAYER_MAX_SPEED * 2.5;

struct PlayerSystem;

impl<'a> specs::System<'a> for PlayerSystem {
    type SystemData = PlayerData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let physics = data.c.physics_thread_link.lock().unwrap();

        for (&body_id, player) in (&data.rigidbodyidc, &mut data.playerc).join() {
            player.touching_ground = !physics
                .get_bodies_intersecting_sensor(player.sensor_id())
                .is_empty();

            physics.clear_lin_force(body_id);

            let mut lvel = physics.get_lin_vel(body_id);


            let mass = 1.0 / physics.get_inv_mass(body_id);
            let lin_force = mass * PLAYER_ACCELERATION;

            // if self.touching_ground // why??????
            {
                if player.moving_right == player.moving_left {
                    let neg = lvel.x < 0.0;
                    lvel.x = (lvel.x.abs() - PLAYER_ACCELERATION * data.c.time).max(0.0);
                    if neg {
                        lvel.x = -lvel.x;
                    }
                } else {
                    if player.moving_left {
                        if lvel.norm() < PLAYER_MAX_SPEED {
                            physics.append_lin_force(body_id, Vector::new(-lin_force, 0.0));
                        }
                    // lvel.x = (lvel.x - PLAYER_ACCELERATION).max(-PLAYER_MAX_SPEED);
                    } else if player.moving_right {
                        if lvel.norm() < PLAYER_MAX_SPEED {
                            physics.append_lin_force(body_id, Vector::new(lin_force, 0.0));
                        }
                        // lvel.x = (lvel.x + PLAYER_ACCELERATION).min(PLAYER_MAX_SPEED);
                    }
                }
            }

            physics.set_lin_vel(body_id, lvel);

            physics.set_rotation(body_id, Rotation::new(0.0));
        }
    }
}


#[derive(SystemData)]
struct TimeStopData<'a> {
    rigidbodyidc: WS<'a, RigidBodyID>,
    time_stop_storec: WS<'a, TimeStopStore>,

    c: specs::Fetch<'a, SystemContext>,
}

struct TimeStopSystem;

impl<'a> specs::System<'a> for TimeStopSystem {
    type SystemData = TimeStopData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let physics = data.c.physics_thread_link.lock().unwrap();

        if data.c.time_is_stopped {
            for (&body_id, store) in (&data.rigidbodyidc, &mut data.time_stop_storec).join() {
                assert!(store.saved_ang_vel.is_none() == store.saved_lin_vel.is_none());

                // use zero values if this body was created during time stop
                let saved_lin_vel = store.saved_lin_vel.unwrap_or(Vector::zero());
                let saved_ang_vel = store.saved_ang_vel.unwrap_or(Orientation::zero());

                let init_lin_vel = physics.get_lin_vel(body_id);
                let init_ang_vel = physics.get_ang_vel(body_id);

                let ratio = (0.001f64.powf(data.c.time as f64)) as N;
                let new_lin_vel = init_lin_vel * ratio;
                let new_ang_vel = init_ang_vel * ratio;

                store.saved_lin_vel = Some(saved_lin_vel + init_lin_vel - new_lin_vel);
                store.saved_ang_vel = Some(saved_ang_vel + init_ang_vel - new_ang_vel);

                physics.set_lin_vel(body_id, new_lin_vel);
                physics.set_ang_vel(body_id, new_ang_vel);
            }
        }
    }
}

#[derive(SystemData)]
struct KnifeData<'a> {
    rigid_body_idc: WS<'a, RigidBodyID>,
    knifec: WS<'a, Knife>,
    hitpointsc: WS<'a, Hitpoints>,
    removec: WS<'a, Remove>,
    playerc: WS<'a, Player>,

    entities: specs::Entities<'a>,
    c: specs::Fetch<'a, SystemContext>,
}

struct KnifeSystem;

impl<'a> specs::System<'a> for KnifeSystem {
    type SystemData = KnifeData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let physics = data.c.physics_thread_link.lock().unwrap();

        for (entity, &body_id, knife) in (&*data.entities, &data.rigid_body_idc, &mut data.knifec).join() {
            if knife.stuck_into_entity.is_some() {
                // pass
            } else {
                if let Some(contacts) = data.c.contact_map.get(&body_id) {
                    for contact in contacts {
                        if let Some(hitpoints) = data.hitpointsc.get_mut(contact.obj2.entity) {
                            knife.stuck_into_entity = Some(contact.obj2.entity);
                            data.c.push_events(spawn_blood(contact.position1));
                            hitpoints.damage(1);

                            physics.set_lin_vel(body_id, Vector::new(0.0, 0.0));
                            physics.set_ang_vel(body_id, Orientation::new(0.0));

                            add_fixed_joint_from_contact(&physics, &contact);
                            physics.set_collision_groups_kind(body_id, CollisionGroupsKind::EmbeddedKnife);
                            break;
                        }
                    }
                }
            }

            if data.playerc.get(data.c.player).unwrap().picking_up {
                let player_body_id = *data.rigid_body_idc.get(data.c.player).unwrap();
                let player_pos = physics.get_position(player_body_id);
                let player_shape = physics.get_shape_handle(player_body_id);
                let knife_pos = physics.get_position(body_id);
                let knife_shape = physics.get_shape_handle(body_id);

                if query::contact(&player_pos, &*player_shape, &knife_pos, &*knife_shape, 0.05).is_some() {
                    // Pick up the knife
                    data.removec.insert(entity, Remove);
                    data.playerc.get_mut(data.c.player).unwrap().inc_knives();
                }
            }
        }
    }
}

#[derive(SystemData)]
struct RemoveData<'a> {
    rigid_body_idc: WS<'a, RigidBodyID>,
    removec: WS<'a, Remove>,

    entities: specs::Entities<'a>,
    c: specs::Fetch<'a, SystemContext>,
}

struct RemoveSystem;

impl<'a> specs::System<'a> for RemoveSystem {
    type SystemData = RemoveData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        for (&body_id, _) in (&data.rigid_body_idc, &data.removec).join() {
            data.c
                .physics_thread_link
                .lock()
                .unwrap()
                .remove_rigid_body(body_id);
        }

        for (entity, _) in (&*data.entities, &data.removec).join() {
            data.entities.delete(entity);
        }
    }
}


#[derive(SystemData)]
struct TimedRemoveData<'a> {
    timed_removec: WS<'a, TimedRemove>,
    removec: WS<'a, Remove>,

    entities: specs::Entities<'a>,
    c: specs::Fetch<'a, SystemContext>,
}

struct TimedRemoveSystem;

impl<'a> specs::System<'a> for TimedRemoveSystem {
    type SystemData = TimedRemoveData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (entity, timed_remove) in (&*data.entities, &mut data.timed_removec).join() {
            if !data.c.time_is_stopped {
                timed_remove.0 -= data.c.time;
            }

            if timed_remove.0 <= 0.0 {
                data.removec.insert(entity, Remove);
            }
        }
    }
}

// Helper functions

fn spawn_blood(origin: Point<N>) -> Vec<Event> {
    let mut res = Vec::new();

    use rand;
    use rand::distributions::{ChiSquared, IndependentSample, Normal, Range};

    let mean_size = 0.065;

    let size_dist = Normal::new(mean_size, 0.02);
    let velocity_dist = Normal::new(0.0, 1.0);
    let ttl_dist = ChiSquared::new(4.0);

    let rng = &mut rand::thread_rng();

    let max_num_dist = Range::new(2, 5);

    for i in 0..max_num_dist.ind_sample(rng) {
        let size = size_dist
            .ind_sample(rng)
            .max(0.055)
            .max(BODY_MARGIN as f64)
            .min(0.1);

        // Bigger particles tend to live for less time
        let ttl = ttl_dist.ind_sample(rng).min(30.0) * (mean_size / size);

        res.push(Event::SpawnParticle {
            rect: Rect::new(origin.x, origin.y, size as N, size as N),
            velocity: Vector::new(
                velocity_dist.ind_sample(rng) as N,
                velocity_dist.ind_sample(rng) as N,
            ),
            ttl: ttl as N,
        });
    }

    res
}

fn add_fixed_joint_from_contact(physics: &PhysicsThreadLink, contact: &Contact) {
    let body1 = contact.obj1.rigid_body_id;
    let body2 = contact.obj2.rigid_body_id;

    let p1 = contact.position1 - Point::from_coordinates(physics.get_position(body1).translation.vector);
    let p2 = contact.position2 - Point::from_coordinates(physics.get_position(body2).translation.vector);

    let r1 = physics.get_rotation(body1);
    let r2 = physics.get_rotation(body2);

    let mut local_pos1 = Isometry::new(p1, 0.0);
    let mut local_pos2 = Isometry::new(p2, 0.0);

    local_pos1.append_rotation_mut(&Rotation::new(-r1));
    local_pos1.rotation = UnitComplex::new(-r1);

    local_pos2.append_rotation_mut(&Rotation::new(-r2));
    local_pos2.rotation = UnitComplex::new(-r2);

    physics.add_fixed_joint(body1, body2, local_pos1, local_pos2);
}
