extern crate nalgebra as na;
extern crate obj;
extern crate renderer;
extern crate sfml;

use na::{Vector3, Vector4};
use renderer::*;
use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::system::Clock;
use sfml::window::{Event, Key, Style, VideoMode};
use std::env;

use obj::*;
use std::fs::File;
use std::io::BufReader;

const FPS: f32 = 60.0;
const WIN_WIDTH: usize = 1920;
const WIN_HEIGHT: usize = 1080;

fn load_model_from_file(file_name: &String) -> core::Mesh {
    let mut model = core::Mesh::new();
    let f = match File::open(file_name) {
        Ok(v) => v,
        Err(_e) => {
            println!("Error: Could not open file {}", file_name);
            return model;
        }
    };

    let input = BufReader::new(f);
    let obj: Obj = load_obj(input).unwrap();

    let mut f = 0;
    while f < obj.indices.len() {
        assert!(f + 2 < obj.indices.len());
        let white = renderer::core::Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };

        let i = obj.indices[f] as usize;
        let j = obj.indices[f + 1] as usize;
        let k = obj.indices[f + 2] as usize;

        model.faces.push(core::Face {
            v0: renderer::core::Vertex {
                position: Vector4::new(
                    obj.vertices[i].position[0],
                    obj.vertices[i].position[1],
                    obj.vertices[i].position[2],
                    1.0,
                ),
                color: white,
                normal: Vector3::new(
                    obj.vertices[i].normal[0],
                    obj.vertices[i].normal[1],
                    obj.vertices[i].normal[2],
                ),
            },
            v1: renderer::core::Vertex {
                position: Vector4::new(
                    obj.vertices[j].position[0],
                    obj.vertices[j].position[1],
                    obj.vertices[j].position[2],
                    1.0,
                ),
                color: white,
                normal: Vector3::new(
                    obj.vertices[j].normal[0],
                    obj.vertices[j].normal[1],
                    obj.vertices[j].normal[2],
                ),
            },
            v2: renderer::core::Vertex {
                position: Vector4::new(
                    obj.vertices[k].position[0],
                    obj.vertices[k].position[1],
                    obj.vertices[k].position[2],
                    1.0,
                ),
                color: white,
                normal: Vector3::new(
                    obj.vertices[k].normal[0],
                    obj.vertices[k].normal[1],
                    obj.vertices[k].normal[2],
                ),
            },
        });

        f = f + 3;
    }

    return model;
}

