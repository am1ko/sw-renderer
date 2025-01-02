extern crate nalgebra as na;
extern crate obj;
extern crate renderer;
extern crate minifb;

use na::{Vector3, Vector4};
use renderer::*;
use std::env;
use obj::*;
use std::fs::File;
use std::io::BufReader;
use minifb::{Key, Window, WindowOptions};

const FPS: usize = 60;
const WIN_WIDTH: usize = 800;
const WIN_HEIGHT: usize = 600;

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
            position: Vector4::new(-side_len/2.0, 0.0, 0.0, 1.0),
            color: green,
            normal: Vector3::new(0.0, 0.0, 1.0),
        },
        v2: renderer::core::Vertex {
            position: Vector4::new(side_len/2.0, 0.0, 0.0, 1.0),
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

    model.translate(Vector3::new(0.0, 0.0, -6.0));

    let eye_pos = Vector3::new(0.0, 0.0, 0.0);
    let mut _vel = Vector3::new(0.0, 0.0, 0.0);
    let mut db = core::DisplayBuffer::new(WIN_WIDTH as usize, WIN_HEIGHT as usize, 4);
    let mut _mouselook_enabled = false;
    let lookat = Vector3::new(0.0, 0.0, -1.0);
    let mut buffer: Vec<u32> = vec![0; WIN_WIDTH * WIN_HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIN_WIDTH,
        WIN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(FPS);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        db.clear();
        model.render(eye_pos, lookat, &mut db);

        for i in 0..WIN_WIDTH {
            for j in 0..WIN_HEIGHT {
                let idx = (i + j * WIN_WIDTH) * 4;
                let color = core::Color {
                    r: db.data[idx],
                    g: db.data[idx + 1],
                    b: db.data[idx + 2],
                    a: db.data[idx + 3],
                };
                buffer[i + j * WIN_WIDTH] = color.to_u32();
            }
        }

        window
            .update_with_buffer(&buffer, WIN_WIDTH, WIN_HEIGHT)
            .unwrap();
    }

}
