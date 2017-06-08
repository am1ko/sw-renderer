extern crate sfml;
extern crate nalgebra as na;

use sfml::graphics::{Color, RenderTarget, RenderWindow, Sprite, Texture};
use sfml::window::{Event, Key, style, VideoMode};
use sfml::system::Clock;
use na::{Vector2, Vector3, Vector4, Matrix3x4, Matrix4, RowVector4};

const WIN_WIDTH: usize = 1024;
const WIN_HEIGHT: usize = 768;
const BYTES_PER_PIXEL: usize = 4;
const FPS: f32 = 60.0;

pub struct Mesh {
    vertices: Vec<Vector4<f32>>,
    poly_sizes: Vec<i32>,
    poly_indices: Vec<[i32; 3]>,
    position: Vector4<f32>,
    angle: Vector3<f32>,
}

impl Mesh {
    pub fn new() -> Mesh {
        return Mesh {
                   vertices: Vec::new(),
                   poly_sizes: Vec::new(),
                   poly_indices: Vec::new(),
                   position: Vector4::new(0.0, 0.0, 0.0, 1.0),
                   angle: Vector3::new(0.0, 0.0, 0.0),
               };
    }
}

fn set_pixel(x: usize,
             y: usize,
             color: u32,
             pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    let index = (WIN_HEIGHT - y) * WIN_WIDTH * BYTES_PER_PIXEL + x * BYTES_PER_PIXEL;
    if index > 0 && index < (pixels.len() - BYTES_PER_PIXEL) {
        pixels[index] = ((color & 0x000000FF) >> 0) as u8;
        pixels[index + 1] = ((color & 0x0000FF00) >> 8) as u8;
        pixels[index + 2] = ((color & 0x00FF0000) >> 16) as u8;
        pixels[index + 3] = ((color & 0xFF000000) >> 24) as u8;
    }
}

