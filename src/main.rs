use std::process::exit;

use macroquad::prelude::*;
use video::SimpleEncoder;

mod video;

pub mod colors {
    use lazy_static::lazy_static;
    use macroquad::color::Color;

    lazy_static! {
        pub static ref BG: Color = Color::from_rgba(39, 55, 77, 255);
        pub static ref SECONDARY: Color = Color::from_rgba(82, 109, 130, 255);
        pub static ref PRIMARY: Color = Color::from_rgba(157, 178, 191, 255);
        pub static ref HIGHLIGHT: Color = Color::from_rgba(221, 230, 237, 255);
    }
}

trait Function {
    fn f(&self, x: f32) -> f32;
    fn f_der(&self, x: f32) -> f32;
}

struct Polynomial {
    components: Vec<(f32, i32)>,
}

impl Polynomial {
    fn new(components: Vec<(f32, i32)>) -> Self {
        Self { components }
    }

    fn line(b: f32, a: f32) -> Self {
        Self {
            components: vec![(b, 0), (a, 1)],
        }
    }

    fn parabola(c: f32, b: f32, a: f32) -> Self {
        Self {
            components: vec![(c, 0), (b, 1), (a, 2)],
        }
    }

    fn parabola_p(center: Vec2, a: f32) -> Self {
        Self::parabola(center.x.powi(2) + center.y, -2.0 * center.x, a)
    }

    fn nth(coefs: impl IntoIterator<Item = f32>) -> Self {
        Self {
            components: coefs
                .into_iter()
                .enumerate()
                .filter(|(_, x)| *x != 0.0)
                .map(|(p, x)| (x, p as i32))
                .collect(),
        }
    }
}

impl Function for Polynomial {
    fn f(&self, x: f32) -> f32 {
        self.components
            .iter()
            .fold(0.0, |acc, (a, p)| acc + a * x.powi(*p))
    }

    fn f_der(&self, x: f32) -> f32 {
        self.components
            .iter()
            .fold(0.0, |acc, (a, p)| acc + a * (*p as f32) * x.powi(p - 1))
    }
}

struct SemiCircle {
    center: Vec2,
    r: f32,
}

impl SemiCircle {
    fn new(center: Vec2, r: f32) -> Self {
        Self { center, r }
    }

    fn base(&self, x: f32) -> f32 {
        (self.r.powi(2) - (x - self.center.x).powi(2)).sqrt()
    }
}

impl Function for SemiCircle {
    fn f(&self, x: f32) -> f32 {
        if (x - self.center.x).abs() <= self.r {
            -self.base(x) + self.center.y
        } else {
            (x - self.center.x).abs() + self.center.y - self.r
        }
    }

    fn f_der(&self, x: f32) -> f32 {
        if (x - self.center.x).abs() <= self.r {
            (x - self.center.x) / self.base(x)
        } else if x > self.center.x {
            1.0
        } else {
            -1.0
        }
    }
}

fn draw_line_p(p0: Vec2, p1: Vec2, thickness: f32, color: Color) {
    draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color)
}

fn color_hsv(h: f32, s: f32, v: f32, a: f32) -> Color {
    let range = (h / 60.0) as u8;
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    match range {
        0 => Color::new(c + m, x + m, m, a),
        1 => Color::new(x + m, c + m, m, a),
        2 => Color::new(m, c + m, x + m, a),
        3 => Color::new(m, x + m, c + m, a),
        4 => Color::new(x + m, m, c + m, a),
        _ => Color::new(c + m, m, x + m, a),
    }
}

fn math_to_screen_x(x: f32) -> f32 {
    (x + XDIM / 2.0 - XCENTER) / XDIM * screen_width()
}

fn math_to_screen_y(y: f32) -> f32 {
    (-y + YDIM / 2.0 + YCENTER) / YDIM * screen_height()
}

fn math_to_screen(mut p: Vec2) -> Vec2 {
    p.x = math_to_screen_x(p.x);
    p.y = math_to_screen_y(p.y);
    p
}

fn screen_to_math_x(x: f32) -> f32 {
    (x / screen_width()) * XDIM - XDIM / 2.0 + XCENTER
}

