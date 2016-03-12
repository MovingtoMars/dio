extern crate piston_window;

use piston_window::*;

use engine::world::World;
use interface::camera::Camera;

pub fn render(win: &PistonWindow, cam: &Camera, world: &mut World) {
    win.draw_2d(|c, g| {
        clear([0.0; 4], g);

        let (zx, zy) = cam.pos_to_screen(win.draw_size(), 0.0, 0.0);
        let (w, h) = cam.pair_metres_to_pixels(world.data.get_width(), world.data.get_height());
        rectangle([1.0; 4],
                [zx, zy, w, h],
                c.transform, g);
    });

    for e in world.get_entities_ref() {
        e.render(&world.data.b2world, win, cam);
    }
}


pub fn fill_rectangle(win: &PistonWindow, cam: &Camera, x: f32, y: f32, w: f32, h: f32) {
    win.draw_2d(|c, g| {
        let (zx, zy) = cam.pos_to_screen(win.draw_size(), x, y);
        let (w, h) = cam.pair_metres_to_pixels(w, h);
        rectangle([0.0, 1.0, 0.0, 1.0],
                [zx, zy, w, h],
                c.transform, g);
    });
}