fn draw_line(p1: Vector2<f32>,
             p2: Vector2<f32>,
             color: u32,
             pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {

    let threshold = 1.0;
    let sub = p2 - p1;

    let dist = (sub.x + sub.y).abs().sqrt();

    if dist > threshold {
        let middle = p1 + sub / 2.0;
        if (middle.x >= 0.0 && middle.x <= WIN_WIDTH as f32) &&
           (middle.y >= 0.0 && middle.y <= WIN_HEIGHT as f32) {
            set_pixel(middle.x as usize, middle.y as usize, color, pixels);

            draw_line(p1, middle, color, pixels);
            draw_line(middle, p2, color, pixels);
        }
    }
}

// fn draw_point(p: Vector2<f32>,
// color: u32,
// pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
//
// for i in -1..2 {
// for j in -1..2 {
// let px = p.x + ((i as i32) as f32);
// let py = p.y + ((j as i32) as f32);
//
// if px > 0.0 && py > 0.0 {
// set_pixel(px as usize, py as usize, color, pixels);
// }
// }
// }
// }


fn to_ndc_space(v: Vector2<f32>) -> Vector2<f32> {
    let ret = Vector2::new((1.0 + v.x) / 2.0, (1.0 + v.y) / 2.0);

    return ret;
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


//fn rotate_mesh(mesh: &mut Mesh, angle: Vector3<f32>) {
//    mesh.angle.x = mesh.angle.x + angle.x;
//    mesh.angle.y = mesh.angle.y + angle.y;
//    mesh.angle.z = mesh.angle.z + angle.z;
//}

fn project_vertex(v: Vector4<f32>, m: Matrix4<f32>) -> Vector2<f32> {
    let v_xformed = m * v;

    // Perspective division, far away points moved closer to origin
    // To screen space. All visible points between [-1, 1].
    let scr = Vector2::new(v_xformed.x / v_xformed.w, v_xformed.y / v_xformed.w);

    // To Normalized Device Coordinates. All visible points between [0, 1]
    let n = to_ndc_space(scr);

    // To actual screen pixel coordinates
    return to_raster_space(n);
}

fn render_mesh(mesh: &Mesh,
               eye: Vector4<f32>,
               pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
    let m_rot_x =
        Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, 0.0),
                             RowVector4::new(0.0, mesh.angle.x.cos(), mesh.angle.x.sin(), 0.0),
                             RowVector4::new(0.0, -mesh.angle.x.sin(), mesh.angle.x.cos(), 0.0),
                             RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
    let m_rot_y =
        Matrix4::from_rows(&[RowVector4::new(mesh.angle.y.cos(), 0.0, -mesh.angle.y.sin(), 0.0),
                             RowVector4::new(0.0, 1.0, 0.0, 0.0),
                             RowVector4::new(mesh.angle.y.sin(), 0.0, mesh.angle.y.cos(), 0.0),
                             RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
    let m_rot_z =
        Matrix4::from_rows(&[RowVector4::new(mesh.angle.z.cos(), -mesh.angle.z.sin(), 0.0, 0.0),
                             RowVector4::new(mesh.angle.z.sin(), mesh.angle.z.cos(), 0.0, 0.0),
                             RowVector4::new(0.0, 0.0, 1.0, 0.0),
                             RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

    let m_trans = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, mesh.position.x),
                                       RowVector4::new(0.0, 1.0, 0.0, mesh.position.y),
                                       RowVector4::new(0.0, 0.0, 1.0, mesh.position.z),
                                       RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

    let model = m_trans * m_rot_z * m_rot_y * m_rot_x;
    let view: Matrix4<f32> = look_at(eye, mesh.position, Vector4::new(0.0, 1.0, 0.0, 0.0));
    let projection: Matrix4<f32> =
        perspective_projection(0.1, 5.0, 78.0, ((WIN_WIDTH as f32) / (WIN_HEIGHT as f32)));
    let xform = projection * view * model;

    for p in mesh.poly_indices.iter() {
        let p1 = project_vertex(mesh.vertices[p[0] as usize], xform);
        let p2 = project_vertex(mesh.vertices[p[1] as usize], xform);
        let p3 = project_vertex(mesh.vertices[p[2] as usize], xform);

        draw_line(p1, p2, 0xFFFFFFFF, pixels);
        draw_line(p2, p3, 0xFFFFFFFF, pixels);
        draw_line(p3, p1, 0xFFFFFFFF, pixels);
    }

    // let m = model * v;
    // let vi = view * m;
    // let p = projection * vi;
    // println!("MODEL SPACE");
    // println!("{}", v);
    // println!("WORLD SPACE");
    // println!("{}", m);
    // println!("VIEW SPACE");
    // println!("{}", vi);
    // println!("PROJECTION SPACE");
    // println!("{}", p);
    // println!("SCREEN SPACE");
    // println!("{}", scr);
    // println!("NDC SPACE");
    // println!("{}", n);
    // println!("RASTER SPACE");
    // println!("{}", r);
    //
}

fn perspective_projection(n: f32, f: f32, angle_of_view: f32, aspect_ratio: f32) -> Matrix4<f32> {
    let deg_to_rad = std::f32::consts::PI / 180.0;
    let size = n * (deg_to_rad * angle_of_view / 2.0).tan();
    let l = -size;
    let r = size;
    let b = -size / aspect_ratio;
    let t = size / aspect_ratio;

    return Matrix4::from_rows(&[RowVector4::new(2.0 * n / (r - l), 0.0, (r + l) / (r - l), 0.0),
                                RowVector4::new(0.0, 2.0 * n / (t - b), (t + b) / (t - b), 0.0),
                                RowVector4::new(0.0,
                                                0.0,
                                                -(f + n) / (f - n),
                                                -(2.0 * f * n) / (f - n)),
                                RowVector4::new(0.0, 0.0, -1.0, 0.0)]);

}

fn look_at(eye: Vector4<f32>, lookat: Vector4<f32>, up: Vector4<f32>) -> Matrix4<f32> {
    // Rotate so that the line of sight from the eye position to the target maps to the z axis.
    // Camera up direction maps to y axis. x- axis is defined from the other two by cross product

    let reduce_dim = Matrix3x4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, 0.0),
                                            RowVector4::new(0.0, 1.0, 0.0, 0.0),
                                            RowVector4::new(0.0, 0.0, 1.0, 0.0)]);
    let eye = reduce_dim * eye;
    let lookat = reduce_dim * lookat;
    let up = reduce_dim * up;

    let z = (lookat - eye).normalize();
    let x = (up.cross(&z)).normalize();
    let y = (z.cross(&x)).normalize();

    //let rotation = Matrix4::from_columns(&[x, y, z, Vector4::new(0.0, 0.0, 0.0, 1.0)]);

    let rotation = Matrix4::from_rows(&[RowVector4::new(x.x, x.y, x.z, 0.0),
                                        RowVector4::new(y.x, y.y, y.z, 0.0),
                                        RowVector4::new(z.x, z.y, z.z, 0.0),
                                        RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

    // Translate to the inverse of the eye position (the world rotates in the opposite direction
    // around the camera that is fixed)
    let translation = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, -eye.x),
                                           RowVector4::new(0.0, 1.0, 0.0, -eye.y),
                                           RowVector4::new(0.0, 0.0, 1.0, -eye.z),
                                           RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
    return rotation * translation;
}

fn main() {
    let mut clock = Clock::start();
    let vm = VideoMode::new(WIN_WIDTH as u32, WIN_HEIGHT as u32, 32);
    let w = RenderWindow::new(vm, "GFX demo", style::CLOSE, &Default::default());

    let mut window = w.unwrap();
    window.set_vertical_sync_enabled(true);

    let mut texture = Texture::new(WIN_WIDTH as u32, WIN_HEIGHT as u32).unwrap();

    let mut cube = Mesh::new();
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

    translate_mesh(&mut cube, Vector3::new(0.0, 0.0, -3.0));

    let mut eye_pos = Vector4::new(0.0, 0.0, 0.0, 1.0);
    let mut vel = Vector3::new(0.0, 0.0, 0.0);
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

        //rotate_mesh(&mut cube, Vector3::new(0.00, 0.01, 0.01));
        render_mesh(&cube, eye_pos, &mut display_buffer);

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
