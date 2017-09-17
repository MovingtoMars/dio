use super::*;

use std::collections::HashMap;

use na::geometry::Translation;
use ncollide::shape::ShapeHandle;
use ncollide::bounding_volume::{HasBoundingVolume, AABB};
use ncollide::world::CollisionGroups;
use nphysics;
use nphysics::math::{AngularInertia, Isometry, Orientation, Point, Vector};
use nphysics::object::{RigidBodyCollisionGroups, Sensor, SensorCollisionGroups, STATIC_GROUP_ID};
use nphysics::detection::joint::{Anchor, Fixed};
use specs::Entity;

use chan;

use self::MessageToPhysicsThread::*;
use self::MessageFromPhysicsThread::*;

pub const PLAYER_GROUP_ID: usize = 1;
pub const GENERIC_DYNAMIC_GROUP_ID: usize = 2;
pub const DEAD_ENEMY_GROUP_ID: usize = 3;
pub const PARTICLE_GROUP_ID: usize = 4;

#[derive(Debug, Clone, Copy)]
pub enum CollisionGroupsKind {
    Particle,
    GenericDynamic,
    GenericStatic,
    EmbeddedKnife,
    Knife,
    Player,
    DeadEnemy,
}

impl CollisionGroupsKind {
    pub fn to_collision_groups(self) -> RigidBodyCollisionGroups {
        use self::CollisionGroupsKind::*;

        let mut g = RigidBodyCollisionGroups::new_dynamic();
        g.set_membership(&[]);

        match self {
            Particle => {
                g.modify_membership(PARTICLE_GROUP_ID, true);
                g.enable_interaction_with_static();
                g
            }
            GenericDynamic => {
                g.modify_membership(GENERIC_DYNAMIC_GROUP_ID, true);
                g.enable_interaction_with_sensors();
                g.modify_blacklist(PARTICLE_GROUP_ID, true);
                g.modify_whitelist(DEAD_ENEMY_GROUP_ID, false);
                g.enable_interaction_with_static();
                g
            }
            GenericStatic => {
                let mut g = RigidBodyCollisionGroups::new_static();
                g.set_membership(&[]);
                g.modify_whitelist(GENERIC_DYNAMIC_GROUP_ID, true);
                g.modify_whitelist(PARTICLE_GROUP_ID, true);
                // g.modify_whitelist(DEAD_ENEMY_GROUP_ID, true);
                g.enable_interaction_with_sensors();
                g
            }
            EmbeddedKnife => {
                g.enable_interaction_with_static();
                g
            }
            Knife => {
                let mut g = GenericDynamic.to_collision_groups();
                g.modify_blacklist(PLAYER_GROUP_ID, true);
                // g.modify_whitelist(DEAD_ENEMY_GROUP_ID, true);
                g
            }
            Player => {
                let mut g = GenericDynamic.to_collision_groups();
                g.modify_membership(PLAYER_GROUP_ID, true);
                g
            }
            DeadEnemy => {
                // g.set_whitelist(&[]);
                // g.enable_interaction_with_static();
                // g.disable_interaction_with_sensors();
                // g.modify_membership(DEAD_ENEMY_GROUP_ID, true);
                // g.modify_blacklist(PARTICLE_GROUP_ID, true);
                // g

                GenericDynamic.to_collision_groups()
            }
        }
    }
}

// XXX rename?
pub struct PhysicsThreadLink {
    pub send: chan::Sender<MessageToPhysicsThread>, // XXX private
    pub recv: chan::Receiver<MessageFromPhysicsThread>,
}

impl PhysicsThreadLink {
    pub fn step(&self, dt: N) {
        self.send.send(Step(dt));
        self.recv.recv().unwrap().unwrap_finish_step();
    }

    pub fn get_position(&self, id: RigidBodyID) -> Isometry<N> {
        self.send.send(GetPosition(id));
        self.recv.recv().unwrap().unwrap_position()
    }

    pub fn get_half_extents(&self, id: RigidBodyID) -> (N, N) {
        self.send.send(GetHalfExtents(id));
        self.recv.recv().unwrap().unwrap_half_extents()
    }

    pub fn get_rotation(&self, id: RigidBodyID) -> N {
        self.send.send(GetRotation(id));
        self.recv.recv().unwrap().unwrap_rotation()
    }

    pub fn get_lin_vel(&self, id: RigidBodyID) -> Vector<N> {
        self.send.send(GetLinVel(id));
        self.recv.recv().unwrap().unwrap_lin_vel()
    }

    pub fn set_lin_vel(&self, id: RigidBodyID, x: Vector<N>) {
        self.send.send(SetLinVel(id, x));
    }

