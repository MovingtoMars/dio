use piston_window::*;
use piston_window::character::CharacterCache;
use specs::Join;

use engine::World;
use engine::{Hitpoints, Name, RenderItem, RenderItemKind, Renderable};
use interface::camera::Camera;
use media::*;

pub fn render(win: &mut PistonWindow, cam: &Camera, world: &mut World, input: &Input, fonts: &mut Fonts) {
    let win_draw_size = win.draw_size();

    win.draw_2d(input, |c, g| {
        clear([0.0; 4], g);

        rectangle(
            [1.0; 4],
            [0.0, 0.0, cam.win_w as f64, cam.win_h as f64],
            c.transform,
            g,
        );
    });

    for (entity, renderable) in (&*world.entities(), &world.read_component::<Renderable>()).join() {
        let x = renderable.x;
        let y = renderable.y;
        let rotation = renderable.rotation;

        for item in &renderable.items {
            let &RenderItem {
                rel_x,
                rel_y,
                rel_rotation,
                color,
                ..
            } = item;

            if rel_rotation != 0.0 {
                eprintln!("Relative rendering rotations don't work yet!");
            }

            let abs_x = x + rel_x;
            let abs_y = y + rel_y;

            match item.kind {
                RenderItemKind::Rectangle { w, h } => {
                    fill_rectangle(win, input, cam, color, abs_x, abs_y, w, h, rotation);
                }
                RenderItemKind::Text { ref text, size } => {
                    draw_text(
                        win,
                        input,
                        cam,
                        fonts,
                        color,
                        abs_x,
                        abs_y,
                        size,
                        rotation,
                        x,
                        y,
                        &text,
                    );
                }
                RenderItemKind::Info => {
                    let hitpointsc = world.read_component::<Hitpoints>();
                    let hp = hitpointsc.get(entity);

                    let namec = world.read_component::<Name>();
                    let name = namec.get(entity);

                    let mut abs_y = abs_y;

                    if let Some(hp) = hp {
                        draw_text(
                            win,
                            input,
                            cam,
                            fonts,
                            color,
                            abs_x,
                            abs_y,
                            14,
                            rotation,
                            x,
                            y,
                            &format!("{}/{}", hp.current(), hp.max()),
                        );
                        abs_y -= cam.pixels_to_metres(16.0);
                    }

                    if let Some(name) = name {
                        draw_text(
                            win,
                            input,
                            cam,
                            fonts,
                            color,
                            abs_x,
                            abs_y,
                            14,
                            rotation,
                            x,
                            y,
                            &format!("{}", name.0),
                        );
                    }
                }
            }
        }
    }

    if let Some(time_stop_remaining) = world.time_stop_remaining() {
        let win_draw_size = win.draw_size();
        let width = time_stop_remaining as f64 / 5.0 * 0.2 * win_draw_size.width as f64;

        win.draw_2d(input, |c, g| {
            rectangle(
                [0.5, 0.7, 1.0, 1.0],
                [20.0, 20.0, width, 20.0],
                c.transform,
                g,
            );
        });
    }

    let player = world.clone_player_component();
    let knives_text = &format!(
        "Knives: {}/{}",
        player.num_knives(),
        player.max_num_knives()
    );
    let width = fonts.bold.glyphs.width(16, knives_text);

    win.draw_2d(input, |c, g| {
        text(
            [0.0, 0.0, 0.0, 1.0],
            18,
            knives_text,
            &mut fonts.bold.glyphs,
            c.transform.trans(20.0, win_draw_size.height as f64 - 20.0),
            g,
        );
    });
}

pub struct Fonts {
    pub regular: FontHandle,
    pub bold: FontHandle,
}

impl Fonts {
    pub fn new(media: &MediaHandle) -> Self {
        let regular = FontHandle::new(media, "NotoSans-unhinted/NotoSans-Regular.ttf");
        let bold = FontHandle::new(media, "NotoSans-unhinted/NotoSans-Bold.ttf");
        Fonts { regular, bold }
    }
}

// arrays are in [x, y, w, h] format
pub fn render_image(win: &mut PistonWindow, input: &Input, cam: &Camera, image_tex: &ImageHandle, target: [f32; 4], source: Option<[f64; 4]>) {
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

// TODO support for origin coords
pub fn fill_rectangle(win: &mut PistonWindow, input: &Input, cam: &Camera, colour: [f32; 4], cx: f32, cy: f32, w: f32, h: f32, rot: f32) {
    // let (x, y, w, h) = { (x + w / 2.0, y + h / 2.0, w, h) };

    win.draw_2d(input, |c, g| {
        let (zx, zy) = cam.pos_to_screen(cx, cy);
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

// TODO make inputs a struct
/// origin coords are used as origin for rotation
pub fn draw_text(
    win: &mut PistonWindow,
    input: &Input,
    cam: &Camera,
    fonts: &mut Fonts,
    colour: [f32; 4],
    cx: f32,
    cy: f32,
    size: u32,
    rot: f32,
    origin_x: f32,
    origin_y: f32,
    text_: &str,
) {
    win.draw_2d(input, |c, g| {
        let (zx, zy) = cam.pos_to_screen(cx, cy);
        let (origin_zx, origin_zy) = cam.pos_to_screen(origin_x, origin_y);
        let width = fonts.bold.glyphs.width(size, text_);

        text(
            colour,
            size,
            text_,
            &mut fonts.bold.glyphs,
            c.transform
                .trans(origin_zx, origin_zy)
                .rot_rad(rot as f64)
                .trans(zx - origin_zx, zy - origin_zy)
                .trans(-width / 2.0, 0.0),
            g,
        );
    });
}
