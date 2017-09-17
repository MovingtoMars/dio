use super::*;

use std::cmp;

use specs::{self, Component, DenseVecStorage, Entity, HashMapStorage, VecStorage};
use nphysics::math::{Orientation, Vector};

pub fn register_components(world: &mut specs::World) {
    macro_rules! register_components {
        ($world:expr, $($comp:ty),* $(,)*) => {
            $($world.register::<$comp>());+
        }
    }

    register_components! {
        world,
        RigidBodyID,
        Renderable,
        Player,
        TimeStopStore,
        Hitpoints,
        Knife,
        Remove,
        TimedRemove,
        Name,
        BasicEnemy,
        Bullet,
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RigidBodyID(u32);

impl RigidBodyID {
    pub fn new(x: u32) -> Self {
        RigidBodyID(x)
    }
}

impl Component for RigidBodyID {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct RenderItem {
    pub rel_rotation: N,
    pub rel_x: N,
    pub rel_y: N,
    pub color: [f32; 4],

    pub kind: RenderItemKind,
}

impl RenderItem {
    pub fn rectangle(rel_x: N, rel_y: N, w: N, h: N, rel_rotation: N, color: [f32; 4]) -> Self {
        RenderItem {
            rel_x,
            rel_y,
            rel_rotation,
            color,
            kind: RenderItemKind::Rectangle { w, h },
        }
    }

    pub fn text<S: Into<String>>(rel_x: N, rel_y: N, rel_rotation: N, color: [f32; 4], text: S, size: u32) -> Self {
        RenderItem {
            rel_x,
            rel_y,
            rel_rotation,
            color,
            kind: RenderItemKind::Text {
                text: text.into(),
                size,
            },
        }
    }

    pub fn info(rel_x: N, rel_y: N, rel_rotation: N, color: [f32; 4]) -> Self {
        RenderItem {
            rel_x,
            rel_y,
            rel_rotation,
            color,
            kind: RenderItemKind::Info,
        }
    }

    pub fn ellipse(rel_x: N, rel_y: N, w: N, h: N, rel_rotation: N, color: [f32; 4]) -> Self {
        RenderItem {
            rel_x,
            rel_y,
            rel_rotation,
            color,
            kind: RenderItemKind::Ellipse { w, h },
        }
    }
}

#[derive(Debug, Clone)]
pub enum RenderItemKind {
    Rectangle { w: N, h: N },
    Text { text: String, size: u32 },
    Info,
    Ellipse { w: N, h: N },
}

#[derive(Debug, Clone)]
pub struct Renderable {
    pub x: N,
    pub y: N,
    pub rotation: N,
    pub items: Vec<RenderItem>,
}

impl Renderable {
    pub fn new(x: N, y: N, rotation: N) -> Self {
        Renderable {
            x,
            y,
            rotation,
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, item: RenderItem) {
        self.items.push(item);
    }

    pub fn with(mut self, item: RenderItem) -> Self {
        self.push(item);
        self
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Player {
    pub moving_right: bool,
    pub moving_left: bool,
    pub touching_ground: bool,
    pub release_jump: bool,
    pub picking_up: bool,

    num_knives: usize,
    max_num_knives: usize,

    sensor_id: SensorID,
}

impl Player {
    pub fn new(sensor_id: SensorID, max_num_knives: usize) -> Self {
        Player {
            moving_right: false,
            moving_left: false,
            touching_ground: false,
            release_jump: false,
            picking_up: false,

            num_knives: max_num_knives,
            max_num_knives,

            sensor_id,
        }
    }

    pub fn sensor_id(&self) -> SensorID {
        self.sensor_id
    }

    pub fn dec_knives(&mut self) {
        if self.num_knives >= 1 {
            self.num_knives -= 1;
        }
    }

    pub fn inc_knives(&mut self) {
        if self.num_knives < self.max_num_knives {
            self.num_knives += 1;
        }
    }

    pub fn num_knives(&self) -> usize {
        self.num_knives
    }

    pub fn max_num_knives(&self) -> usize {
        self.max_num_knives
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
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
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct Knife {
    pub stuck_into_entity: Option<Entity>,
}

impl Component for Knife {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Hitpoints {
    current: u16,
    max: u16,
}

impl Component for Hitpoints {
    type Storage = VecStorage<Self>;
}

impl Hitpoints {
    pub fn new(max: u16) -> Self {
        Hitpoints { max, current: max }
    }

    pub fn set_current(&mut self, x: u16) {
        self.current = cmp::min(x, self.max);
    }

    pub fn damage(&mut self, damage: u16) {
        if damage > self.current {
            self.set_current(0);
        } else {
            let new = self.current - damage;
            self.set_current(new);
        }
    }

    pub fn heal(&mut self, heal: u16) {
        self.current = cmp::min(self.current + heal, self.max);
    }

    pub fn current(&self) -> u16 {
        self.current
    }

    pub fn max(&self) -> u16 {
        self.max
    }
}

// XXX is this the best way to remove entities?
#[derive(Debug, Clone)]
pub struct Remove;

impl Component for Remove {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct TimedRemove(pub N);

impl Component for TimedRemove {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Name(pub String);

impl Component for Name {
    type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct BasicEnemy {
    pub is_dead: bool,
}

impl BasicEnemy {
    pub fn new() -> Self {
        BasicEnemy { is_dead: false }
    }
}

impl Component for BasicEnemy {
    type Storage = HashMapStorage<Self>;
}

// TODO: CCD
#[derive(Debug, Clone)]
pub struct Bullet;

impl Component for Bullet {
    type Storage = HashMapStorage<Self>;
}