    pub fn get_ang_vel(&self, id: RigidBodyID) -> Orientation<N> {
        self.send.send(GetAngVel(id));
        self.recv.recv().unwrap().unwrap_ang_vel()
    }

    pub fn set_ang_vel(&self, id: RigidBodyID, x: Orientation<N>) {
        self.send.send(SetAngVel(id, x));
    }

    pub fn get_inv_mass(&self, id: RigidBodyID) -> N {
        self.send.send(GetInvMass(id));
        self.recv.recv().unwrap().unwrap_inv_mass()
    }

    pub fn set_inv_mass(&self, id: RigidBodyID, x: N) {
        self.send.send(SetInvMass(id, x));
    }

    pub fn set_rotation(&self, id: RigidBodyID, x: nphysics::math::Rotation<N>) {
        self.send.send(SetRotation(id, x));
    }

    pub fn append_lin_force(&self, id: RigidBodyID, x: Vector<N>) {
        self.send.send(AppendLinForce(id, x));
    }

    pub fn apply_central_impulse(&self, id: RigidBodyID, x: Vector<N>) {
        self.send.send(ApplyCentralImpulse(id, x));
    }

    pub fn clear_lin_force(&self, id: RigidBodyID) {
        self.send.send(ClearLinForce(id));
    }

    pub fn set_gravity(&self, g: Vector<N>) {
        self.send.send(SetGravity(g));
    }

    pub fn add_sensor(&self, id: SensorID, shape: ShapeHandle<Point<N>, Isometry<N>>, parent: Option<RigidBodyID>, rel_pos: Option<Isometry<N>>) {
        self.send.send(AddSensor {
            id,
            shape,
            parent,
            rel_pos,
        });
    }

    pub fn get_bodies_intersecting_sensor(&self, id: SensorID) -> Vec<UserData> {
        self.send.send(GetBodiesIntersectingSensor(id));
        self.recv
            .recv()
            .unwrap()
            .unwrap_bodies_intersecting_sensor()
    }

    pub fn get_contacts(&self) -> Vec<Contact> {
        self.send.send(GetContacts);
        self.recv.recv().unwrap().unwrap_contacts()
    }

    pub fn remove_rigid_body(&self, id: RigidBodyID) {
        self.send.send(RemoveRigidBody(id));
    }

    pub fn add_fixed_joint(&self, body1: RigidBodyID, body2: RigidBodyID, pos1: Isometry<N>, pos2: Isometry<N>) {
        self.send.send(AddFixedJoint {
            body1,
            body2,
            pos1,
            pos2,
        });
    }

    pub fn set_collision_groups_kind(&self, id: RigidBodyID, kind: CollisionGroupsKind) {
        self.send.send(SetCollisionGroupsKind(id, kind));
    }

    pub fn get_shape_handle(&self, id: RigidBodyID) -> ShapeHandle<Point<N>, Isometry<N>> {
        self.send.send(GetShapeHandle(id));
        self.recv.recv().unwrap().unwrap_shape_handle()
    }
}

pub enum MessageToPhysicsThread {
    Step(N),
    AddRigidBody {
        id: RigidBodyID,
        entity: Entity,
        shape: ShapeHandle<Point<N>, Isometry<N>>,
        mass_properties: Option<(N, Point<N>, AngularInertia<N>)>,
        restitution: N,
        friction: N,
        translation: Vector<N>,
        collision_groups_kind: CollisionGroupsKind,
    },
    RemoveRigidBody(RigidBodyID),
    GetPosition(RigidBodyID),
    GetHalfExtents(RigidBodyID), // XXX rename GetBoundingHalfExtents
    GetRotation(RigidBodyID),
    SetRotation(RigidBodyID, nphysics::math::Rotation<N>),
    GetLinVel(RigidBodyID),
    SetLinVel(RigidBodyID, Vector<N>),
    GetAngVel(RigidBodyID),
    SetAngVel(RigidBodyID, Orientation<N>),
    GetInvMass(RigidBodyID),
    SetInvMass(RigidBodyID, N),
    AppendLinForce(RigidBodyID, Vector<N>),
    ClearLinForce(RigidBodyID),
    SetGravity(Vector<N>),
    ApplyCentralImpulse(RigidBodyID, Vector<N>),
    AddFixedJoint {
        body1: RigidBodyID,
        body2: RigidBodyID,
        pos1: Isometry<N>,
        pos2: Isometry<N>,
    },
    SetCollisionGroupsKind(RigidBodyID, CollisionGroupsKind),

