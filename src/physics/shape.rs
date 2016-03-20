use std::f64;

use physics::world::Vec2;

pub trait Shape {
    fn bounds(&self, x: f64, y: f64) -> (f64, f64, f64, f64);
    fn cast_ray(&self, x: f64, y: f64, origin: Vec2, dir: Vec2) -> Option<Vec2>;
    fn contains(&self, x: f64, y: f64, point: Vec2) -> bool;
    fn mass(&self, density: f64) -> f64;
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
    fn bounds(&self, x: f64, y: f64) -> (f64, f64, f64, f64) {
        (x - self.hw, y - self.hh, x + self.hw, y + self.hh)
    }

    fn mass(&self, density: f64) -> f64 {
        (self.hw * 2.0) * (self.hh * 2.0) * density
    }

    fn contains(&self, x: f64, y: f64, point: Vec2) -> bool {
        let (x1, y1, x2, y2) = self.bounds(x, y);
        x1 <= point.x && point.x <= x1 + x2 && y1 <= point.y && point.y <= y1 + y2
    }

    fn cast_ray(&self, x: f64, y: f64, origin: Vec2, dir: Vec2) -> Option<Vec2> {
        let mut lowest_norm = f64::MAX;
        let mut closest_inter: Option<Vec2> = Option::None;

        let (x1, y1, x2, y2) = self.bounds(x, y);

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
