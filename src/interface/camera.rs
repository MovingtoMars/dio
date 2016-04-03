extern crate piston_window;

pub struct Camera {
    // the position of the world which is at the centre of the screen (in metres)
    pub x: f32,
    pub y: f32,

    // the position in the window where the mouse pointer is
    pub mouse_x: f64,
    pub mouse_y: f64,

    pub pixels_per_metre: f64,
}

impl Camera {
    pub fn new(x: f32, y: f32, pixels_per_metre: f64) -> Camera {
        Camera {
            x: x,
            y: y,
            mouse_x: 0.0, // TODO ???
            mouse_y: 0.0,
            pixels_per_metre: pixels_per_metre,
        }
    }

    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn metres_to_pixels(&self, val: f32) -> f64 {
        val as f64 * self.pixels_per_metre
    }

    pub fn pixels_to_metres(&self, val: f64) -> f32 {
        (val / self.pixels_per_metre) as f32
    }

    pub fn pair_metres_to_pixels(&self, x: f32, y: f32) -> (f64, f64) {
        (self.metres_to_pixels(x), self.metres_to_pixels(y))
    }

    pub fn pair_pixels_to_metres(&self, x: f64, y: f64) -> (f32, f32) {
        (self.pixels_to_metres(x), self.pixels_to_metres(y))
    }

    pub fn pos_to_screen(&self, screen_size: piston_window::Size, x: f32, y: f32) -> (f64, f64) {
        let (px, py) = self.pair_metres_to_pixels(x - self.x, y - self.y);
        (px + (screen_size.width / 2) as f64,
         py + (screen_size.height / 2) as f64)
    }

    pub fn screen_to_pos(&self, x: f64, y: f64) -> (f32, f32) {
        let (wx, wy) = self.pair_pixels_to_metres(x, y);
        (wx + self.x, wy + self.y)
    }

    pub fn array_pos_to_screen(&self, screen_size: piston_window::Size, pos: [f32; 4]) -> [f64; 4] {
        let mut npos = [0.0; 4];
        npos[0] = self.metres_to_pixels(pos[0] - self.x) + (screen_size.width / 2) as f64;
        npos[1] = self.metres_to_pixels(pos[1] - self.y) + (screen_size.height / 2) as f64;
        npos[2] = self.metres_to_pixels(pos[2]);
        npos[3] = self.metres_to_pixels(pos[3]);

        npos
    }
}
