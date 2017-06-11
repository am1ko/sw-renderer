extern crate nalgebra as na;
use self::na::{Vector2, Vector3, Vector4, Matrix3x4, Matrix4, RowVector4};

pub const WIN_WIDTH: usize = 1024;
pub const WIN_HEIGHT: usize = 768;
pub const BYTES_PER_PIXEL: usize = 4;

fn to_ndc_space(v: Vector2<f32>) -> Vector2<f32> {
    let ret = Vector2::new((1.0 + v.x) / 2.0, (1.0 + v.y) / 2.0);

    return ret;
}

fn to_raster_space(v: Vector2<f32>) -> Vector2<f32> {
    return Vector2::new(v.x * WIN_WIDTH as f32, v.y * WIN_HEIGHT as f32);
}

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

fn perspective_projection(n: f32, f: f32, angle_of_view: f32, aspect_ratio: f32) -> Matrix4<f32> {
    let deg_to_rad = ::std::f32::consts::PI / 180.0;
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
    // Camera up direction maps to y axis. x- axis is defined from the other two by cross
    // product

    // We do not care about the w-component. Lets get rid of it since cross product is not
    // defined for 4D vectors
    let reduce_dim = Matrix3x4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, 0.0),
                                            RowVector4::new(0.0, 1.0, 0.0, 0.0),
                                            RowVector4::new(0.0, 0.0, 1.0, 0.0)]);
    let eye = reduce_dim * eye;
    let lookat = reduce_dim * lookat;
    let up = reduce_dim * up;

    // Unit vectors in camera space
    let z = (lookat - eye).normalize();
    let x = (up.cross(&z)).normalize();
    let y = (z.cross(&x)).normalize();

    // The view matrix is the inverse of a model matrix that would transform a model of the
    // camera into world space (transformation and rotation)

    // This is an orientation matrix that is transposed. Transpose effectively performs
    // inversion. This achieves the effect that the world rotates around the camera
    let rotation = Matrix4::from_rows(&[RowVector4::new(x.x, x.y, x.z, 0.0),
                                        RowVector4::new(y.x, y.y, y.z, 0.0),
                                        RowVector4::new(z.x, z.y, z.z, 0.0),
                                        RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

    // Translate to the inverse of the eye position (the world moves in the opposite direction
    // around the camera that is fixed)
    let translation = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, -eye.x),
                                           RowVector4::new(0.0, 1.0, 0.0, -eye.y),
                                           RowVector4::new(0.0, 0.0, 1.0, -eye.z),
                                           RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

    // Use inverse multiplication order to produce inversed combined matrix
    return rotation * translation;
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

pub struct Mesh {
    pub vertices: Vec<Vector4<f32>>,
    pub poly_sizes: Vec<i32>,
    pub poly_indices: Vec<[i32; 3]>,
    pub position: Vector4<f32>,
    pub angle: Vector3<f32>,
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

    pub fn render(self: &Mesh,
                  eye: Vector4<f32>,
                  pixels: &mut [u8; WIN_WIDTH * WIN_HEIGHT * BYTES_PER_PIXEL]) {
        let m_rot_x = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, 0.0),
                                           RowVector4::new(0.0,
                                                           self.angle.x.cos(),
                                                           self.angle.x.sin(),
                                                           0.0),
                                           RowVector4::new(0.0,
                                                           -self.angle.x.sin(),
                                                           self.angle.x.cos(),
                                                           0.0),
                                           RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
        let m_rot_y = Matrix4::from_rows(&[RowVector4::new(self.angle.y.cos(),
                                                           0.0,
                                                           -self.angle.y.sin(),
                                                           0.0),
                                           RowVector4::new(0.0, 1.0, 0.0, 0.0),
                                           RowVector4::new(self.angle.y.sin(),
                                                           0.0,
                                                           self.angle.y.cos(),
                                                           0.0),
                                           RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
        let m_rot_z = Matrix4::from_rows(&[RowVector4::new(self.angle.z.cos(),
                                                           -self.angle.z.sin(),
                                                           0.0,
                                                           0.0),
                                           RowVector4::new(self.angle.z.sin(),
                                                           self.angle.z.cos(),
                                                           0.0,
                                                           0.0),
                                           RowVector4::new(0.0, 0.0, 1.0, 0.0),
                                           RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

        let m_trans = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, self.position.x),
                                           RowVector4::new(0.0, 1.0, 0.0, self.position.y),
                                           RowVector4::new(0.0, 0.0, 1.0, self.position.z),
                                           RowVector4::new(0.0, 0.0, 0.0, 1.0)]);

        let model = m_trans * m_rot_z * m_rot_y * m_rot_x;
        let view: Matrix4<f32> = look_at(eye, self.position, Vector4::new(0.0, 1.0, 0.0, 0.0));
        let projection: Matrix4<f32> =
            perspective_projection(0.1, 5.0, 78.0, ((WIN_WIDTH as f32) / (WIN_HEIGHT as f32)));
        let xform = projection * view * model;

        for p in self.poly_indices.iter() {
            let p1 = project_vertex(self.vertices[p[0] as usize], xform);
            let p2 = project_vertex(self.vertices[p[1] as usize], xform);
            let p3 = project_vertex(self.vertices[p[2] as usize], xform);

            draw_line(p1, p2, 0xFFFFFFFF, pixels);
            draw_line(p2, p3, 0xFFFFFFFF, pixels);
            draw_line(p3, p1, 0xFFFFFFFF, pixels);
        }
    }

    pub fn translate(self: &mut Mesh, translation: Vector3<f32>) {
        let xform = Matrix4::from_rows(&[RowVector4::new(1.0, 0.0, 0.0, translation.x),
                                         RowVector4::new(0.0, 1.0, 0.0, translation.y),
                                         RowVector4::new(0.0, 0.0, 1.0, translation.z),
                                         RowVector4::new(0.0, 0.0, 0.0, 1.0)]);
        self.position = xform * self.position;
    }

    pub fn rotate(this: &mut Mesh, angle: Vector3<f32>) {
        this.angle.x = this.angle.x + angle.x;
        this.angle.y = this.angle.y + angle.y;
        this.angle.z = this.angle.z + angle.z;
    }
}
