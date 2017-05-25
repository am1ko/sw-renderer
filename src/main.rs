extern crate sfml;
mod vector;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::window::{Event, Key, style, VideoMode};
use sfml::system::Clock;

use vector::{Vector2, Vector3};

const WIN_WIDTH: usize = 800;
const WIN_HEIGHT: usize = 600;
const BYTES_PER_PIXEL: usize = 4;
const FPS: f32 = 60.0;

pub struct Mesh{
    vertices: Vec<Vector3>
    // rotation
    // position
}

impl Mesh {
    pub fn new() -> Mesh {
        return Mesh{vertices: Vec::new()};
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

fn render_mesh(mesh: &Mesh, pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    for v in mesh.vertices.iter() {
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

fn main() {
    let mut clock = Clock::start();
    let vm = VideoMode::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, 32);
    // let mut window = RenderWindow::new((WIN_WIDTH as u32, WIN_HEIGHT as u32),
    let w = RenderWindow::new(vm, "GFX demo", style::CLOSE, &Default::default());

    let mut window = w.unwrap();
    window.set_vertical_sync_enabled(true);

    let mut texture = Texture::new(WIN_WIDTH as u32, WIN_HEIGHT as u32).unwrap();

    let mut cube = Mesh::new();
    cube.vertices.append(&mut vec![Vector3 {
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
                    }]);

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
                }
                Event::KeyPressed { code: Key::A, .. } |
                Event::KeyPressed { code: Key::Left, .. } |
                Event::KeyPressed { code: Key::H, .. } => {
                }
                Event::KeyPressed { code: Key::W, .. } |
                Event::KeyPressed { code: Key::Up, .. } |
                Event::KeyPressed { code: Key::K, .. } => {
                }
                Event::KeyPressed { code: Key::S, .. } |
                Event::KeyPressed { code: Key::Down, .. } |
                Event::KeyPressed { code: Key::J, .. } => {
                }
                Event::KeyPressed { code: Key::R, .. } => {
                }
                _ => {}
            }
        }

        render_mesh(&cube, &mut display_buffer);

        if clock.elapsed_time().as_seconds() > 1.0 / FPS {
            clock.restart();


            texture.update_from_pixels(&display_buffer, WIN_WIDTH as u32, WIN_HEIGHT as u32, 0, 0);
            sprite.set_texture(&texture, false);

            window.clear(&Color::black());
            window.draw(&sprite);
            window.display();
        }
    }
}
