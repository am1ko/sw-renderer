// Software rendering pipeline
//
// For each mesh m
// For each vertex v (dim 4) in m
// 1) Model to world space (MODEL matrix 4x4)
// 2) World to camera space (VIEW matrix 4x4)
// 3) Camera to homogeneous clip space (PROJECTION matrix 4x4), w = 1
// 4) Clipping + perspective divide (normalization) => NDC space [-1, 1]
// 5) Viewport transform => raster space [0, W-1, 0, H-1]

use na::{Matrix3x4, Matrix4, RowVector4, Vector3, Vector4};
use std::mem;

/// Renderable represents any model that can be drawn to a display buffer
pub trait Renderable {
    /// Draw the model to a display buffer (render target)
    ///
    /// * `color` - Color to use
    /// * `buffer` - Display buffer (render target)
    fn render(&self, color: Color, buffer: &mut DisplayBuffer);
}

pub struct Triangle<T> {
    /// Vertex of a triangle (largest y-coordinate)
    pub v0: T,
    /// Vertex of a triangle
    pub v1: T,
    /// Vertex of a triangle (smallest y-coordinate)
    pub v2: T,
}

pub struct LineSegment<T> {
    /// End point of line segment
    pub v0: T,
    /// End point of line segment
    pub v1: T,
}

impl Triangle<Vector4<f32>> {
    /// Perform a linear transformation to all vertices of the triangle
    pub fn transform(&self, m: Matrix4<f32>) -> Triangle<Vector4<f32>> {
        Triangle {
            v0: m * self.v0,
            v1: m * self.v1,
            v2: m * self.v2,
        }
    }
}

impl Triangle<Vector3<f32>> {
    /// Return the center point of the triangle
    pub fn center_point(&self) -> Vector3<f32> {
        Vector3::new(
            (self.v0.x + self.v1.x + self.v2.x) / 3.0,
            (self.v0.y + self.v1.y + self.v2.y) / 3.0,
            (self.v0.z + self.v1.z + self.v2.z) / 3.0,
        )
    }

    pub fn order_by_y(&mut self) {
        let mut ordered = [self.v0, self.v1, self.v2];
        ordered.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        self.v0 = ordered[2];
        self.v1 = ordered[1];
        self.v2 = ordered[0];
    }

    /// Return true if the triangle is top-flat
    pub fn is_top_flat(&self) -> bool {
        self.v0.y as i32 == self.v1.y as i32
    }

    /// Return true if the triangle is bottom-flat
    pub fn is_bottom_flat(&self) -> bool {
        self.v1.y as i32 == self.v2.y as i32
    }

    /// Convert to usize type
    pub fn to_usize(&self) -> Triangle<Vector3<usize>>{
        Triangle {
            v0: Vector3::new(self.v0.x as usize, self.v0.y as usize, self.v0.z as usize),
            v1: Vector3::new(self.v1.x as usize, self.v1.y as usize, self.v1.z as usize),
            v2: Vector3::new(self.v2.x as usize, self.v2.y as usize, self.v2.z as usize),
        }
    }

    /// Convert to i64 type
    pub fn to_i64(&self) -> Triangle<Vector3<i64>> {
        Triangle {
            v0: Vector3::new(self.v0.x as i64, self.v0.y as i64, self.v0.z as i64),
            v1: Vector3::new(self.v1.x as i64, self.v1.y as i64, self.v1.z as i64),
            v2: Vector3::new(self.v2.x as i64, self.v2.y as i64, self.v2.z as i64),
        }
    }
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

fn build_view_matrix(eye: Vector3<f32>, lookat: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    // Rotate so that the line of sight from the eye position to the target maps to the z axis.
    // Camera up direction maps to y axis. x- axis is defined from the other two by cross
    // product

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
    /// Alpha value (0 - fully transparent, 255 - fully opaque)
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
    /// Z/depth buffer
    pub z_buffer: Box<[f32]>
}

impl DisplayBuffer {
    pub fn new(width: usize, height: usize, bpp: usize) -> DisplayBuffer {
        return DisplayBuffer {
            height: height,
            width: width,
            bpp: bpp,
            data: vec![0; width * height * bpp].into_boxed_slice(),
            z_buffer: vec![std::f32::MIN; width * height].into_boxed_slice(),
        };
    }

