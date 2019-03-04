extern crate nalgebra as na;
extern crate renderer;
extern crate sfml;

use na::{Vector3, Vector4};
use renderer::*;
use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::system::Clock;
use sfml::window::{Event, Key, Style, VideoMode};

const FPS: f32 = 60.0;
const WIN_WIDTH: usize = 2560;
const WIN_HEIGHT: usize = 1440;

fn create_cube() -> core::Mesh {
    let mut cube = core::Mesh::new();
    cube.vertices.append(&mut vec![
        Vector4::new(-1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(-1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, -1.0, 1.0, 1.0),
        Vector4::new(-1.0, 1.0, -1.0, 1.0),
        Vector4::new(1.0, 1.0, -1.0, 1.0),
        Vector4::new(1.0, -1.0, -1.0, 1.0),
        Vector4::new(-1.0, -1.0, -1.0, 1.0),
    ]);

    cube.triangle_normals.append(&mut vec![
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -1.0),
        Vector3::new(0.0, 0.0, -1.0),
    ]);

    cube.poly_indices.append(&mut vec![
        [0, 1, 2],
        [1, 2, 3],
        [1, 3, 6],
        [1, 5, 6],
        [0, 1, 4],
        [1, 4, 5],
        [2, 3, 7],
        [3, 6, 7],
        [0, 2, 7],
        [0, 4, 7],
        [4, 5, 6],
        [4, 6, 7],
    ]);
    cube.to_triangles();

    return cube;
}

fn main() {
    let mut clock = Clock::start();
    let vm = VideoMode::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, 32);
    let mut window = RenderWindow::new(vm, "GFX demo", Style::CLOSE, &Default::default());

    window.set_vertical_sync_enabled(true);

    let mut texture = Texture::new(WIN_WIDTH as u32, WIN_HEIGHT as u32).unwrap();

    let mut cube = create_cube();
    let mut cube2 = create_cube();

    cube.translate(Vector3::new(0.0, 0.0, -6.0));
    cube2.translate(Vector3::new(0.0, 0.0, -24.0));

    let mut eye_pos = Vector3::new(0.0, 0.0, 0.0);
    let mut vel = Vector3::new(0.0, 0.0, 0.0);
    let mut db = core::DisplayBuffer::new(WIN_WIDTH, WIN_HEIGHT, 4);

    loop {
        let mut sprite = Sprite::new();

        loop {
            let event = match window.poll_event() {
                Some(val) => val,
                None => break,
            };

            match event {
                Event::Closed |
                Event::KeyPressed { code: Key::Q, .. } |
                Event::KeyPressed { code: Key::Escape, .. } => return,
                Event::KeyPressed { code: Key::D, .. } |
                Event::KeyPressed { code: Key::Right, .. } |
                Event::KeyPressed { code: Key::L, .. } => {
                    vel.x = 10.0;
                }
                Event::KeyReleased { code: Key::D, .. } |
                Event::KeyReleased { code: Key::Right, .. } |
                Event::KeyReleased { code: Key::L, .. } => {
                    vel.x = 0.0;
                }
                Event::KeyPressed { code: Key::A, .. } |
                Event::KeyPressed { code: Key::Left, .. } |
                Event::KeyPressed { code: Key::H, .. } => {
                    vel.x = -10.0;
                }

                Event::KeyReleased { code: Key::A, .. } |
                Event::KeyReleased { code: Key::Left, .. } |
                Event::KeyReleased { code: Key::H, .. } => {
                    vel.x = 0.0;
                }

                Event::KeyPressed { code: Key::W, .. } |
                Event::KeyPressed { code: Key::Up, .. } |
                Event::KeyPressed { code: Key::K, .. } => {
                    vel.z = -1.0;
                }
                Event::KeyReleased { code: Key::W, .. } |
                Event::KeyReleased { code: Key::Up, .. } |
                Event::KeyReleased { code: Key::K, .. } => {
                    vel.z = 0.0;
                }
                Event::KeyPressed { code: Key::S, .. } |
                Event::KeyPressed { code: Key::Down, .. } |
                Event::KeyPressed { code: Key::J, .. } => {
                    vel.z = 1.0;
                }
                Event::KeyReleased { code: Key::S, .. } |
                Event::KeyReleased { code: Key::Down, .. } |
                Event::KeyReleased { code: Key::J, .. } => {
                    vel.z = 0.0;
                }
                Event::KeyPressed { code: Key::R, .. } => {}
                _ => {}
            }
        }

        eye_pos.x = eye_pos.x + vel.x * (1.0 / FPS);
        eye_pos.y = eye_pos.y + vel.y * (1.0 / FPS);
        eye_pos.z = eye_pos.z + vel.z * (1.0 / FPS);

        /*
        let mouse_x = window.mouse_position().x;
        let mouse_y = WIN_HEIGHT as i32 - window.mouse_position().y;
        let lookat_x = (mouse_x as f32 - ((WIN_WIDTH/2) as f32)) / (WIN_WIDTH as f32);
        let lookat_y = (mouse_y as f32 - ((WIN_HEIGHT/2) as f32)) / (WIN_HEIGHT as f32);
        let lookat = Vector3::new(lookat_x as f32, lookat_y as f32, -1.0);
        */
        let lookat = Vector3::new(0.0, 0.0, -10.0);

        db.clear();
        cube.rotate(Vector3::new(0.001, 0.001, 0.001));
        cube2.rotate(Vector3::new(-0.001, -0.001, -0.001));

        cube.render(eye_pos, lookat, &mut db);
        cube2.render(eye_pos, lookat, &mut db);

/*
        if mouse_x > 0 && mouse_y > 0 {
            let color = core::Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            };
            let p1: na::Vector2<usize> = na::Vector2::new(0, 0);
            let p2: na::Vector2<usize> = na::Vector2::new(mouse_x as usize, mouse_y as usize);
            rasterization::draw_line_usize(p1, p2, color, &mut db);
        }
        let color = core::Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };

        let p1: na::Vector2<usize> = na::Vector2::new(0, core::WIN_HEIGHT/2);
        let p2: na::Vector2<usize> = na::Vector2::new(core::WIN_WIDTH-1, core::WIN_HEIGHT/2);
        rasterization::draw_line_usize(p1, p2, color, &mut db);

        let color = core::Color {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        };
        let p1: na::Vector2<usize> = na::Vector2::new(0, 384);
        let p2: na::Vector2<usize> = na::Vector2::new(50, 434);
        let p3: na::Vector2<usize> = na::Vector2::new(100, 334);

        rasterization::draw_triangle_usize(p1, p2, p3, color, &mut db);
        */

        if clock.elapsed_time().as_seconds() > 1.0 / FPS {
            clock.restart();
            texture.update_from_pixels(&*db.data, db.width as u32, db.height as u32, 0, 0);
            sprite.set_texture(&texture, false);

            window.clear(&Color::BLACK);
            window.draw(&sprite);
            window.display();
        }
    }
}
