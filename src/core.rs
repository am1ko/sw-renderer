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

/// Renderable represents any model that can be drawn to a display buffer
pub trait Renderable {
    /// Draw the model to a display buffer (render target)
    ///
    /// * `buffer` - Display buffer (render target)
    fn render(&self, buffer: &mut DisplayBuffer);
}

#[derive(Copy, Clone)]
pub struct Vertex<T: Copy> {
    /// Color of the vertex
    pub color: Color,
    /// Position of the vertex
    pub position: T,
    /// Normal vector of the vertex
    pub normal: Vector3<f32>,
}

pub struct Face<T: Copy> {
    /// Vertex of a triangle
    pub v0: Vertex<T>,
    /// Vertex of a triangle
    pub v1: Vertex<T>,
    /// Vertex of a triangle
    pub v2: Vertex<T>,
}

impl Face<Vector4<f32>> {
    /// Perform a linear transformation to all vertices of the triangle
    pub fn transform(&self, m: Matrix4<f32>) -> Face<Vector4<f32>> {
        // Normal vectors cannot simply be transformed with the matrix m like
        // vertex coordinates. Instead the scales must be inverted. So when we
        // scale the vertices by factor x in any axis, we must scale the normals
        // by 1/x. This is achieved by transforming the normals using the
        // inverse transpose of matrix m
        let m_normal = m
            .fixed_slice::<nalgebra::U3, nalgebra::U3>(0, 0)
            .try_inverse()
            .expect("Could not invert matrix")
            .transpose();

        Face {
            v0: Vertex {
                position: m * self.v0.position,
                color: self.v0.color,
                normal: m_normal * self.v0.normal,
            },
            v1: Vertex {
                position: m * self.v1.position,
                color: self.v1.color,
                normal: m_normal * self.v1.normal,
            },
            v2: Vertex {
                position: m * self.v2.position,
                color: self.v2.color,
                normal: m_normal * self.v2.normal,
            },
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

    return Matrix4::from_rows(&[
        RowVector4::new(2.0 * n / (r - l), 0.0, (r + l) / (r - l), 0.0),
        RowVector4::new(0.0, 2.0 * n / (t - b), (t + b) / (t - b), 0.0),
        RowVector4::new(0.0, 0.0, -(f + n) / (f - n), -(2.0 * f * n) / (f - n)),
        RowVector4::new(0.0, 0.0, -1.0, 0.0),
    ]);
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
    let rotation = Matrix4::from_rows(&[
        RowVector4::new(x.x, x.y, x.z, 0.0),
        RowVector4::new(y.x, y.y, y.z, 0.0),
        RowVector4::new(z.x, z.y, z.z, 0.0),
        RowVector4::new(0.0, 0.0, 0.0, 1.0),
    ]);

    // Translate to the inverse of the eye position (the world moves in the opposite direction
    // around the camera that is fixed)
    let translation = Matrix4::from_rows(&[
        RowVector4::new(1.0, 0.0, 0.0, -eye.x),
        RowVector4::new(0.0, 1.0, 0.0, -eye.y),
        RowVector4::new(0.0, 0.0, 1.0, -eye.z),
        RowVector4::new(0.0, 0.0, 0.0, 1.0),
    ]);

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

impl Color {
    pub fn to_u32(&self) -> u32 {
        return ((self.a as u32) << 24) | ((self.b as u32) << 16) | ((self.g as u32) << 8) | (self.r as u32);
    }
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
    pub z_buffer: Box<[f32]>,
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
        return self.height * self.width;
    }

    /// Reset the contents of the buffer so that all pixels are black
    pub fn clear(&mut self) {
        self.data = vec![0; self.width * self.height * self.bpp].into_boxed_slice();
        // this takes a lot of time when the initialization value is not 0.0
        self.z_buffer = vec![std::f32::MIN; self.width * self.height].into_boxed_slice();
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
                self.z_buffer[index] = z;
                self.data[index * self.bpp] = color.r;
                self.data[index * self.bpp + 1] = color.g;
                self.data[index * self.bpp + 2] = color.b;
                self.data[index * self.bpp + 3] = color.a;
            }
        }
    }
}

/// A mesh is a collection of triangles that form a 3D surface
pub struct Mesh {
    /// World position of the center of the mesh
    pub position: Vector4<f32>,
    /// Rotation of the mesh around all 3 axis vectors
    pub angle: Vector3<f32>,
    /// Triangle faces that make up the mesh surface
    pub faces: Vec<Face<Vector4<f32>>>,
}