fn screen_to_math_y(y: f32) -> f32 {
    -((y / screen_height()) * YDIM - YDIM / 2.0) - YCENTER
}

fn screen_to_math(mut p: Vec2) -> Vec2 {
    p.x = screen_to_math_x(p.x);
    p.y = screen_to_math_y(p.y);
    p
}

fn draw_axes() {
    let p0 = math_to_screen(vec2(0.0, -YDIM - YCENTER.abs() - 1.0));
    let p1 = math_to_screen(vec2(0.0, YDIM + YCENTER.abs() + 1.0));
    draw_line_p(p0, p1, 1.0, *colors::SECONDARY);
    let p0 = math_to_screen(vec2(-XDIM - XCENTER.abs() - 1.0, 0.0));
    let p1 = math_to_screen(vec2(XDIM + XCENTER.abs() + 1.0, 0.0));
    draw_line_p(p0, p1, 1.0, *colors::SECONDARY);
}

fn draw_f(f: &impl Function) {
    let mut start = vec2(0.0, 0.0);
    for i in -1..(screen_width() as i32 + 1) {
        let x = screen_to_math_x(i as f32);
        let y = f.f(x);
        let end = math_to_screen(vec2(x, y));
        draw_line_p(start, end, 1.0, *colors::PRIMARY);
        let mut c = *colors::PRIMARY;
        c.a = 0.5;
        draw_line_p(start, end, 2.0, c);

        // let tan = vec2(1.0, f_der(x)).normalize().perp();
        // draw_line_p(
        //     math_to_screen(vec2(x, y)),
        //     math_to_screen(vec2(x, y) + tan.normalize_or_zero() / 10.0),
        //     1.0,
        //     c,
        // );
        start = end;
    }
}

#[derive(Default)]
struct Ball {
    pos: Vec2,
    speed: Vec2,
    color: Color,
    start_y: f32,
}

type Balls = [Ball];

fn get_new(pos: Vec2, speed: Vec2, acc: Vec2, dt: f32) -> (Vec2, Vec2) {
    let nspeed = speed + acc * dt;
    let npos = pos + nspeed * dt + acc * dt * dt / 2.0;
    (nspeed, npos)
}

const EPS: f32 = 0.0001;

fn fix_energy(pos: Vec2, speed: Vec2, start_y: f32, acc: Vec2) -> Vec2 {
    let potential = (pos.y - start_y) * acc.length();
    let speed_len = speed.length();
    let kinetic = speed_len * speed_len / 2.0;
    if kinetic + potential < 0.0 {
        let target = (2.0 * -potential).sqrt();
        speed * target / speed_len
    } else {
        speed
    }
}

fn process_single(
    f: &impl Function,
    pos: Vec2,
    speed: Vec2,
    acc: Vec2,
    dt: f32,
) -> (Vec2, Vec2, f32) {
    let (nspeed, npos) = get_new(pos, speed, acc, dt);
    if f.f(npos.x) <= npos.y {
        return (npos, nspeed, 0.0);
    }

    let mut l = 0.0;
    let mut r = 1.0;
    while r - l > EPS {
        let mid = (l + r) / 2.0;
        let (_, tpos) = get_new(pos, speed, acc, dt * mid);
        if f.f(tpos.x) > tpos.y {
            r = mid;
        } else {
            l = mid;
        }
    }
    let (tspeed, tpos) = get_new(pos, speed, acc, dt * l);
    let tan = vec2(1.0, f.f_der(tpos.x)).normalize().perp();
    let tspeed = tspeed - 2.0 * tspeed.dot(tan) * tan;
    (tpos, tspeed, dt * (1.0 - l))
}

fn update_balls(f: &impl Function, balls: &mut Balls, dt: f32) {
    const ACC: Vec2 = Vec2 { x: 0.0, y: -0.01 };
    for b in balls.iter_mut() {
        let mut npos = b.pos;
        let mut nspeed = b.speed;
        let mut ndt = dt;
        // hard limit to 10 processes
        for _ in 0..10 {
            (npos, nspeed, ndt) = process_single(f, npos, nspeed, ACC, ndt);
            if dt > 0.0 {
                nspeed = fix_energy(npos, nspeed, b.start_y, ACC);
            }
            if ndt / dt < EPS {
                break;
            }
        }
        b.pos = npos;
        b.speed = nspeed;
    }
}

