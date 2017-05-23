extern crate sfml;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::window::{Event, Key, style, VideoMode};
use sfml::system::Clock;
use std::ops::Sub;

const WIN_WIDTH: usize = 800;
const WIN_HEIGHT: usize = 600;
const BYTES_PER_PIXEL: usize = 4;
const FPS: f32 = 60.0;

#[derive(Copy, Clone)]
struct Vector2 {
    x: f32,
    y: f32,
}

#[derive(Copy, Clone)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

fn set_pixel(x: usize,
             y: usize,
             color: u32,
             pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    let index = y * WIN_WIDTH * BYTES_PER_PIXEL + x * BYTES_PER_PIXEL;
    if index > 0 && index < (pixels.len() - BYTES_PER_PIXEL) {
        pixels[index] = ((color & 0x000000FF) >> 0) as u8;
        pixels[index + 1] = ((color & 0x0000FF00) >> 8) as u8;
        pixels[index + 2] = ((color & 0x00FF0000) >> 16) as u8;
        pixels[index + 3] = ((color & 0xFF000000) >> 24) as u8;
    }
}

fn normalize(v: Vector2) -> Vector2 {
    return Vector2 {
        x: (1.0 + v.x) / 2.0,
        y: (1.0 + v.y) / 2.0,
    };
}

fn to_raster_space(v: Vector2) -> Vector2 {
    return Vector2 {
        x: v.x * WIN_WIDTH as f32,
        y: v.y * WIN_HEIGHT as f32,
    };
}

fn project(v: Vector3) -> Vector2 {
    let proj = Vector2 {
        x: v.x / v.z,
        y: v.y / v.z,
    };
    let n = normalize(proj);
    return to_raster_space(n);
}

fn draw_cube(pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    let vertexes = [Vector3 {
                        x: 1.0,
                        y: -1.0,
                        z: -5.0,
                    },
                    Vector3 {
                        x: 1.0,
                        y: -1.0,
                        z: -3.0,
                    },
                    Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: -5.0,
                    },
                    Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: -3.0,
                    },
                    Vector3 {
                        x: -1.0,
                        y: -1.0,
                        z: -5.0,
                    },
                    Vector3 {
                        x: -1.0,
                        y: -1.0,
                        z: -3.0,
                    },
                    Vector3 {
                        x: -1.0,
                        y: 1.0,
                        z: -5.0,
                    },
                    Vector3 {
                        x: -1.0,
                        y: 1.0,
                        z: -3.0,
                    }];

    for v in vertexes.iter() {
        if true {
            let v_proj = Vector2 {
                x: v.x / v.z,
                y: v.y / v.z,
            };
            let n = normalize(v_proj);
            let r = to_raster_space(n);
            set_pixel(r.x as usize, r.y as usize, 0xFFFFFFFF, pixels);
        }
    }
    // let a = project(vertexes[i]);
    // let b = project(vertexes[(i+1) % vertexes.len()]);
    // let c = project(vertexes[(i+2) % vertexes.len()]);
    // draw_vector(b - a, a, pixels);
    // draw_vector(c - b, b, pixels);
    // draw_vector(a - c, c, pixels);
    //
    // for i in 0..vertexes.len()
    // {
    // let a = project(vertexes[i]);
    //
    // for j in 0..vertexes.len()
    // {
    // let b = project(vertexes[(i+1) % vertexes.len()]);
    // draw_vector(b - a, a, pixels);
    // }
    // }
    //
}

fn draw_vector(v: Vector2,
               o: Vector2,
               pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    let mut r = Vector2 {
        x: o.x,
        y: WIN_HEIGHT as f32 - o.y,
    };
    let dv = Vector2 {
        x: v.x / 1000.0,
        y: -1.0 * v.y / 1000.0,
    };

    for _ in 1..1000 {
        if r.y >= 0.0 && r.x >= 0.0 {
            set_pixel(r.x as usize, r.y as usize, 0xFF0000FF, pixels);
            r.x = r.x + dv.x;
            r.y = r.y + dv.y;
        }
    }
}

fn rotate_vec(v: &mut Vector2, a: f32) {
    // transformation
    // unit vector i lands to cos(a)*i + sin(a)*j
    // unit vector j lands to -sin(a)*i + cos(a)*j
    // [cos(a), -sin(a)][x]
    // [sin(a),  cos(a)][y]
    //   ^        ^      ^
    // new i    new j  input

    let new_x = a.cos() * v.x + -1.0 * a.sin() * v.y;
    let new_y = a.sin() * v.x + a.cos() * v.y;
    v.x = new_x;
    v.y = new_y;
}

fn main() {
    let mut clock = Clock::start();
    let vm = VideoMode::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, 32);
    // let mut window = RenderWindow::new((WIN_WIDTH as u32, WIN_HEIGHT as u32),
    let w = RenderWindow::new(vm, "GFX demo", style::CLOSE, &Default::default());

    let mut window = w.unwrap();
    window.set_vertical_sync_enabled(true);

    let mut v = Vector2 { x: 30.0, y: 30.0 };
    let o = Vector2 {
        x: 400.0,
        y: 300.0,
    };
    let mut texture = Texture::new(WIN_WIDTH as u32, WIN_HEIGHT as u32).unwrap();

    loop {
        let mut sprite = Sprite::new();
        let mut display_buffer: [u8; WIN_HEIGHT * WIN_WIDTH * BYTES_PER_PIXEL] =
            [0; WIN_HEIGHT * WIN_WIDTH * BYTES_PER_PIXEL];
        for event in window.events() {
            match event {
                Event::Closed |
                Event::KeyPressed { code: Key::Q, .. } |
                Event::KeyPressed { code: Key::Escape, .. } => return,
                Event::KeyPressed { code: Key::D, .. } |
                Event::KeyPressed { code: Key::Right, .. } |
                Event::KeyPressed { code: Key::L, .. } => {
                    v.x = v.x + 1.0;
                }
                Event::KeyPressed { code: Key::A, .. } |
                Event::KeyPressed { code: Key::Left, .. } |
                Event::KeyPressed { code: Key::H, .. } => {
                    v.x = v.x - 1.0;
                }
                Event::KeyPressed { code: Key::W, .. } |
                Event::KeyPressed { code: Key::Up, .. } |
                Event::KeyPressed { code: Key::K, .. } => {
                    v.y = v.y + 1.0;
                }
                Event::KeyPressed { code: Key::S, .. } |
                Event::KeyPressed { code: Key::Down, .. } |
                Event::KeyPressed { code: Key::J, .. } => {
                    v.y = v.y - 1.0;
                }
                Event::KeyPressed { code: Key::R, .. } => {
                    rotate_vec(&mut v, 0.01);
                }
                _ => {}
            }
        }

        // rotate_vec(&mut v, 0.01);
        draw_cube(&mut display_buffer);

        if clock.elapsed_time().as_seconds() > 1.0 / FPS {
            clock.restart();

            // draw_vector(v, o, &mut display_buffer);

            texture.update_from_pixels(&display_buffer, WIN_WIDTH as u32, WIN_HEIGHT as u32, 0, 0);
            sprite.set_texture(&texture, false);

            window.clear(&Color::black());
            window.draw(&sprite);
            window.display();
        }
    }
}
