// Software rendering pipeline
//
// For each mesh m
// For each vertex v (dim 4) in m
// 1) Model to world space (MODEL matrix 4x4)
// 2) World to camera space (VIEW matrix 4x4)
// 3) Camera to homogeneous clip space (PROJECTION matrix 4x4), w = 1
// 4) Clipping + perspective divide (normalization) => NDC space [-1, 1]
// 5) Viewport transform => raster space [0, W-1, 0, H-1]

use na::{Vector2, Vector3, Vector4, Matrix3x4, Matrix4, RowVector4};

/// Renderable represents any model that can be drawn to a display buffer
pub trait Renderable {
    /// Draw the model to a display buffer (render target)
    ///
    /// * `color` - Color to use
    /// * `buffer` - Display buffer (render target)
    fn render(&self, color: Color, buffer: &mut DisplayBuffer);
}

pub struct Triangle<T> {
    /// Vertex of a triangle
    pub v0: T,
    /// Vertex of a triangle
    pub v1: T,
    /// Vertex of a triangle
    pub v2: T,
}

pub struct LineSegment<T> {
    /// End point of line segment
    pub v0: T,
    /// End point of line segment
    pub v1: T,
}

impl Triangle<Vector2<usize>> {
    /// Order the points of a triangle based on the y-coordinate
    pub fn order_by_y(&mut self) {
        let mut ordered = [self.v0, self.v1, self.v2];
        ordered.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        self.v0 = ordered[2];
        self.v1 = ordered[1];
        self.v2 = ordered[0];
    }

    /// Return true if the triangle is top-flat
    pub fn is_top_flat(&self) -> bool {
        self.v0.y == self.v1.y
    }

    /// Return true if the triangle is bottom-flat
    pub fn is_bottom_flat(&self) -> bool {
        self.v1.y == self.v2.y
    }
}

/// Transforms a single vertex from model space to viewport
///
/// # Arguments
///
/// * `v` - A vertex in model coordinates
/// * `m` - A transformation matrix
/// * `vp_width` - Width of the viewport
/// * `vp_height` - Height of the viewport
/// * return - Transformed vertex
fn transform_vertex(
    v: Vector4<f32>,
    m: Matrix4<f32>,
    vp_width: f32,
    vp_height: f32,
) -> Vector2<usize> {
    // Steps 1 - 3: MODEL-VIEW-PROJECTION transform
    let v_camera = m * v;

    // Step 4.1: CLIPPING
    // TODO(amiko)

    // Step 4.2: PERSPECTIVE DIVIDE (normalization)
    // Perspective division, far away points moved closer to origin
    // To screen space. All visible points between [-1, 1].
    let v_ndc = Vector3::new(
        v_camera.x / v_camera.w,
        v_camera.y / v_camera.w,
        v_camera.z / v_camera.w,
    );

    // Step 5: Viewport transform
    Vector2::new(
        ((1.0 + v_ndc.x) * 0.5 * vp_width) as usize,
        ((1.0 + v_ndc.y) * 0.5 * vp_height) as usize,
    )
}

fn build_perspective_matrix(n: f32, f: f32, angle_of_view: f32, aspect_ratio: f32) -> Matrix4<f32> {
    let deg_to_rad = ::std::f32::consts::PI / 180.0;
    let size = n * (deg_to_rad * angle_of_view / 2.0).tan();
    let l = -size;
    let r = size;
    let b = -size / aspect_ratio;
    let t = size / aspect_ratio;

    return Matrix4::from_rows(
        &[
            RowVector4::new(2.0 * n / (r - l), 0.0, (r + l) / (r - l), 0.0),
            RowVector4::new(0.0, 2.0 * n / (t - b), (t + b) / (t - b), 0.0),
            RowVector4::new(0.0, 0.0, -(f + n) / (f - n), -(2.0 * f * n) / (f - n)),
            RowVector4::new(0.0, 0.0, -1.0, 0.0),
        ],
    );

}

