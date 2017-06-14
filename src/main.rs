extern crate sfml;
extern crate nalgebra as na;
extern crate renderer;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::window::{Event, Key, style, VideoMode};
use sfml::system::Clock;
use na::{Vector3, Vector4};
use renderer::*;

const FPS: f32 = 60.0;

fn main() {
    let mut clock = Clock::start();
    let vm = VideoMode::new(core::WIN_WIDTH as u32, core::WIN_HEIGHT as u32, 32);
    let w = RenderWindow::new(vm, "GFX demo", style::CLOSE, &Default::default());

    let mut window = w.unwrap();
    window.set_vertical_sync_enabled(true);

    let mut texture = Texture::new(core::WIN_WIDTH as u32, core::WIN_HEIGHT as u32).unwrap();

    let mut cube = core::Mesh::new();
    cube.vertices
        .append(&mut vec![Vector4::new(-1.0, 1.0, 1.0, 1.0),
                          Vector4::new(1.0, 1.0, 1.0, 1.0),
                          Vector4::new(-1.0, -1.0, 1.0, 1.0),
                          Vector4::new(1.0, -1.0, 1.0, 1.0),
                          Vector4::new(-1.0, 1.0, -1.0, 1.0),
                          Vector4::new(1.0, 1.0, -1.0, 1.0),
                          Vector4::new(1.0, -1.0, -1.0, 1.0),
                          Vector4::new(-1.0, -1.0, -1.0, 1.0)]);

    cube.poly_sizes
        .append(&mut vec![3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
    cube.poly_indices
        .append(&mut vec![[0, 1, 2], [1, 2, 3], [1, 3, 6], [1, 5, 6], [0, 1, 4], [1, 4, 5],
                          [2, 3, 7], [3, 6, 7], [0, 2, 7], [0, 4, 7], [4, 5, 6], [4, 6, 7]]);

    cube.translate(Vector3::new(0.0, 0.0, -3.0));

    let mut eye_pos = Vector4::new(0.0, 0.0, 0.0, 1.0);
    let mut vel = Vector3::new(0.0, 0.0, 0.0);
    let mut db = core::DisplayBuffer::new(core::WIN_WIDTH, core::WIN_HEIGHT, 4);

    loop {
        let mut sprite = Sprite::new();

        for event in window.events() {
            match event {
                Event::Closed |
                Event::KeyPressed { code: Key::Q, .. } |
                Event::KeyPressed { code: Key::Escape, .. } => return,
                Event::KeyPressed { code: Key::D, .. } |
                Event::KeyPressed { code: Key::Right, .. } |
                Event::KeyPressed { code: Key::L, .. } => {
                    vel.x = 1.0;
                }
                Event::KeyReleased { code: Key::D, .. } |
                Event::KeyReleased { code: Key::Right, .. } |
                Event::KeyReleased { code: Key::L, .. } => {
                    vel.x = 0.0;
                }
                Event::KeyPressed { code: Key::A, .. } |
                Event::KeyPressed { code: Key::Left, .. } |
                Event::KeyPressed { code: Key::H, .. } => {
                    vel.x = -1.0;
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

        // rotate_mesh(&mut cube, Vector3::new(0.00, 0.01, 0.01));
        db.clear();
        cube.render(eye_pos, &mut db);

        if clock.elapsed_time().as_seconds() > 1.0 / FPS {
            clock.restart();
            texture.update_from_pixels(&db.data, db.width as u32, db.height as u32, 0, 0);
            sprite.set_texture(&texture, false);

            window.clear(&Color::black());
            window.draw(&sprite);
            window.display();
        }
    }
}
