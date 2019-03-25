use core::{Color, DisplayBuffer, LineSegment, Renderable, Triangle};
use na::{Vector3};

/// Interpolate a 1D function
///
/// * 'x' X-coordinate at which to find the interpolated Y-value
/// * 'y1' Y-value at x=x1
/// * 'y0' Y-value at x=x0
/// * 'x0' Lower bound of X
/// * 'x1' Upper bound of X
fn interpolate(x: f32, y1: f32, y0: f32, x1: f32, x0: f32) -> f32 {
    y0 + (x - x0)*(y1 - y0)/(x1-x0)
}

impl Renderable for Triangle<Vector3<f32>> {
    /// Draw a color-filled triangle
    fn render(&self, color: Color, buffer: &mut DisplayBuffer) {
        let triangle = self.to_usize();

        let p1 = triangle.v0;
        let p2 = triangle.v1;
        let p3 = triangle.v2;

        if p1.x >= buffer.width || p1.y >= buffer.height || p2.x >= buffer.width ||
            p2.y >= buffer.height || p3.x >= buffer.width || p3.y >= buffer.height {
            return;
        }

        if self.is_top_flat() {
            fill_top_flat_triangle(&self, color, buffer);
        } else if self.is_bottom_flat() {
            fill_bottom_flat_triangle(&self, color, buffer);
        } else {
            // split the triangle into two: a bottom flat one and a top flat one
            let x4 = self.v0.x + (self.v0.y - self.v1.y) / (self.v0.y - self.v2.y) * (self.v2.x - self.v0.x);
            let v_middle = Vector3::new(x4, self.v1.y, self.v1.z);

            let bottom_flat = Triangle {
                v0: self.v0,
                v1: self.v1,
                v2: v_middle,
            };
            let top_flat = Triangle {
                v0: self.v1,
                v1: v_middle,
                v2: self.v2,
            };

            fill_bottom_flat_triangle(&bottom_flat, color, buffer);
            fill_top_flat_triangle(&top_flat, color, buffer);
        }
    }
}

impl Renderable for LineSegment<Vector3<f32>> {
    /// Draw a colored line segment between two points
    fn render(&self, color: Color, buffer: &mut DisplayBuffer) {
        if self.v0.x as usize >= buffer.width || self.v0.y as usize >= buffer.height {
            return;
        }
        if self.v1.x as usize >= buffer.width || self.v1.y as usize >= buffer.height {
            return;
        }

        let mut x = self.v0.x as i32;
        let mut y = self.v0.y as i32;
        let x2 = self.v1.x as i32;
        let y2 = self.v1.y as i32;
        let dx = (x2 - x).abs();
        let dy = (y2 - y).abs();
        let sx = if x < x2 { 1 } else { -1 };
        let sy = if y < y2 { 1 } else { -1 };
        let mut err: i32 = dx - dy;

        loop {
            // todo z interpolation
            buffer.set_pixel(x as usize, y as usize, self.v0.z, color);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx
            }
            if e2 < dx {
                err += dx;
                y += sy
            }
        }
    }
}

/// Draw a filled bottom-flat triangle
///
/// * `triangle` - Triangle to fill
/// * `color` - Color of the triangle
/// * `buffer` - Display buffer (render target)
fn fill_bottom_flat_triangle(
    triangle: &Triangle<Vector3<f32>>,
    color: Color,
    buffer: &mut DisplayBuffer,
) {
    // Top vertex
    let v0 = triangle.to_usize().v0;
    // Bottom left vertex
    let v1 = if triangle.v1.x < triangle.v2.x { triangle.to_usize().v1 } else {triangle.to_usize().v2};
    // Bottom right vertex
    let v2 = if triangle.v1.x < triangle.v2.x { triangle.to_usize().v2 } else {triangle.to_usize().v1};

    // dx / dy for both edges of the triangle
    let inv_slope_1 = (v0.x as f32 - v1.x as f32) / (v1.y as f32 - v0.y as f32);
    let inv_slope_2 = (v0.x as f32 - v2.x as f32) / (v1.y as f32 - v0.y as f32);

    // Scan line starting x-coordinate
    let mut x_start: f32 = v0.x as f32;
    // Scan line ending x-coordinate
    let mut x_end: f32 = v0.x as f32;

    for y in (v1.y..=v0.y).rev() {
        // Interpolate the z-coordinate between two vertices of the triangle
        let z_start = interpolate(y as f32, v0.z as f32, v1.z as f32, v0.y as f32, v1.y as f32);
        let z_end = interpolate(y as f32, v0.z as f32, v2.z as f32, v0.y as f32, v2.y as f32);
        let scan_line_start = Vector3::new(x_start, y as f32, z_start);
        let scan_line_end = Vector3::new(x_end, y as f32, z_end);

        let scan_line = LineSegment {
            v0: scan_line_start,
            v1: scan_line_end,
        };

        scan_line.render(color, buffer);

        x_start += inv_slope_1;
        x_end += inv_slope_2;
    }
}

/// Draw a filled top-flat triangle
///
/// * `triangle` - Triangle to fill
/// * `color` - Color of the triangle
/// * `buffer` - Display buffer (render target)
fn fill_top_flat_triangle(
    triangle: &Triangle<Vector3<f32>>,
    color: Color,
    buffer: &mut DisplayBuffer,
) {
    // Top left vertex
    let v0 = if triangle.v0.x < triangle.v1.x { triangle.to_usize().v0 } else {triangle.to_usize().v1};
    // Top right vertex
    let v1 = if triangle.v0.x < triangle.v1.x { triangle.to_usize().v1 } else {triangle.to_usize().v0};
    // Bottom vertex
    let v2 = triangle.to_usize().v2;

    // dx / dy for both edges of the triangle
    let inv_slope_1 = (v0.x as f32 - v2.x as f32) / (v2.y as f32 - v0.y as f32);
    let inv_slope_2 = (v1.x as f32 - v2.x as f32) / (v2.y as f32 - v1.y as f32);

    let mut x_start = v2.x as f32;
    let mut x_end = v2.x as f32;

    // Loop y from bottom to top of the triangle
    for y in v2.y..=v0.y {
        let z_start = interpolate(y as f32, v0.z as f32, v2.z as f32, v0.y as f32, v2.y as f32);
        let z_end = interpolate(y as f32, v1.z as f32, v2.z as f32, v1.y as f32, v2.y as f32);
        let scan_line_start = Vector3::new(x_start, y as f32, z_start);
        let scan_line_end = Vector3::new(x_end, y as f32, z_end);

        let scan_line = LineSegment {
            v0: scan_line_start,
            v1: scan_line_end,
        };

        scan_line.render(color, buffer);

        x_start -= inv_slope_1;
        x_end -= inv_slope_2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate() {
        // Case: Interpolate y = 0.5x at x: 1.0
        // Expect: Y-value close to 0.5
        let y = interpolate(1.0, 1.0, 0.0, 2.0, 0.0);
        assert!(y > 0.495);
        assert!(y < 0.505);
    }
}