    AddSensor {
        id: SensorID,
        shape: ShapeHandle<Point<N>, Isometry<N>>,
        parent: Option<RigidBodyID>,
        rel_pos: Option<Isometry<N>>,
    },
    GetBodiesIntersectingSensor(SensorID),

    GetContacts,
    GetShapeHandle(RigidBodyID),
}

pub enum MessageFromPhysicsThread {
    FinishStep,
    Position(Isometry<N>),
    HalfExtents(N, N),
    Rotation(N),
    LinVel(Vector<N>),
    AngVel(Orientation<N>),
    InvMass(N),
    BodiesIntersectingSensor(Vec<UserData>),
    Contacts(Vec<Contact>),
    ShapeHandle(ShapeHandle<Point<N>, Isometry<N>>),
}

impl MessageFromPhysicsThread {
    pub fn unwrap_finish_step(self) {
        match self {
            FinishStep => {}
            _ => panic!("Expected FinishStep"),
        }
    }

    pub fn unwrap_position(self) -> Isometry<N> {
        match self {
            Position(x) => x,
            _ => panic!("Expected Position"),
        }
    }

    pub fn unwrap_half_extents(self) -> (N, N) {
        match self {
            HalfExtents(x, y) => (x, y),
            _ => panic!("Expected HalfExtents"),
        }
    }

    pub fn unwrap_rotation(self) -> N {
        match self {
            Rotation(x) => x,
            _ => panic!("Expected Rotation"),
        }
    }

    pub fn unwrap_lin_vel(self) -> Vector<N> {
        match self {
            LinVel(x) => x,
            _ => panic!("Expected LinVel"),
        }
    }

    pub fn unwrap_ang_vel(self) -> Orientation<N> {
        match self {
            AngVel(x) => x,
            _ => panic!("Expected AngVel"),
        }
    }

    pub fn unwrap_inv_mass(self) -> N {
        match self {
            InvMass(x) => x,
            _ => panic!("Expected InvMass"),
        }
    }

    pub fn unwrap_bodies_intersecting_sensor(self) -> Vec<UserData> {
        match self {
            BodiesIntersectingSensor(x) => x,
            _ => panic!("Expected BodiesIntersectingSensor"),
        }
    }

    pub fn unwrap_contacts(self) -> Vec<Contact> {
        match self {
            Contacts(x) => x,
            _ => panic!("Expected Contacts"),
        }
    }

    pub fn unwrap_shape_handle(self) -> ShapeHandle<Point<N>, Isometry<N>> {
        match self {
            ShapeHandle(x) => x,
            _ => panic!("Expected ShapeHandle"),
        }
    }
}