    /// return the size of the buffer in bytes
    pub fn size(&self) -> usize {
        return self.height * self.width * self.bpp;
    }

    /// return the number of pixels
    pub fn num_pixels(&self) -> usize {
        return self.height*self.width;
    }

    /// Reset the contents of the buffer so that all pixels are black
    pub fn clear(&mut self) {
        self.data = vec![0; self.width * self.height * self.bpp].into_boxed_slice();
        // this takes a lot of time when the initialization value is not 0.0
        self.z_buffer = vec![std::f32::MIN; self.width * self.height].into_boxed_slice();
        // faster version
        // unsafe {
            // libc::memset(
                // self.z_buffer.as_mut_ptr() as _,
                // std::f32::MIN as i32,
                // self.z_buffer.len() * mem::size_of::<f32>(),
            // );
        // }
    }

    /// Set a single pixel to a desired color
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate in pixels, value 0 corresponds to left edge
    /// * `y` - Y coordinate in pixels, value 0 correspoonds to bottom edge
    /// * 'color' - Color of the pixel
    pub fn set_pixel(&mut self, x: usize, y: usize, z: f32, color: Color) {
        assert!(x < self.width);
        assert!(y < self.height);
        let index: usize = (self.height - y - 1) * self.width + x;

        if index < self.num_pixels() {
            if self.z_buffer[index] < z {
                // always 1
                // println!("{} over {}", z, self.z_buffer[index]);

                self.z_buffer[index] = z;
                self.data[index*self.bpp] = color.r;
                self.data[index*self.bpp + 1] = color.g;
                self.data[index*self.bpp + 2] = color.b;
                self.data[index*self.bpp + 3] = color.a;
            }
        }
    }
}

/// A mesh is a collection of triangles that form a 3D surface
pub struct Mesh {
    /// Individual vertices that make up the surface of the mesh. Each vertex is
    /// a 4D vector [x, y, z, w]
    pub vertices: Vec<Vector4<f32>>,
    /// Specifies which vertices make a single polygon.
    pub poly_indices: Vec<[i32; 3]>,
    /// World position of the center of the mesh
    pub position: Vector4<f32>,
    /// Rotation of the mesh around all 3 axis vectors
    pub angle: Vector3<f32>,
    /// Triangles that make up the mesh surface
    pub triangles: Vec<Triangle<Vector4<f32>>>,
    /// Unit normal vectors of each triangle
    pub triangle_normals: Vec<Vector3<f32>>,
}

impl Mesh {
    pub fn new() -> Mesh {
        return Mesh {
            vertices: Vec::new(),
            poly_indices: Vec::new(),
            position: Vector4::new(0.0, 0.0, 0.0, 1.0),
            angle: Vector3::new(0.0, 0.0, 0.0),
            triangles: Vec::new(),
            triangle_normals: Vec::new(),
        };
    }

    /// Builds triangles out of the vertices of the mesh
    pub fn to_triangles(self: &mut Mesh) {
        for p in self.poly_indices.iter() {
            self.triangles.push(Triangle {
                v0: self.vertices[p[0] as usize],
                v1: self.vertices[p[1] as usize],
                v2: self.vertices[p[2] as usize],
            });
        }
    }

