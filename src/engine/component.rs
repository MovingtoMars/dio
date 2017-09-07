use super::*;

use specs::{Component, DenseVecStorage, VecStorage};
use nphysics::math::{Orientation, Vector};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RigidBodyID(u32);

impl RigidBodyID {
    pub fn new(x: u32) -> Self {
        RigidBodyID(x)
    }
}

impl Component for RigidBodyID {
    type Storage = VecStorage<RigidBodyID>;
}

#[derive(Debug, Default, Clone)]
pub struct Renderable {
    pub rotation: f32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}

#[derive(Debug, Clone)]
pub struct Player {
    pub moving_right: bool,
    pub moving_left: bool,
    pub touching_ground: bool,
    pub release_jump: bool,

    sensor_id: SensorID,
}

impl Player {
    pub fn new(sensor_id: SensorID) -> Self {
        Player {
            moving_right: false,
            moving_left: false,
            touching_ground: false,
            release_jump: false,
            sensor_id,
        }
    }

    pub fn sensor_id(&self) -> SensorID {
        self.sensor_id
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Player>;
}


#[derive(Debug, Clone, Default)]
pub struct TimeStopStore {
    pub saved_lin_vel: Option<Vector<N>>,
    pub saved_ang_vel: Option<Orientation<N>>,
}

impl TimeStopStore {
    pub fn new() -> Self {
        TimeStopStore::default()
    }
}

impl Component for TimeStopStore {
    type Storage = VecStorage<TimeStopStore>;
}
