use core::{Color, DisplayBuffer};
use na::Vector2;

pub fn draw_line_f32(p1: Vector2<f32>,
                     p2: Vector2<f32>,
                     color: Color,
                     buffer: &mut DisplayBuffer) {

    let threshold = 1.0;
    let sub = p2 - p1;
    let dist = (sub.x + sub.y).abs().sqrt();

    if dist > threshold {
        let middle = p1 + sub / 2.0;
        if (middle.x >= 0.0 && middle.x <= buffer.width as f32) &&
           (middle.y >= 0.0 && middle.y <= buffer.height as f32) {
            buffer.set_pixel(middle.x as usize, middle.y as usize, color);

            draw_line_f32(p1, middle, color, buffer);
            draw_line_f32(middle, p2, color, buffer);
        }
    }
}

pub fn draw_line_usize(p1: Vector2<usize>,
                       p2: Vector2<usize>,
                       color: Color,
                       buffer: &mut DisplayBuffer) {
    if p1.x >= buffer.width || p1.y >= buffer.height {
        return;
    }
    if p2.x >= buffer.width || p2.y >= buffer.height {
        return;
    }

    let mut x = p1.x as i32;
    let mut y = p1.y as i32;
    let x2 = p2.x as i32;
    let y2 = p2.y as i32;
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

#[cfg(test)]
mod test {
    extern crate core;
    extern crate nalgebra as na;

    use core::{Color, DisplayBuffer};
    use na::Vector2;
    #[test]
    fn test_draw_line_usize() {
        let p1: Vector2<usize> = Vector2::new(0, 1);
        let p2: Vector2<usize> = Vector2::new(6, 4);
        let mut db = DisplayBuffer::new(1024, 768, 1);
        let color = Color {
            r: 1,
            g: 0,
            b: 0,
            a: 0,
        };

        super::draw_line_usize(p1, p2, color, &mut db);

        assert_eq!(db.data[0], 1);
    }
}