pub fn physics_thread_inner(gravity: Vector<N>, recv: chan::Receiver<MessageToPhysicsThread>, send: chan::Sender<MessageFromPhysicsThread>) {
    let mut physics_world = nphysics::world::World::new();
    physics_world.set_gravity(gravity);

    let mut rigid_body_id_map = HashMap::new();
    let mut sensor_map = HashMap::new();

    macro_rules! body {
        ($map:expr, $id:expr) => {$map.get(&$id).unwrap().borrow()}
    }

    macro_rules! body_mut {
        ($map:expr, $id:expr) => {$map.get(&$id).unwrap().borrow_mut()}
    }

    for recv_message in recv.iter() {
        match recv_message {
            Step(dt) => {
                physics_world.step(dt);
                send.send(FinishStep);
            }

            AddRigidBody {
                id,
                entity,
                shape,
                mass_properties,
                restitution,
                friction,
                translation,
                collision_groups_kind,
            } => {
                let mut body = RigidBody::new(shape, mass_properties, restitution, friction);
                body.set_margin(BODY_MARGIN);
                body.set_translation(Translation::from_vector(translation));
                // body.set_deactivation_threshold(None); // XXX
                body.set_user_data(Some(Box::new(UserData {
                    rigid_body_id: id,
                    entity,
                })));

                body.set_collision_groups(collision_groups_kind.to_collision_groups());

                let bh = physics_world.add_rigid_body(body);
                rigid_body_id_map.insert(id, bh);
            }

            RemoveRigidBody(id) => {
                let bh = rigid_body_id_map.remove(&id);
                if let Some(bh) = bh {
                    physics_world.remove_rigid_body(&bh);
                } else {
                    // XXX
                    panic!("oh no");
                }
            }

            GetHalfExtents(id) => {
                let body = body!(rigid_body_id_map, id);
                let bounding_aabb: AABB<Point<N>> = body.bounding_volume(body.position());
                let half_extents = bounding_aabb.half_extents();
                send.send(HalfExtents(half_extents.x, half_extents.y));
            }

            GetPosition(id) => {
                let body = body!(rigid_body_id_map, id);
                send.send(Position(*body.position()));
            }

            GetRotation(id) => {
                let body = body!(rigid_body_id_map, id);
                let rotation = body.position().rotation.angle();
                send.send(Rotation(rotation));
            }

            SetRotation(id, x) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.set_rotation(x);
            }

            GetLinVel(id) => {
                let body = body!(rigid_body_id_map, id);
                send.send(LinVel(body.lin_vel()))
            }

            SetLinVel(id, x) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.set_lin_vel(x);
            }

            GetAngVel(id) => {
                let body = body!(rigid_body_id_map, id);
                send.send(AngVel(body.ang_vel()))
            }

            SetAngVel(id, x) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.set_ang_vel(x);
            }

            GetInvMass(id) => {
                let body = body!(rigid_body_id_map, id);
                send.send(InvMass(body.inv_mass()))
            }

            SetInvMass(id, x) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.set_inv_mass(x);
            }

            AppendLinForce(id, x) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.append_lin_force(x);
            }

            ClearLinForce(id) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.clear_linear_force();
            }

            SetGravity(g) => {
                physics_world.set_gravity(g);
            }

            ApplyCentralImpulse(id, x) => {
                let mut body = body_mut!(rigid_body_id_map, id);
                body.apply_central_impulse(x);
            }

            SetCollisionGroupsKind(id, k) => {
                {
                    let mut body = body_mut!(rigid_body_id_map, id);
                    body.set_collision_groups(k.to_collision_groups());
                }
                physics_world.update_rigid_body_collision_groups(rigid_body_id_map.get(&id).unwrap().clone());
            }

            AddFixedJoint {
                body1,
                body2,
                pos1,
                pos2,
            } => {
                let anchor1 = Anchor::new(Some(rigid_body_id_map.get(&body1).unwrap().clone()), pos1);
                let anchor2 = Anchor::new(Some(rigid_body_id_map.get(&body2).unwrap().clone()), pos2);

                physics_world.add_fixed(Fixed::new(anchor1, anchor2));
            }

            AddSensor {
                id,
                shape,
                parent,
                rel_pos,
            } => {
                let mut sensor = Sensor::new_with_shared_shape(
                    shape,
                    parent.map(|id| rigid_body_id_map.get(&id).unwrap().clone()),
                );
                if let Some(rel_pos) = rel_pos {
                    sensor.set_relative_position(rel_pos);
                }

                let mut cg = *sensor.collision_groups();
                cg.enable_interaction_with_static();
                cg.modify_membership(PARTICLE_GROUP_ID, false);
                *sensor.collision_groups_mut() = cg;

                sensor.enable_interfering_bodies_collection();

                sensor_map.insert(id, physics_world.add_sensor(sensor));
            }

            GetBodiesIntersectingSensor(id) => {
                let sensor = sensor_map.get(&id).unwrap().borrow();
                let interfering_bodies = sensor.interfering_bodies();

                send.send(BodiesIntersectingSensor(
                    interfering_bodies
                        .unwrap()
                        .into_iter()
                        .map(|body| {
                            *body.borrow()
                                .user_data()
                                .unwrap()
                                .downcast_ref::<UserData>()
                                .unwrap()
                        })
                        .collect(),
                ));
            }

            GetContacts => {
                let contacts = physics_world
                    .collision_world()
                    .contacts()
                    .into_iter()
                    .map(|(obj1, obj2, contact)| {
                        Contact {
                            obj1: *obj1.data
                                .borrow_rigid_body()
                                .user_data()
                                .unwrap()
                                .downcast_ref::<UserData>()
                                .unwrap(),
                            obj2: *obj2.data
                                .borrow_rigid_body()
                                .user_data()
                                .unwrap()
                                .downcast_ref::<UserData>()
                                .unwrap(),

                            depth: contact.depth,
                            normal: contact.normal,
                            position1: contact.world1,
                            position2: contact.world2,
                        }
                    })
                    .collect();

                send.send(Contacts(contacts));
            }

            GetShapeHandle(id) => send.send(ShapeHandle(body!(rigid_body_id_map, id).shape().clone())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UserData {
    pub rigid_body_id: RigidBodyID,
    pub entity: Entity,
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub obj1: UserData,
    pub obj2: UserData,
    pub normal: Vector<N>,
    pub depth: N,
    pub position1: Point<N>,
    pub position2: Point<N>,
}

impl Contact {
    pub fn flip(mut self) -> Self {
        use std;
        std::mem::swap(&mut self.obj1, &mut self.obj2);
        std::mem::swap(&mut self.position1, &mut self.position2);

        self.normal = self.normal * -1.0;

        self
    }
}