fn draw_balls(balls: &Balls) {
    for b in balls.iter() {
        let s = math_to_screen(b.pos);
        draw_circle(s.x, s.y, 5.0, b.color);
        if false {
            draw_line_p(
                s,
                math_to_screen(b.pos + b.speed.normalize_or_zero() / 10.0),
                1.0,
                b.color,
            );
        }
    }
}

// fn init_balls(f: &impl Function, start: f32, shift: f32) -> Vec<Ball> {
//     let start = vec2(start, f.f(start) + shift);
//     let from = -500;
//     let to = 501;
//     (from..to)
//         .map(|i| {
//             let pos = start + vec2(0.0001, 0.0) * i as f32;
//             let color = color_hsv(
//                 (i - from) as f32 / (to - from) as f32 * 350.0,
//                 1.0,
//                 1.0,
//                 1.0,
//             );
//             (pos, color)
//         })
//         .map(|(pos, color)| Ball {
//             pos,
//             color,
//             start_y: pos.y,
//             ..Default::default()
//         })
//         .collect()
// }

fn init_balls(f: &impl Function, start: f32, shift: f32) -> Vec<Ball> {
    let mut balls = Vec::default();
    for x in (0..WINDOW_WIDTH / 2).step_by(10) {
        for y in (0..WINDOW_HEIGHT).step_by(10) {
            let pos = screen_to_math(vec2(x as f32, y as f32));
            if f.f(pos.x) < pos.y {
                balls.push(Ball {
                    pos,
                    color: color_hsv(x as f32 / 540.0 * 360.0, y as f32 / 1080.0, 1.0, 1.0),
                    start_y: pos.y,
                    ..Default::default()
                })
            }
        }
    }
    balls
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ballz".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

const XDIM: f32 = 10.0;
const XCENTER: f32 = 0.0;
const YDIM: f32 = 10.0;
const YCENTER: f32 = 0.0;
const WINDOW_WIDTH: u32 = 1080;
const WINDOW_HEIGHT: u32 = 1080;

#[macroquad::main(window_conf)]
async fn main() {
    let f = Polynomial::nth(vec![-2.0, 0.0, -1.0, 0.0, 0.1]);
    // let f = Polynomial::parabola_p(vec2(0.0, -2.0), 0.1);
    // let f = SemiCircle::new(vec2(0.0, -0.0), 4.0);
    let mut balls = init_balls(&f, XCENTER - XDIM * 2.5 / 8.0, 3.0);

    const START_DT: f32 = 0.002;
    let mut dt: f32 = START_DT;

    let render_target = render_target(WINDOW_WIDTH, WINDOW_HEIGHT);
    render_target.texture.set_filter(FilterMode::Linear);
    let mut camera = Camera2D::from_display_rect(Rect::new(
        0.0,
        screen_height(),
        screen_width(),
        -screen_height(),
    ));
    camera.render_target = Some(render_target);

    let mut encoder = Some(SimpleEncoder::new("output.mp4", 1080, 1080, 30).unwrap());
    // let mut encoder: Option<SimpleEncoder> = None;

    loop {
        set_camera(&camera);

        clear_background(*colors::BG);
        // draw_axes();
        draw_f(&f);

        if is_key_pressed(KeyCode::Right) {
            dt *= 2.0;
        } else if is_key_pressed(KeyCode::Left) {
            dt /= 2.0;
        }
        dt = dt.clamp(START_DT / 8.0, START_DT * 8.0);
        for _ in 0..100 {
            update_balls(&f, &mut balls, dt);
        }
        draw_balls(&balls);

        set_default_camera();

        if let Some(e) = &mut encoder {
            let image = render_target.texture.get_texture_data();
            let still_rendering = e.render(&image.bytes).unwrap();
            if !still_rendering {
                let encoder = encoder.take().unwrap();
                encoder.done().unwrap();
                println!("Rendering done");
            }
        }

        clear_background(BLACK);
        draw_texture_ex(
            render_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        draw_text(
            &format!("FPS: {}", get_fps()),
            10.0,
            20.0,
            20.0,
            *colors::PRIMARY,
        );

        next_frame().await
    }
}