impl Mesh {
    pub fn new() -> Mesh {
        return Mesh {
            position: Vector4::new(0.0, 0.0, 0.0, 1.0),
            angle: Vector3::new(0.0, 0.0, 0.0),
            faces: Vec::new(),
        };
    }

    /// Render a mesh into a display buffer
    ///
    /// # Arguments
    ///
    /// * `eye` - Position of the camera eye
    /// * 'lookat' - Focus point of the eye
    /// * `buffer` - Display buffer (render target)
    pub fn render(
        self: &Mesh,
        eye: Vector3<f32>,
        lookat: Vector3<f32>,
        buffer: &mut DisplayBuffer,
    ) {
        let m_rot_x = Matrix4::from_rows(&[
            RowVector4::new(1.0, 0.0, 0.0, 0.0),
            RowVector4::new(0.0, self.angle.x.cos(), self.angle.x.sin(), 0.0),
            RowVector4::new(0.0, -self.angle.x.sin(), self.angle.x.cos(), 0.0),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ]);
        let m_rot_y = Matrix4::from_rows(&[
            RowVector4::new(self.angle.y.cos(), 0.0, -self.angle.y.sin(), 0.0),
            RowVector4::new(0.0, 1.0, 0.0, 0.0),
            RowVector4::new(self.angle.y.sin(), 0.0, self.angle.y.cos(), 0.0),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ]);
        let m_rot_z = Matrix4::from_rows(&[
            RowVector4::new(self.angle.z.cos(), -self.angle.z.sin(), 0.0, 0.0),
            RowVector4::new(self.angle.z.sin(), self.angle.z.cos(), 0.0, 0.0),
            RowVector4::new(0.0, 0.0, 1.0, 0.0),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ]);

        let m_trans = Matrix4::from_rows(&[
            RowVector4::new(1.0, 0.0, 0.0, self.position.x),
            RowVector4::new(0.0, 1.0, 0.0, self.position.y),
            RowVector4::new(0.0, 0.0, 1.0, self.position.z),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ]);

        let model = m_trans * m_rot_z * m_rot_y * m_rot_x;
        let aspect_ratio = (buffer.width as f32) / (buffer.height as f32);
        let view: Matrix4<f32> = build_view_matrix(eye, lookat, Vector3::new(0.0, 1.0, 0.0));
        let projection: Matrix4<f32> = build_perspective_matrix(0.1, 5.0, 78.0, aspect_ratio);

        for t in self.faces.iter() {
            let face_world = t.transform(model);
            let reduce_dim = Matrix3x4::from_rows(&[
                RowVector4::new(1.0, 0.0, 0.0, 0.0),
                RowVector4::new(0.0, 1.0, 0.0, 0.0),
                RowVector4::new(0.0, 0.0, 1.0, 0.0),
            ]);
            let triangle_world_3d = Face {
                v0: Vertex {
                    position: reduce_dim * face_world.v0.position,
                    color: face_world.v0.color,
                    normal: face_world.v0.normal,
                },
                v1: Vertex {
                    position: reduce_dim * face_world.v1.position,
                    color: face_world.v1.color,
                    normal: face_world.v1.normal,
                },
                v2: Vertex {
                    position: reduce_dim * face_world.v2.position,
                    color: face_world.v2.color,
                    normal: face_world.v2.normal,
                },
            };

            // Light vector is a unit vector from the mesh to the light source.
            let brightness_v0 = (eye - triangle_world_3d.v0.position)
                .normalize()
                .dot(&triangle_world_3d.v0.normal);
            let brightness_v1 = (eye - triangle_world_3d.v1.position)
                .normalize()
                .dot(&triangle_world_3d.v1.normal);
            let brightness_v2 = (eye - triangle_world_3d.v2.position)
                .normalize()
                .dot(&triangle_world_3d.v2.normal);
            assert!(brightness_v0 <= 1.0);
            assert!(brightness_v1 <= 1.0);
            assert!(brightness_v2 <= 1.0);

            // If the dot product is positive, the light is hitting the outer
            // surface of the mesh. In this case the value of the dot product
            // determines the intensity of the reflected light. If the dot
            // product is negative, the light is hitting the inner surface of
            // the mesh and we can simply ignore the triangle (not render it)
            if brightness_v0 > 0.0 || brightness_v1 > 0.0 || brightness_v2 > 0.0 {
                // Step 2: World to camera space
                let triangle_view = face_world.transform(view);

                // Step 3: Camera to clip space
                let triangle_camera = triangle_view.transform(projection);

                // Step 4.2: PERSPECTIVE DIVIDE (normalization)
                // Perspective division, far away points moved closer to origin
                // To screen space. All visible points between [-1, 1].
                let t_ndc = Face {
                    v0: Vertex {
                        position: Vector3::new(
                            triangle_camera.v0.position.x / triangle_camera.v0.position.w,
                            triangle_camera.v0.position.y / triangle_camera.v0.position.w,
                            triangle_camera.v0.position.z,
                        ),
                        color: Color {
                            r: (triangle_camera.v0.color.r as f32 * brightness_v0) as u8,
                            g: (triangle_camera.v0.color.g as f32 * brightness_v0) as u8,
                            b: (triangle_camera.v0.color.b as f32 * brightness_v0) as u8,
                            a: (triangle_camera.v0.color.a as f32 * brightness_v0) as u8,
                        },
                        normal: triangle_camera.v0.normal,
                    },
                    v1: Vertex {
                        position: Vector3::new(
                            triangle_camera.v1.position.x / triangle_camera.v1.position.w,
                            triangle_camera.v1.position.y / triangle_camera.v1.position.w,
                            triangle_camera.v1.position.z,
                        ),
                        color: Color {
                            r: (triangle_camera.v1.color.r as f32 * brightness_v1) as u8,
                            g: (triangle_camera.v1.color.g as f32 * brightness_v1) as u8,
                            b: (triangle_camera.v1.color.b as f32 * brightness_v1) as u8,
                            a: (triangle_camera.v1.color.a as f32 * brightness_v1) as u8,
                        },
                        normal: triangle_camera.v1.normal,
                    },
                    v2: Vertex {
                        position: Vector3::new(
                            triangle_camera.v2.position.x / triangle_camera.v2.position.w,
                            triangle_camera.v2.position.y / triangle_camera.v2.position.w,
                            triangle_camera.v2.position.z,
                        ),
                        color: Color {
                            r: (triangle_camera.v2.color.r as f32 * brightness_v2) as u8,
                            g: (triangle_camera.v2.color.g as f32 * brightness_v2) as u8,
                            b: (triangle_camera.v2.color.b as f32 * brightness_v2) as u8,
                            a: (triangle_camera.v2.color.a as f32 * brightness_v2) as u8,
                        },
                        normal: triangle_camera.v2.normal,
                    },
                };

                // Step 5: Viewport transform
                let t_viewport = Face {
                    v0: Vertex {
                        position: Vector3::new(
                            (1.0 + t_ndc.v0.position.x) * 0.5 * buffer.width as f32,
                            (1.0 + t_ndc.v0.position.y) * 0.5 * buffer.height as f32,
                            t_ndc.v0.position.z,
                        ),
                        color: t_ndc.v0.color,
                        normal: t_ndc.v0.normal,
                    },
                    v1: Vertex {
                        position: Vector3::new(
                            (1.0 + t_ndc.v1.position.x) * 0.5 * buffer.width as f32,
                            (1.0 + t_ndc.v1.position.y) * 0.5 * buffer.height as f32,
                            t_ndc.v1.position.z,
                        ),
                        color: t_ndc.v1.color,
                        normal: t_ndc.v1.normal,
                    },
                    v2: Vertex {
                        position: Vector3::new(
                            (1.0 + t_ndc.v2.position.x) * 0.5 * buffer.width as f32,
                            (1.0 + t_ndc.v2.position.y) * 0.5 * buffer.height as f32,
                            t_ndc.v2.position.z,
                        ),
                        color: t_ndc.v2.color,
                        normal: t_ndc.v2.normal,
                    },
                };

                t_viewport.render(buffer);
            }
        }
    }

    /// Translate (move) a mesh in space
    ///
    /// # Arguments
    ///
    /// * `translation` - Vector that specifies the displacement
    pub fn translate(self: &mut Mesh, translation: Vector3<f32>) {
        let xform = Matrix4::from_rows(&[
            RowVector4::new(1.0, 0.0, 0.0, translation.x),
            RowVector4::new(0.0, 1.0, 0.0, translation.y),
            RowVector4::new(0.0, 0.0, 1.0, translation.z),
            RowVector4::new(0.0, 0.0, 0.0, 1.0),
        ]);
        self.position = xform * self.position;
    }

    /// Rotate a mesh
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle around each cartesian axis in radians
    pub fn rotate(self: &mut Mesh, angle: Vector3<f32>) {
        self.angle.x = self.angle.x + angle.x;
        self.angle.y = self.angle.y + angle.y;
        self.angle.z = self.angle.z + angle.z;
    }
}
