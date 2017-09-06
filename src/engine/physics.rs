use super::*;

use std::collections::HashMap;

use na::Isometry2;
use na::geometry::Translation;
use ncollide::shape::ShapeHandle;
use ncollide::bounding_volume::{HasBoundingVolume, AABB};
use nphysics;
use nphysics::math::{AngularInertia, Orientation, Point, Vector};

use chan;

use self::MessageToPhysicsThread::*;
use self::MessageFromPhysicsThread::*;

// XXX rename?
pub struct PhysicsThreadLink {
    pub send: chan::Sender<MessageToPhysicsThread>, // XXX private
    pub recv: chan::Receiver<MessageFromPhysicsThread>,
}

impl PhysicsThreadLink {
    pub fn step(&self, dt: N) {
        self.send.send(Step(dt));
    }

    pub fn get_position(&self, id: RigidBodyID) -> (N, N) {
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
}

pub enum MessageToPhysicsThread {
    Step(N),
    AddRigidBody {
        id: RigidBodyID,
        shape: ShapeHandle<Point<N>, Isometry2<N>>,
        mass_properties: Option<(N, Point<N>, AngularInertia<N>)>,
        restitution: N,
        friction: N,
        translation: Vector<N>,
    },
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
}

pub enum MessageFromPhysicsThread {
    Position(N, N),
    HalfExtents(N, N),
    Rotation(N),
    LinVel(Vector<N>),
    AngVel(Orientation<N>),
    InvMass(N),
}

impl MessageFromPhysicsThread {
    pub fn unwrap_position(self) -> (N, N) {
        match self {
            Position(x, y) => (x, y),
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
}

pub fn physics_thread_inner(gravity: Vector<N>, recv: chan::Receiver<MessageToPhysicsThread>, send: chan::Sender<MessageFromPhysicsThread>) {
    let mut physics_world = nphysics::world::World::new();
    physics_world.set_gravity(gravity);

    let mut rigid_body_id_map = HashMap::new();

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
            }

            AddRigidBody {
                id,
                shape,
                mass_properties,
                restitution,
                friction,
                translation,
            } => {
                let mut body = RigidBody::new(shape, mass_properties, restitution, friction);
                body.set_margin(BODY_MARGIN);
                body.set_translation(Translation::from_vector(translation));
                body.set_deactivation_threshold(None); // XXX
                rigid_body_id_map.insert(id, physics_world.add_rigid_body(body));
            }

            GetHalfExtents(id) => {
                let body = body!(rigid_body_id_map, id);
                let bounding_aabb: AABB<Point<N>> = body.bounding_volume(body.position());
                let half_extents = bounding_aabb.half_extents();
                send.send(HalfExtents(half_extents.x, half_extents.y));
            }

            GetPosition(id) => {
                let body = body!(rigid_body_id_map, id);
                let x = body.position().translation.vector.x;
                let y = body.position().translation.vector.y;
                send.send(Position(x, y));
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
        }
    }
}
