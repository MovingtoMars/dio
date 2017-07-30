use piston_window::*;

use engine::world::World;
use interface::camera::Camera;
use media;

pub fn render(win: &mut PistonWindow, cam: &Camera, world: &mut World, input: &Input) {
    win.draw_2d(input, |c, g| {
        clear([0.0; 4], g);

        rectangle(
            [1.0; 4],
            [0.0, 0.0, cam.win_w as f64, cam.win_h as f64],
            c.transform,
            g,
        );
    });

    for e in world.get_entities_ref() {
        e.borrow_mut()
            .render(&world.data.physics_world, win, input, cam);
    }
}

// arrays are in [x, y, w, h] format
pub fn render_image(
    win: &mut PistonWindow,
    input: &Input,
    cam: &Camera,
    image_tex: &media::image::ImageHandle,
    target: [f32; 4],
    source: Option<[f64; 4]>,
) {
    let image_bounds = Image {
        color: None,
        rectangle: Some(cam.array_pos_to_screen(target)),
        source_rectangle: source,
    };

    win.draw_2d(input, |c, g| {
        g.image(
            &image_bounds,
            image_tex.borrow_texture(),
            &c.draw_state,
            c.transform,
        );
    });
}

// vr - vertical radius
// hr - horizontal radius
pub fn fill_ellipse(win: &mut PistonWindow, input: &Input, cam: &Camera, colour: [f32; 4], cx: f32, cy: f32, hr: f32, vr: f32) {
    win.draw_2d(input, |c, g| {
        let (cx, cy) = cam.pos_to_screen(cx, cy);
        let (hr, vr) = cam.pair_metres_to_pixels(hr, vr);
        let rect = [cx - hr, cy - vr, hr * 2.0, vr * 2.0];
        let e = Ellipse {
            color: colour,
            border: None,
            resolution: 100,
        };
        g.ellipse(&e, rect, &c.draw_state, c.transform);
    });
}

pub fn fill_rectangle(win: &mut PistonWindow, input: &Input, cam: &Camera, colour: [f32; 4], x: f32, y: f32, w: f32, h: f32, rot: f32) {
    let (x, y, w, h) = { (x + w / 2.0, y + h / 2.0, w, h) };

    win.draw_2d(input, |c, g| {
        let (zx, zy) = cam.pos_to_screen(x, y);
        let (w, h) = cam.pair_metres_to_pixels(w, h);
        let rect = [0.0, 0.0, w, h];
        rectangle(
            colour,
            rect,
            c.transform
                .trans(zx, zy)
                .rot_rad(rot as f64)
                .trans(-w / 2.0, -h / 2.0),
            g,
        );
    });
}