    /// Render a mesh into a display buffer
    ///
    /// # Arguments
    ///
    /// * `eye` - Position of the camera eye
    /// * `buffer` - Display buffer (render target)
    pub fn render(self: &Mesh, eye: Vector3<f32>, lookat: Vector3<f32>, buffer: &mut DisplayBuffer, color: Color) {
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
            build_view_matrix(eye, lookat, Vector3::new(0.0, 1.0, 0.0));
        let projection: Matrix4<f32> = build_perspective_matrix(0.1, 5.0, 78.0, aspect_ratio);

        // loop through all polygons, each consists of 3 vertices
        for (i, t) in self.triangles.iter().enumerate() {
            let triangle_world = t.transform(model);

            // lighting
            let reduce_dim = Matrix3x4::from_rows(
                &[
                    RowVector4::new(1.0, 0.0, 0.0, 0.0),
                    RowVector4::new(0.0, 1.0, 0.0, 0.0),
                    RowVector4::new(0.0, 0.0, 1.0, 0.0),
                ],
            );
            let triangle_world_3d = Triangle {
                v0: reduce_dim * triangle_world.v0,
                v1: reduce_dim * triangle_world.v1,
                v2: reduce_dim * triangle_world.v2,
            };

            // Light vector is a unit vector from the mesh to the light source.
            let n = self.triangle_normals[i];
            let n = model*Vector4::new(n.x, n.y, n.z, 0.0);
            let triangle_normal = Vector3::new(n.x, n.y, n.z);
            let light_vector = (eye - triangle_world_3d.center_point()).normalize();
            let brightness = light_vector.dot(&triangle_normal);

            // If the dot product is positive, the light is hitting the outer
            // surface of the mesh. In this case the value of the dot product
            // determines the intensity of the reflected light. If the dot
            // product is negative, the light is hitting the inner surface of
            // the mesh and we can simply ignore the triangle (not render it)
            if brightness > 0.0 {
                let color = Color {
                    r: (brightness*color.r as f32) as u8,
                    g: (brightness*color.g as f32) as u8,
                    b: (brightness*color.b as f32) as u8,
                    a: 255,
                };

                // Step 2: World to camera space
                let triangle_view = triangle_world.transform(view);
                // Step 3: Camera to clip space
                let triangle_camera = triangle_view.transform(projection);

                if triangle_camera.v0.z != 1.0 {
                    println!("camera {}", triangle_camera.v0.z)
                }

                // Step 4.2: PERSPECTIVE DIVIDE (normalization)
                // Perspective division, far away points moved closer to origin
                // To screen space. All visible points between [-1, 1].
                let t_ndc = Triangle {
                    v0: Vector3::new(
                        triangle_camera.v0.x / triangle_camera.v0.w,
                        triangle_camera.v0.y / triangle_camera.v0.w,
                        triangle_camera.v0.z,
                    ),
                    v1: Vector3::new(
                        triangle_camera.v1.x / triangle_camera.v1.w,
                        triangle_camera.v1.y / triangle_camera.v1.w,
                        triangle_camera.v1.z,
                    ),
                    v2: Vector3::new(
                        triangle_camera.v2.x / triangle_camera.v2.w,
                        triangle_camera.v2.y / triangle_camera.v2.w,
                        triangle_camera.v2.z,
                    ),
                };

                if t_ndc.v0.z != 1.0 {
                    println!("ndc {}", t_ndc.v0.z)
                }


                // Step 5: Viewport transform
                let mut t_viewport = Triangle {
                    v0: Vector3::new(
                        (1.0 + t_ndc.v0.x) * 0.5 * buffer.width as f32,
                        (1.0 + t_ndc.v0.y) * 0.5 * buffer.height as f32,
                        t_ndc.v0.z
                    ),
                    v1: Vector3::new(
                        (1.0 + t_ndc.v1.x) * 0.5 * buffer.width as f32,
                        (1.0 + t_ndc.v1.y) * 0.5 * buffer.height as f32,
                        t_ndc.v1.z
                    ),
                    v2: Vector3::new(
                        (1.0 + t_ndc.v2.x) * 0.5 * buffer.width as f32,
                        (1.0 + t_ndc.v2.y) * 0.5 * buffer.height as f32,
                        t_ndc.v2.z
                    ),
                };

                if t_viewport.v0.z != 1.0 {
                    println!("vp {}", t_viewport.v0.z)
                }


                t_viewport.order_by_y();
                t_viewport.render(color, buffer);
            }

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
    pub fn rotate(self: &mut Mesh, angle: Vector3<f32>) {
        self.angle.x = self.angle.x + angle.x;
        self.angle.y = self.angle.y + angle.y;
        self.angle.z = self.angle.z + angle.z;
    }
}
