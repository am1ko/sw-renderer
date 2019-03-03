use core::{Color, DisplayBuffer, LineSegment, Renderable, Triangle};
use na::Vector2;

impl Renderable for Triangle<Vector2<usize>> {
    /// Draw a color-filled triangle
    fn render(&self, color: Color, buffer: &mut DisplayBuffer) {
        let p1 = self.v0;
        let p2 = self.v1;
        let p3 = self.v2;

        if p1.x >= buffer.width || p1.y >= buffer.height || p2.x >= buffer.width ||
            p2.y >= buffer.height || p3.x >= buffer.width || p3.y >= buffer.height
        {
            return;
        }

        if self.is_top_flat() {
            fill_top_flat_triangle(self, color, buffer);
        } else if self.is_bottom_flat() {
            fill_bottom_flat_triangle(self, color, buffer);
        } else {
            let pf1: Vector2<f32> = Vector2::new(p1.x as f32, p1.y as f32);
            let pf2: Vector2<f32> = Vector2::new(p2.x as f32, p2.y as f32);
            let pf3: Vector2<f32> = Vector2::new(p3.x as f32, p3.y as f32);

            // split the triangle into two: a bottom flat one and a top flat one
            let x4 = (pf1.x + (pf1.y - pf2.y) / (pf1.y - pf3.y) * (pf3.x - pf1.x)) as usize;
            let p4: Vector2<usize> = Vector2::new(x4, p2.y);

            let bottom_flat = Triangle {
                v0: p1,
                v1: p2,
                v2: p4,
            };
            let top_flat = Triangle {
                v0: p2,
                v1: p4,
                v2: p3,
            };

            fill_bottom_flat_triangle(&bottom_flat, color, buffer);
            fill_top_flat_triangle(&top_flat, color, buffer);
        }
    }
}

impl Renderable for LineSegment<Vector2<usize>> {
    /// Draw a colored line segment between two points
    fn render(&self, color: Color, buffer: &mut DisplayBuffer) {
        if self.v0.x >= buffer.width || self.v0.y >= buffer.height {
            return;
        }
        if self.v1.x >= buffer.width || self.v1.y >= buffer.height {
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
            buffer.set_pixel(x as usize, y as usize, color);

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

impl Renderable for LineSegment<Vector2<f32>> {
    /// Draw a colored line segment between two points
    fn render(&self, color: Color, buffer: &mut DisplayBuffer) {
        let threshold = 1.0;
        let sub = self.v1 - self.v0;
        let dist = (sub.x + sub.y).abs().sqrt();

        if dist > threshold {
            let middle = self.v0 + sub / 2.0;
            if (middle.x >= 0.0 && middle.x <= buffer.width as f32) &&
                (middle.y >= 0.0 && middle.y <= buffer.height as f32)
            {
                buffer.set_pixel(middle.x as usize, middle.y as usize, color);

                let first = LineSegment {
                    v0: self.v0,
                    v1: middle,
                };
                let second = LineSegment {
                    v0: middle,
                    v1: self.v1,
                };

                first.render(color, buffer);
                second.render(color, buffer);
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
    triangle: &Triangle<Vector2<usize>>,
    color: Color,
    buffer: &mut DisplayBuffer,
) {
    let v0 = triangle.v0;
    let v1 = triangle.v1;
    let v2 = triangle.v2;
    let inv_slope_1 = (v0.x as f32 - v1.x as f32) / (v1.y as f32 - v0.y as f32);
    let inv_slope_2 = (v0.x as f32 - v2.x as f32) / (v1.y as f32 - v0.y as f32);

    let mut curr_x_1: f32 = v0.x as f32;
    let mut curr_x_2: f32 = v0.x as f32;

    for y in (v1.y..v0.y + 1).rev() {
        let scan_line_start: Vector2<usize> = Vector2::new(curr_x_1 as usize, y);
        let scan_line_end: Vector2<usize> = Vector2::new(curr_x_2 as usize, y);

        let scan_line = LineSegment {
            v0: scan_line_start,
            v1: scan_line_end,
        };

        scan_line.render(color, buffer);

        curr_x_1 += inv_slope_1;
        curr_x_2 += inv_slope_2;
    }
}

/// Draw a filled top-flat triangle
///
/// * `triangle` - Triangle to fill
/// * `color` - Color of the triangle
/// * `buffer` - Display buffer (render target)
fn fill_top_flat_triangle(
    triangle: &Triangle<Vector2<usize>>,
    color: Color,
    buffer: &mut DisplayBuffer,
) {
    let v0 = triangle.v0;
    let v1 = triangle.v1;
    let v2 = triangle.v2;
    let inv_slope_1 = (v0.x as f32 - v2.x as f32) / (v2.y as f32 - v0.y as f32);
    let inv_slope_2 = (v1.x as f32 - v2.x as f32) / (v2.y as f32 - v1.y as f32);

    let mut curr_x_1 = v2.x as f32;
    let mut curr_x_2 = v2.x as f32;

    for y in v2.y..v0.y + 1 {
        let scan_line_start: Vector2<usize> = Vector2::new(curr_x_1 as usize, y);
        let scan_line_end: Vector2<usize> = Vector2::new(curr_x_2 as usize, y);

        let scan_line = LineSegment {
            v0: scan_line_start,
            v1: scan_line_end,
        };

        scan_line.render(color, buffer);

        curr_x_1 -= inv_slope_1;
        curr_x_2 -= inv_slope_2;
    }
}
