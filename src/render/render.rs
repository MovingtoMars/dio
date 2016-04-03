extern crate piston_window;

use piston_window::*;

use engine::world::World;
use interface::camera::Camera;
use media;

pub fn render(win: &PistonWindow, cam: &Camera, world: &mut World) {
    win.draw_2d(|c, g| {
        clear([0.0; 4], g);

        let (zx, zy) = cam.pos_to_screen(0.0, 0.0);
        let (w, h) = cam.pair_metres_to_pixels(world.data.get_width(), world.data.get_height());
        rectangle([1.0; 4], [zx, zy, w, h], c.transform, g);
    });

    for e in world.get_entities_ref() {
        e.borrow_mut().render(&world.data.physics_world, win, cam);
    }
}

// arrays are in [x, y, w, h] format
pub fn render_image(win: &PistonWindow, cam: &Camera, image_tex: &media::image::ImageHandle, target: [f32; 4], source: Option<[i32; 4]>) {
    let image_bounds = Image {
        color: None,
        rectangle: Some(cam.array_pos_to_screen(target)),
        source_rectangle: source,
    };

    win.draw_2d(|c, g| {
        g.image(&image_bounds,
                image_tex.borrow_texture(),
                &c.draw_state,
                c.transform);
    });
}

pub fn fill_rectangle(win: &PistonWindow, cam: &Camera, colour: [f32; 4], x: f32, y: f32, w: f32, h: f32, rot: f32) {
    let (x, y, w, h) = {
        (x + w / 2.0, y + h / 2.0, w, h)
    };

    win.draw_2d(|c, g| {
        let (zx, zy) = cam.pos_to_screen(x, y);
        let (w, h) = cam.pair_metres_to_pixels(w, h);
        let rect = [0.0, 0.0, w, h];
        rectangle(colour, rect, c.transform.trans(zx, zy).rot_rad(rot as f64).trans(-w / 2.0, -h / 2.0), g);
    });
}
