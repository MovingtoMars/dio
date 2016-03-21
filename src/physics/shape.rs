use std::f64;

use physics::world::Vec2;
use physics::body;

pub trait Shape {
    fn bounds(&self, pos: Vec2) -> (f64, f64, f64, f64);
    fn cast_ray(&self, pos: Vec2, origin: Vec2, dir: Vec2) -> Option<Vec2>;
    fn contains(&self, pos: Vec2, point: Vec2) -> bool;
    fn mass(&self, density: f64) -> f64;
    fn variant(&self) -> ShapeVariant;
    fn collides_with(&self,
                     self_pos: Vec2,
                     other: &Shape,
                     other_pos: Vec2)
                     -> Option<body::Collision>;
}

pub enum ShapeVariant {
    Rect(Rect),
}

#[derive(Copy,Clone)]
pub struct Rect {
    hw: f64,
    hh: f64,
}

impl Rect {
    pub fn new(hw: f64, hh: f64) -> Rect {
        Rect { hw: hw, hh: hh }
    }
}

impl Shape for Rect {
    fn collides_with(&self,
                     self_pos: Vec2,
                     other: &Shape,
                     other_pos: Vec2)
                     -> Option<body::Collision> {
        match other.variant() {
            ShapeVariant::Rect(rect) => {
                let (x1, y1, x2, y2) = self.bounds(self_pos);
                let (cx, cy) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
                let (ox1, oy1, ox2, oy2) = rect.bounds(other_pos);
                // let (cox1, cox2) = ((ox1 + ox2) / 2.0, (oy1 + oy2) / 2.0);

                let collides = !(y2 < oy1 || y1 > oy2 || x1 > ox2 || x2 < ox1);
                if !collides {
                    return None;
                }

                let ix1 = x1.max(ox1);
                let iy1 = y1.max(oy1);
                let ix2 = x2.min(ox2);
                let iy2 = y2.min(oy2);

                let collision_point = Vec2::new((ix1 + ix2) / 2.0, (iy1 + iy2) / 2.0);

                let self_collision_normal = if ix2 - ix1 > iy2 - iy1 {
                    // top/bottom collision
                    if collision_point.y < cy {
                        // self is bottom
                        // other is top
                        Vec2::new(0.0, 1.0)
                    } else {
                        // self is top
                        // other is bottom
                        Vec2::new(0.0, -1.0)
                    }
                } else {
                    // side collision
                    if collision_point.x < cx {
                        // self is right
                        // other is left
                        Vec2::new(1.0, 0.0)
                    } else {
                        // self is left
                        // other is right
                        Vec2::new(-1.0, 0.0)
                    }
                };

                return Some(body::Collision {
                    point_a: collision_point,
                    point_b: collision_point,
                    normal_a: self_collision_normal,
                    normal_b: self_collision_normal.mul(-1.0),
                });
            }
        }
    }

    fn variant(&self) -> ShapeVariant {
        ShapeVariant::Rect(*self)
    }

    fn bounds(&self, pos: Vec2) -> (f64, f64, f64, f64) {
        (pos.x - self.hw,
         pos.y - self.hh,
         pos.x + self.hw,
         pos.y + self.hh)
    }

    fn mass(&self, density: f64) -> f64 {
        (self.hw * 2.0) * (self.hh * 2.0) * density
    }

    fn contains(&self, pos: Vec2, point: Vec2) -> bool {
        let (x1, y1, x2, y2) = self.bounds(pos);
        x1 <= point.x && point.x <= x2 && y1 <= point.y && point.y <= y2
    }

    fn cast_ray(&self, pos: Vec2, origin: Vec2, dir: Vec2) -> Option<Vec2> {
        let mut lowest_norm = f64::MAX;
        let mut closest_inter: Option<Vec2> = Option::None;

        let (x1, y1, x2, y2) = self.bounds(pos);

        let borders = [(Vec2::new(x1, y1), Vec2::new(0.0, y2 - y1)),
                       (Vec2::new(x1, y2), Vec2::new(x2 - x1, 0.0)),
                       (Vec2::new(x2, y1), Vec2::new(0.0, y2 - y1)),
                       (Vec2::new(x1, y1), Vec2::new(x2 - x1, 0.0))];

        for &(borigin, bdir) in &borders {
            let inter = lines_intersect(origin, dir, borigin, bdir);
            match inter {
                Option::Some(v) => {
                    let norm = v.norm();
                    if norm < lowest_norm {
                        lowest_norm = norm;
                        closest_inter = inter;
                    }
                }
                None => {}
            }
        }

        closest_inter
    }
}

pub fn lines_intersect(p1: Vec2, d1: Vec2, p2: Vec2, d2: Vec2) -> Option<Vec2> {
    let (x00, y00) = p1.coords();
    let (x01, y01) = d1.coords();
    let (x10, y10) = p2.coords();
    let (x11, y11) = d2.coords();

    let d = x11 * y01 - x01 * y11;

    // let s = (1.0/d) * ((x00 - x10) * y01 - (y00 - y10) * x01);
    let t = (1.0 / d) * -(-(x00 - x10) * y11 + (y00 - y10) * x11);

    if t >= 0.0 && t <= 1.0 {
        Option::Some(p1 + d1.mul(t))
    } else {
        Option::None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use physics::world::Vec2;

    #[test]
    fn test_lines_intersect() {
        assert_eq!(lines_intersect(Vec2::new(0.0, 0.0),
                                   Vec2::new(5.0, 5.0),
                                   Vec2::new(2.0, 0.0),
                                   Vec2::new(0.0, 6.0))
                       .unwrap()
                       .coords(),
                   (2.0, 2.0));
    }
}
