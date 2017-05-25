extern crate sfml;
extern crate nalgebra as na;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::window::{Event, Key, style, VideoMode};
use sfml::system::Clock;
use na::{Vector2, Vector3, Vector4, Matrix4, RowVector4};

const WIN_WIDTH: usize = 800;
const WIN_HEIGHT: usize = 600;
const BYTES_PER_PIXEL: usize = 4;
const FPS: f32 = 60.0;

pub struct Mesh {
    vertices: Vec<Vector4<f32>>,
    position: Vector4<f32>,
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,
}

impl Mesh {
    pub fn new() -> Mesh {
        return Mesh {
                   vertices: Vec::new(),
                   position: Vector4::new(0.0, 0.0, 0.0, 1.0),
                   angle_x: 0.0,
                   angle_y: 0.0,
                   angle_z: 0.0,
               };
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

fn draw_point(p: Vector2<f32>,
              color: u32,
              pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {

    for i in -1..2 {
        for j in -1..2 {
            let px = p.x + ((i as i32) as f32);
            let py = p.y + ((j as i32) as f32);

            if px > 0.0 && py > 0.0 {
                set_pixel(px as usize, py as usize, color, pixels);
            }
        }
    }
}

// applies only for special case
fn normalize(v: Vector2<f32>) -> Vector2<f32> {
    return Vector2::new((1.0 + v.x) / 2.0, (1.0 + v.y) / 2.0);
}

fn to_raster_space(v: Vector2<f32>) -> Vector2<f32> {
    return Vector2::new(v.x * WIN_WIDTH as f32, v.y * WIN_HEIGHT as f32);
}

fn translate_mesh(mesh: &mut Mesh, translation: Vector3<f32>) {
    let xform = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, translation.x),
                                     RowVector4::new(0.0, 1.0, 0.0, translation.y),
                                     RowVector4::new(0.0, 0.0, 1.0, translation.z),
                                     RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
    mesh.position = xform * mesh.position;
}


fn rotate_mesh(mesh: &mut Mesh, angle_x: f32, angle_y: f32, angle_z: f32) {
    mesh.angle_x = mesh.angle_x + angle_x;
    mesh.angle_y = mesh.angle_y + angle_y;
    mesh.angle_z = mesh.angle_z + angle_z;
}

fn render_mesh(mesh: &Mesh, pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    let m_rot_x =
        Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, 0.0),
                             RowVector4::new(0.0, mesh.angle_x.cos(), mesh.angle_x.sin(), 0.0),
                             RowVector4::new(0.0, -mesh.angle_x.sin(), mesh.angle_x.cos(), 0.0),
                             RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
    let m_rot_y =
        Matrix4::from_rows(&[RowVector4::new(mesh.angle_y.cos(), 0.0, -mesh.angle_y.sin(), 0.0),
                             RowVector4::new(0.0, 1.0, 0.0, 0.0),
                             RowVector4::new(mesh.angle_y.sin(), 0.0, mesh.angle_y.cos(), 0.0),
                             RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
    let m_rot_z =
        Matrix4::from_rows(&[RowVector4::new(mesh.angle_z.cos(), -mesh.angle_z.sin(), 0.0, 0.0),
                             RowVector4::new(mesh.angle_z.sin(), mesh.angle_z.cos(), 0.0, 0.0),
                             RowVector4::new(0.0, 0.0, 1.0, 0.0),
                             RowVector4::new(0.0, 0.0, 0.0, 1.0)]);


    let m_trans = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, mesh.position.x),
                                       RowVector4::new(0.0, 1.0, 0.0, mesh.position.y),
                                       RowVector4::new(0.0, 0.0, 1.0, mesh.position.z),
                                       RowVector4::new(0.0, 0.0, 0.0, 1.0)]);


    for v in mesh.vertices.iter() {
        /* model to world */
        let v_xformed = m_trans * m_rot_x * m_rot_y * m_rot_z * v;

        /* TODO: world to view */

        /* view to projection */
        let v_proj = Vector2::new(v_xformed.x / v_xformed.z, v_xformed.y / v_xformed.z);
        let n = normalize(v_proj);
        let r = to_raster_space(n);
        draw_point(r, 0xFFFFFFFF, pixels);
    }
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
    cube.vertices
        .append(&mut vec![Vector4::new(1.0, -1.0, -1.0, 1.0),
                          Vector4::new(1.0, -1.0, 1.0, 1.0),
                          Vector4::new(1.0, 1.0, -1.0, 1.0),
                          Vector4::new(1.0, 1.0, 1.0, 1.0),
                          Vector4::new(-1.0, -1.0, -1.0, 1.0),
                          Vector4::new(-1.0, -1.0, 1.0, 1.0),
                          Vector4::new(-1.0, 1.0, -1.0, 1.0),
                          Vector4::new(-1.0, 1.0, 1.0, 1.0)]);


    translate_mesh(&mut cube, Vector3::new(0.0, 0.0, -3.0));

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
                    translate_mesh(&mut cube, Vector3::new(0.1, 0.0, 0.0));
                }
                Event::KeyPressed { code: Key::A, .. } |
                Event::KeyPressed { code: Key::Left, .. } |
                Event::KeyPressed { code: Key::H, .. } => {
                    translate_mesh(&mut cube, Vector3::new(-0.1, 0.0, 0.0));
                }
                Event::KeyPressed { code: Key::W, .. } |
                Event::KeyPressed { code: Key::Up, .. } |
                Event::KeyPressed { code: Key::K, .. } => {
                    translate_mesh(&mut cube, Vector3::new(0.0, 0.0, 0.1));
                }
                Event::KeyPressed { code: Key::S, .. } |
                Event::KeyPressed { code: Key::Down, .. } |
                Event::KeyPressed { code: Key::J, .. } => {
                    translate_mesh(&mut cube, Vector3::new(0.0, 0.0, -0.1));
                }
                Event::KeyPressed { code: Key::R, .. } => {}
                _ => {}
            }
        }

        rotate_mesh(&mut cube, 0.00, 0.01 ,0.01);
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