fn build_view_matrix(eye: Vector4<f32>, lookat: Vector4<f32>, up: Vector4<f32>) -> Matrix4<f32> {
    // Rotate so that the line of sight from the eye position to the target maps to the z axis.
    // Camera up direction maps to y axis. x- axis is defined from the other two by cross
    // product

    // We do not care about the w-component. Lets get rid of it since cross product is not
    // defined for 4D vectors
    let reduce_dim = Matrix3x4::from_rows(
        &[
            RowVector4::new(1.0, 0.0, 0.0, 0.0),
            RowVector4::new(0.0, 1.0, 0.0, 0.0),
            RowVector4::new(0.0, 0.0, 1.0, 0.0),
        ],
    );
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
    let rotation = Matrix4::from_rows(
        &[
            RowVector4::new(x.x, x.y, x.z, 0.0),
            RowVector4::new(y.x, y.y, y.z, 0.0),
            RowVector4::new(z.x, z.y, z.z, 0.0),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ],
    );

    // Translate to the inverse of the eye position (the world moves in the opposite direction
    // around the camera that is fixed)
    let translation = Matrix4::from_rows(
        &[
            RowVector4::new(1.0, 0.0, 0.0, -eye.x),
            RowVector4::new(0.0, 1.0, 0.0, -eye.y),
            RowVector4::new(0.0, 0.0, 1.0, -eye.z),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ],
    );

    // Use inverse multiplication order to produce inversed combined matrix
    return rotation * translation;
}

/// Color in RGBA8888 format
#[derive(Copy, Clone)]
pub struct Color {
    /// Red component intensity
    pub r: u8,
    /// Green component intensity
    pub g: u8,
    /// Blue component intensity
    pub b: u8,
    /// Alpha value
    pub a: u8,
}

/// Display buffer defines a memory area that is used for rendering a raw image
pub struct DisplayBuffer {
    /// Width of the display area in pixels
    pub width: usize,
    /// Height of the display area in pixels
    pub height: usize,
    /// Bytes per pixel
    pub bpp: usize,
    /// Contents of the buffer (pixel data)
    pub data: Box<[u8]>,
}

impl DisplayBuffer {
    pub fn new(width: usize, height: usize, bpp: usize) -> DisplayBuffer {
        return DisplayBuffer {
            height: height,
            width: width,
            bpp: bpp,
            data: vec![0; width * height * bpp].into_boxed_slice(),
        };
    }

    /// return the size of the buffer in bytes
    pub fn size(&self) -> usize {
        return self.height * self.width * self.bpp;
    }

    /// Reset the contents of the buffer so that all pixels are black
    pub fn clear(&mut self) {
        self.data = vec![0; self.width * self.height * self.bpp].into_boxed_slice();
    }

    /// Set a single pixel to a desired color
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate in pixels, value 0 corresponds to left edge
    /// * `y` - Y coordinate in pixels, value 0 correspoonds to bottom edge
    /// * 'color' - Color of the pixel
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        assert!(x < self.width);
        assert!(y < self.height);
        let index: usize = (self.height - y - 1) * self.width * self.bpp + x * self.bpp;
        if index < (self.size() - self.bpp) {
            self.data[index] = color.r;
            self.data[index + 1] = color.g;
            self.data[index + 2] = color.b;
            self.data[index + 3] = color.a;
        }
    }
}

/// A mesh is a collection of triangles that form a 3D surface
pub struct Mesh {
    /// Individual vertices that make up the surface of the mesh. Each vertex is
    /// a 4D vector [x, y, z, w]
    pub vertices: Vec<Vector4<f32>>,
    /// Size of each polygon in vertices
    pub poly_sizes: Vec<i32>,
    /// Specifies which vertices make a single polygon.
    pub poly_indices: Vec<[i32; 3]>,
    /// World position of the center of the mesh
    pub position: Vector4<f32>,
    /// Rotation of the mesh around all 3 axis vectors
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

