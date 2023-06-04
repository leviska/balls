use macroquad::prelude::*;

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

const XDIM: f32 = 3.0;
const XCENTER: f32 = 0.0;
const YDIM: f32 = 3.0;
const YCENTER: f32 = 0.0;

fn f(x: f32) -> f32 {
    x.powi(2) - 1.0
}

fn f_der(x: f32) -> f32 {
    2.0 * x
}

fn draw_line_p(p0: Vec2, p1: Vec2, thickness: f32, color: Color) {
    draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color)
}

fn color_hsv(h: f32, s: f32, v: f32) -> Color {
    let range = (h / 60.0) as u8;
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    match range {
        0 => Color::new((c + m), (x + m), m, 1.0),
        1 => Color::new((x + m), (c + m), m, 1.0),
        2 => Color::new(m, (c + m), (x + m), 1.0),
        3 => Color::new(m, (x + m), (c + m), 1.0),
        4 => Color::new((x + m), m, (c + m), 1.0),
        _ => Color::new((c + m), m, (x + m), 1.0),
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

fn draw_f() {
    let mut start = vec2(0.0, 0.0);
    for i in -1..(screen_width() as i32 + 1) {
        let x = screen_to_math_x(i as f32);
        let y = f(x);
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
}

type Balls = [Ball];

fn update_balls(balls: &mut Balls, dt: f32) {
    const STEPS: i32 = 1000;
    let dt = dt / STEPS as f32;
    for b in balls.iter_mut() {
        for _ in 0..STEPS {
            b.speed.y -= 0.01 * dt;
            let mut speed = b.speed;
            let mut pos = b.pos + b.speed * dt;
            if f(pos.x) > pos.y {
                let mut l = 0.0;
                let mut r = 1.0;
                while r - l > 0.001 {
                    let mid = (l + r) / 2.0;
                    let npos = b.pos + b.speed * dt * mid;
                    if f(npos.x) > npos.y {
                        r = mid;
                    } else {
                        l = mid;
                    }
                }
                pos = b.pos + b.speed * dt * l;
                let tan = vec2(1.0, f_der(pos.x)).normalize().perp();
                speed = speed - 2.0 * speed.dot(tan) * tan;
            }
            b.pos = pos;
            b.speed = speed;
        }
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

fn init_balls() -> Vec<Ball> {
    let start = vec2(-0.8, 1.0);
    let shift = 0.005;
    (0..100)
        .into_iter()
        .map(|i| {
            let mut pos = vec2(start.x + shift * i as f32, start.y);
            pos.y = pos.y;
            Ball {
                pos,
                color: Color::from_rgba(255 - i, i + 155, 0, 255),
                ..Default::default()
            }
        })
        .collect()
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Ballz".to_owned(),
        window_width: 1080,
        window_height: 1080,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut balls = init_balls();
    loop {
        clear_background(*colors::BG);

        draw_axes();
        draw_f();

        update_balls(&mut balls, 0.1);
        draw_balls(&balls);

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
