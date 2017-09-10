use super::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use specs::{self, Join};
use nphysics::math::{Isometry, Orientation, Rotation, Vector};
use num::Zero;

pub type RS<'a, T> = specs::ReadStorage<'a, T>;
pub type WS<'a, T> = specs::WriteStorage<'a, T>;

#[derive(Clone)]
pub struct SystemContext {
    pub time: f32,
    pub physics_thread_link: Arc<Mutex<PhysicsThreadLink>>,
    pub time_is_stopped: bool,
    pub contact_map: HashMap<RigidBodyID, Vec<Contact>>,
    pub events: Arc<Mutex<Vec<Event>>>,
}

impl SystemContext {
    pub fn push_event(&self, event: Event) {
        self.events.lock().unwrap().push(event);
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

            renderable.x = pos.x;
            renderable.y = pos.y;
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
    knifec: RS<'a, Knife>,
    hitpointsc: WS<'a, Hitpoints>,
    removec: WS<'a, Remove>,

    entities: specs::Entities<'a>,
    c: specs::Fetch<'a, SystemContext>,
}

struct KnifeSystem;

impl<'a> specs::System<'a> for KnifeSystem {
    type SystemData = KnifeData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let physics = data.c.physics_thread_link.lock().unwrap();

        for (entity, &body_id, knife) in (&*data.entities, &data.rigid_body_idc, &data.knifec).join() {
            if let Some(contacts) = data.c.contact_map.get(&body_id) {
                for contact in contacts {
                    if let Some(hitpoints) = data.hitpointsc.get_mut(contact.obj2.entity) {
                        use rand;
                        use rand::distributions::IndependentSample;
                        let normal = rand::distributions::Normal::new(0.0, 1.0);
                        let chi_squared = rand::distributions::ChiSquared::new(6.0);
                        const MEAN_SIZE: f64 = 0.065;
                        let normal_size = rand::distributions::Normal::new(MEAN_SIZE, 0.015);
                        let rng = &mut rand::thread_rng();

                        for i in 0..3 {
                            let size = normal_size
                                .ind_sample(rng)
                                .max(0.055)
                                .max(BODY_MARGIN as f64);

                            // Bigger particles tend to live for less time
                            let ttl = chi_squared.ind_sample(rng).min(30.0) * (MEAN_SIZE / size);

                            data.c.push_event(Event::SpawnParticle {
                                rect: Rect::new(
                                    contact.position1.x,
                                    contact.position1.y,
                                    size as N,
                                    size as N,
                                ),
                                velocity: Vector::new(normal.ind_sample(rng) as N, normal.ind_sample(rng) as N),
                                ttl: ttl as N,
                            });
                        }

                        hitpoints.damage(1);
                        data.removec.insert(entity, Remove);
                        break;
                    }
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