    /// Render a mesh into a display buffer
    ///
    /// # Arguments
    ///
    /// * `eye` - Position of the camera eye
    /// * `buffer` - Display buffer (render target)
    pub fn render(self: &Mesh, eye: Vector4<f32>, buffer: &mut DisplayBuffer) {
        let m_rot_x = Matrix4::from_rows(
            &[
                RowVector4::new(1.0, 0.0, 0.0, 0.0),
                RowVector4::new(0.0, self.angle.x.cos(), self.angle.x.sin(), 0.0),
                RowVector4::new(0.0, -self.angle.x.sin(), self.angle.x.cos(), 0.0),
                RowVector4::new(0.0, 0.0, 0.0, 1.0),
            ],
        );
        let m_rot_y = Matrix4::from_rows(
            &[
                RowVector4::new(self.angle.y.cos(), 0.0, -self.angle.y.sin(), 0.0),
                RowVector4::new(0.0, 1.0, 0.0, 0.0),
                RowVector4::new(self.angle.y.sin(), 0.0, self.angle.y.cos(), 0.0),
                RowVector4::new(0.0, 0.0, 0.0, 1.0),
            ],
        );
        let m_rot_z = Matrix4::from_rows(
            &[
                RowVector4::new(self.angle.z.cos(), -self.angle.z.sin(), 0.0, 0.0),
                RowVector4::new(self.angle.z.sin(), self.angle.z.cos(), 0.0, 0.0),
                RowVector4::new(0.0, 0.0, 1.0, 0.0),
                RowVector4::new(0.0, 0.0, 0.0, 1.0),
            ],
        );

        let m_trans = Matrix4::from_rows(
            &[
                RowVector4::new(1.0, 0.0, 0.0, self.position.x),
                RowVector4::new(0.0, 1.0, 0.0, self.position.y),
                RowVector4::new(0.0, 0.0, 1.0, self.position.z),
                RowVector4::new(0.0, 0.0, 0.0, 1.0),
            ],
        );

        let model = m_trans * m_rot_z * m_rot_y * m_rot_x;
        let aspect_ratio = (buffer.width as f32) / (buffer.height as f32);
        let view: Matrix4<f32> =
            build_view_matrix(eye, self.position, Vector4::new(0.0, 1.0, 0.0, 0.0));
        let projection: Matrix4<f32> = build_perspective_matrix(0.1, 5.0, 78.0, aspect_ratio);
        let xform = projection * view * model;

        // loop through all polygons, each consists of 3 vertices
        for (i, p) in self.poly_indices.iter().enumerate() {
            let color = Color {
                r: 0,
                g: 255,
                b: 0,
                a: (255 / 4 + 3 * 255 / 4 * i / self.poly_indices.len()) as u8,
            };

            let mut triangle = Triangle {
                v0: transform_vertex(
                    self.vertices[p[0] as usize],
                    xform,
                    buffer.width as f32,
                    buffer.height as f32,
                ),
                v1: transform_vertex(
                    self.vertices[p[1] as usize],
                    xform,
                    buffer.width as f32,
                    buffer.height as f32,
                ),
                v2:  transform_vertex(
                    self.vertices[p[2] as usize],
                    xform,
                    buffer.width as f32,
                    buffer.height as f32,
                ),
            };
            triangle.order_by_y();
            triangle.render(color, buffer);

            // wireframe rendering
            // rasterization::draw_line_usize(p1, p2, color, buffer);
            // rasterization::draw_line_usize(p2, p3, color, buffer);
            // rasterization::draw_line_usize(p3, p1, color, buffer);
        }
    }

    /// Translate (move) a mesh in spce
    ///
    /// # Arguments
    ///
    /// * `translation` - Vector that specifies the desired displacement
    pub fn translate(self: &mut Mesh, translation: Vector3<f32>) {
        let xform = Matrix4::from_rows(
            &[
                RowVector4::new(1.0, 0.0, 0.0, translation.x),
                RowVector4::new(0.0, 1.0, 0.0, translation.y),
                RowVector4::new(0.0, 0.0, 1.0, translation.z),
                RowVector4::new(0.0, 0.0, 0.0, 1.0),
            ],
        );
        self.position = xform * self.position;
    }

    /// Rotate a mesh
    ///
    /// # Arguments
    ///
    /// * `angle` - Desired rotation angle around each cartesian axis in radians
    pub fn rotate(this: &mut Mesh, angle: Vector3<f32>) {
        this.angle.x = this.angle.x + angle.x;
        this.angle.y = this.angle.y + angle.y;
        this.angle.z = this.angle.z + angle.z;
    }
}
