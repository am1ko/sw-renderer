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

fn draw_line(p1: Vector2<f32>,
             p2: Vector2<f32>,
             color: u32,
             pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {

    let threshold = 1.0;
    let sub = (p2 - p1);

    let dist = (sub.x + sub.y).abs().sqrt();

    if dist > threshold {
        let middle = p1 + sub / 2.0;
        if (middle.x >= 0.0 && middle.x <= WIN_WIDTH as f32) &&
            (middle.y >= 0.0 && middle.y <= WIN_HEIGHT as f32)
        {
            set_pixel(middle.x as usize, middle.y as usize, color, pixels);

            draw_line(p1, middle, color, pixels);
            draw_line(middle, p2, color, pixels);
        }
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

fn to_ndc_space(v: Vector2<f32>) -> Vector2<f32> {
    let ret = Vector2::new((1.0 + v.x) / 2.0, (1.0 + v.y) / 2.0);

    /*
    assert!(ret.x >= 0.0 && ret.x <= 1.0);
    assert!(ret.y >= 0.0 && ret.y <= 1.0);
    */

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


fn rotate_mesh(mesh: &mut Mesh, angle_x: f32, angle_y: f32, angle_z: f32) {
    mesh.angle_x = mesh.angle_x + angle_x;
    mesh.angle_y = mesh.angle_y + angle_y;
    mesh.angle_z = mesh.angle_z + angle_z;
}

fn project_vertex(v: Vector4<f32>, m: Matrix4<f32>) -> Vector2<f32> {
        let v_xformed = m * v;

        /* Perspective division, far away points moved closer to origin */
        /* To screen space. All visible points between [-1, 1]. */
        let scr = Vector2::new(v_xformed.x / v_xformed.w, v_xformed.y / v_xformed.w);

        /* To Normalized Device Coordinates. All visible points between [0, 1] */
        let n = to_ndc_space(scr);

        /* To actual screen pixel coordinates */
        return to_raster_space(n);
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

    let model = m_trans * /*m_rot_z */ m_rot_y /* m_rot_x*/;
    let view: Matrix4<f32> = Matrix4::identity(); // TODO(amiko)
    let projection: Matrix4<f32> = perspective_projection(0.1, 5.0, 78.0, 1.33); // TODO(amiko)
    //let projection: Matrix4<f32> = Matrix4::identity(); // TODO(amiko)

    //for v in mesh.vertices.iter() {
    for i in 0..mesh.vertices.len()-1 {
        let xform = projection * view * model;
        let v1 = mesh.vertices[i];
        let v2 = mesh.vertices[i+1];
        let p1 = project_vertex(v1, xform);
        let p2 = project_vertex(v2, xform);
        println!("{}", p1);
        println!("{}", p2);
        draw_line(p1, p2, 0xFFFFFFFF, pixels);
        /*
        draw_point(p1, 0xFFFFFFFF, pixels);
        draw_point(p2, 0xFFFFFFFF, pixels);
        */

/*
        let m = model * v;
        let vi = view * m;
        let p = projection * vi;
        println!("MODEL SPACE");
        println!("{}", v);
        println!("WORLD SPACE");
        println!("{}", m);
        println!("VIEW SPACE");
        println!("{}", vi);
        println!("PROJECTION SPACE");
        println!("{}", p);
        println!("SCREEN SPACE");
        println!("{}", scr);
        println!("NDC SPACE");
        println!("{}", n);
        println!("RASTER SPACE");
        println!("{}", r);
        */
    }
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

        rotate_mesh(&mut cube, 0.00, 0.01, 0.01);
        render_mesh(&cube, &mut display_buffer);
        /*
        let p2 = Vector2::new(100.0, 200.0);
        let p1 = Vector2::new(700.0, 500.0);
        draw_line(p1, p2, 0xFFFFFFFF, &mut display_buffer);
        */

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