fn load_default_model() -> core::Mesh {
    let mut model = core::Mesh::new();
    let red = renderer::core::Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    let green = renderer::core::Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    let blue = renderer::core::Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    let side_len = 1.0;

    model.faces.push(core::Face {
        v0: renderer::core::Vertex {
            position: Vector4::new(0.0, side_len, 0.0, 1.0),
            color: red,
            normal: Vector3::new(0.0, 0.0, 1.0),
        },
        v1: renderer::core::Vertex {
            position: Vector4::new(-side_len, 0.0, 0.0, 1.0),
            color: green,
            normal: Vector3::new(0.0, 0.0, 1.0),
        },
        v2: renderer::core::Vertex {
            position: Vector4::new(side_len, 0.0, 0.0, 1.0),
            color: blue,
            normal: Vector3::new(0.0, 0.0, 1.0),
        },
    });

    return model;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut model = if args.len() == 2 {
        load_model_from_file(&args[1])
    } else {
        println!("Usage: renderer [FILE]");
        println!("No model file given. Loading default model");
        load_default_model()
    };

    let mut clock = Clock::start();
    let vm = VideoMode::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, 32);
    let mut window = RenderWindow::new(vm, "sw-renderer", Style::CLOSE, &Default::default());
    window.set_vertical_sync_enabled(true);

    model.translate(Vector3::new(0.0, 0.0, -6.0));

    let mut eye_pos = Vector3::new(0.0, 0.0, 0.0);
    let mut vel = Vector3::new(0.0, 0.0, 0.0);
    let mut db = core::DisplayBuffer::new(WIN_WIDTH, WIN_HEIGHT, 4);
    let mut texture = Texture::new(WIN_WIDTH as u32, WIN_HEIGHT as u32).unwrap();
    let mut mouselook_enabled = false;

    loop {
        let mut sprite = Sprite::new();

        loop {
            let event = match window.poll_event() {
                Some(val) => val,
                None => break,
            };

            match event {
                Event::Closed
                | Event::KeyPressed { code: Key::Q, .. }
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => return,
                Event::KeyPressed { code: Key::D, .. }
                | Event::KeyPressed {
                    code: Key::Right, ..
                }
                | Event::KeyPressed { code: Key::L, .. } => {
                    vel.x = 10.0;
                }
                Event::KeyReleased { code: Key::D, .. }
                | Event::KeyReleased {
                    code: Key::Right, ..
                }
                | Event::KeyReleased { code: Key::L, .. } => {
                    vel.x = 0.0;
                }
                Event::KeyPressed { code: Key::A, .. }
                | Event::KeyPressed {
                    code: Key::Left, ..
                }
                | Event::KeyPressed { code: Key::H, .. } => {
                    vel.x = -10.0;
                }

                Event::KeyReleased { code: Key::A, .. }
                | Event::KeyReleased {
                    code: Key::Left, ..
                }
                | Event::KeyReleased { code: Key::H, .. } => {
                    vel.x = 0.0;
                }

                Event::KeyPressed { code: Key::W, .. }
                | Event::KeyPressed { code: Key::Up, .. }
                | Event::KeyPressed { code: Key::K, .. } => {
                    vel.z = -1.0;
                }
                Event::KeyReleased { code: Key::W, .. }
                | Event::KeyReleased { code: Key::Up, .. }
                | Event::KeyReleased { code: Key::K, .. } => {
                    vel.z = 0.0;
                }
                Event::KeyPressed { code: Key::S, .. }
                | Event::KeyPressed {
                    code: Key::Down, ..
                }
                | Event::KeyPressed { code: Key::J, .. } => {
                    vel.z = 1.0;
                }
                Event::KeyReleased { code: Key::S, .. }
                | Event::KeyReleased {
                    code: Key::Down, ..
                }
                | Event::KeyReleased { code: Key::J, .. } => {
                    vel.z = 0.0;
                }
                Event::KeyPressed { code: Key::M, .. } => {
                    mouselook_enabled = true;
                }
                Event::KeyPressed { code: Key::R, .. } => {
                    model.rotate(Vector3::new(0.0, 0.01, 0.0));
                }
                _ => {}
            }
        }

        eye_pos.x = eye_pos.x + vel.x * (1.0 / FPS);
        eye_pos.y = eye_pos.y + vel.y * (1.0 / FPS);
        eye_pos.z = eye_pos.z + vel.z * (1.0 / FPS);

        let (lookat_x, lookat_y) = if mouselook_enabled {
            let mouse_x = window.mouse_position().x;
            let mouse_y = WIN_HEIGHT as i32 - window.mouse_position().y;
            (
                (mouse_x as f32 - ((WIN_WIDTH / 2) as f32)) / (WIN_WIDTH as f32),
                (mouse_y as f32 - ((WIN_HEIGHT / 2) as f32)) / (WIN_HEIGHT as f32),
            )
        } else {
            (0.0, 0.0)
        };

        let lookat = Vector3::new(lookat_x as f32, lookat_y as f32, -1.0);

        db.clear();

        model.render(eye_pos, lookat, &mut db);

        if clock.elapsed_time().as_seconds() > 1.0 / FPS {
            clock.restart();
            unsafe {
                texture.update_from_pixels(&*db.data, db.width as u32, db.height as u32, 0, 0);
            }
            sprite.set_texture(&texture, false);

            window.clear(Color::BLACK);
            window.draw(&sprite);
            window.display();
        }
    }
}
