use core::{Color, DisplayBuffer, Face, Renderable};
use na::{Vector2, Vector3};

/// Get barycentric coordinates for a point P with respect to a triangle ABC
///
/// # Arguments
///
/// * 'a' Vertex A of the triangle ABC
/// * 'b' Vertex B of the triangle ABC
/// * 'c' Vertex C of the triangle ABC
/// * 'p' Point P for which to calculate the barycentric coordinates
///
/// Barycentric coordinates (u, v, w) are defined such that uA + vB + wC = P
/// Some useful properties
/// - If u, v, w all are >= 0 then point P is inside the triangle ABC
/// - If any of u, v, w is < 0 then point P is outside the triangle ABC
/// - u, v, w can be used to interpolate the vertex attributes inside the triangle
/// - u + v + w = 1
///
fn get_barycentric(
    a: Vector2<f32>,
    b: Vector2<f32>,
    c: Vector2<f32>,
    p: Vector2<f32>,
) -> (f32, f32, f32) {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;

    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d11 = v1.dot(&v1);
    let d20 = v2.dot(&v0);
    let d21 = v2.dot(&v1);
    let denom = d00 * d11 - d01 * d01;

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    (u, v, w)
}

impl Renderable for Face<Vector3<f32>> {
    /// Draw a color-filled face
    fn render(&self, buffer: &mut DisplayBuffer) {
        // Bounding box for the triangle
        let all_x = [self.v0.position.x, self.v1.position.x, self.v2.position.x];
        let all_y = [self.v0.position.y, self.v1.position.y, self.v2.position.y];
        let min_x = all_x.iter().fold(std::f32::MAX, |a, &b| a.min(b)) as usize;
        let max_x = all_x.iter().fold(std::f32::MIN, |a, &b| a.max(b)) as usize;
        let min_y = all_y.iter().fold(std::f32::MAX, |a, &b| a.min(b)) as usize;
        let max_y = all_y.iter().fold(std::f32::MIN, |a, &b| a.max(b)) as usize;

        if max_x >= buffer.width || max_y >= buffer.height {
            return;
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let v0 = self.v0.position.remove_row(2);
                let v1 = self.v1.position.remove_row(2);
                let v2 = self.v2.position.remove_row(2);
                let p = Vector2::new(x as f32, y as f32);

                let (w0, w1, w2) = get_barycentric(v0, v1, v2, p);
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let z =
                        w0 * self.v0.position.z + w1 * self.v1.position.z + w2 * self.v2.position.z;
                    let color = Color {
                        r: (w0 * self.v0.color.r as f32
                            + w1 * self.v1.color.r as f32
                            + w2 * self.v2.color.r as f32) as u8,
                        g: (w0 * self.v0.color.g as f32
                            + w1 * self.v1.color.g as f32
                            + w2 * self.v2.color.g as f32) as u8,
                        b: (w0 * self.v0.color.b as f32
                            + w1 * self.v1.color.b as f32
                            + w2 * self.v2.color.b as f32) as u8,
                        a: 255,
                    };
                    buffer.set_pixel(x, y, z, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_barycentric_ccw_inside() {
        let a = Vector2::new(1.0, 0.0);
        let b = Vector2::new(0.0, 1.0);
        let c = Vector2::new(-1.0, 0.0);
        let p = Vector2::new(0.0, 0.5);

        let (w0, w1, w2) = get_barycentric(a, b, c, p);

        assert!(w0 > 0.0);
        assert!(w1 > 0.0);
        assert!(w2 > 0.0);
        assert!(w0 < 1.0);
        assert!(w1 < 1.0);
        assert!(w2 < 1.0);
    }

    #[test]
    fn test_get_barycentric_cw_inside() {
        let a = Vector2::new(-1.0, 0.0);
        let b = Vector2::new(0.0, 1.0);
        let c = Vector2::new(1.0, 0.0);
        let p = Vector2::new(0.0, 0.5);

        let (w0, w1, w2) = get_barycentric(a, b, c, p);

        assert!(w0 > 0.0);
        assert!(w1 > 0.0);
        assert!(w2 > 0.0);
        assert!(w0 < 1.0);
        assert!(w1 < 1.0);
        assert!(w2 < 1.0);
    }
}
