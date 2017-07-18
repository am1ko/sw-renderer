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

fn order_by_y(p1: Vector2<usize>, p2: Vector2<usize>, p3: Vector2<usize>) -> [Vector2<usize>; 3] {
    let mut inputs = [p1, p2, p3];
    inputs.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    inputs
}

// p1 is the top vertex
fn fill_bottom_flat_triangle(p1: Vector2<usize>,
                             p2: Vector2<usize>,
                             p3: Vector2<usize>,
                             color: Color,
                             buffer: &mut DisplayBuffer) {
    let inv_slope_1 = (p1.x as f32 - p2.x as f32) / (p2.y as f32 - p1.y as f32);
    let inv_slope_2 = (p1.x as f32 - p3.x as f32) / (p2.y as f32 - p1.y as f32);

    let mut curr_x_1: f32 = p1.x as f32;
    let mut curr_x_2: f32 = p1.x as f32;

    for y in (p2.y..p1.y + 1).rev() {
        let scan_line_start: Vector2<usize> = Vector2::new(curr_x_1 as usize, y);
        let scan_line_end: Vector2<usize> = Vector2::new(curr_x_2 as usize, y);
        draw_line_usize(scan_line_start, scan_line_end, color, buffer);

        curr_x_1 += inv_slope_1;
        curr_x_2 += inv_slope_2;
    }
}

// p3 is the bottom vertex
fn fill_top_flat_triangle(p1: Vector2<usize>,
                          p2: Vector2<usize>,
                          p3: Vector2<usize>,
                          color: Color,
                          buffer: &mut DisplayBuffer) {
    let inv_slope_1 = (p1.x as f32 - p3.x as f32) / (p3.y as f32 - p1.y as f32);
    let inv_slope_2 = (p2.x as f32 - p3.x as f32) / (p3.y as f32 - p2.y as f32);

    let mut curr_x_1 = p3.x as f32;
    let mut curr_x_2 = p3.x as f32;

    for y in p3.y..p1.y + 1 {
        let scan_line_start: Vector2<usize> = Vector2::new(curr_x_1 as usize, y);
        let scan_line_end: Vector2<usize> = Vector2::new(curr_x_2 as usize, y);
        draw_line_usize(scan_line_start, scan_line_end, color, buffer);

        curr_x_1 -= inv_slope_1;
        curr_x_2 -= inv_slope_2;
    }
}
pub fn draw_triangle_usize(p1: Vector2<usize>,
                           p2: Vector2<usize>,
                           p3: Vector2<usize>,
                           color: Color,
                           buffer: &mut DisplayBuffer) {
    let ordered = order_by_y(p1, p2, p3);
    let p1 = ordered[2];
    let p2 = ordered[1];
    let p3 = ordered[0];

    if ordered[1].y == ordered[2].y {
        fill_top_flat_triangle(ordered[2], ordered[1], ordered[0], color, buffer);
    } else if ordered[0].y == ordered[1].y {
        fill_bottom_flat_triangle(ordered[2], ordered[1], ordered[0], color, buffer);
    } else {
        let pf1: Vector2<f32> = Vector2::new(p1.x as f32, p1.y as f32);
        let pf2: Vector2<f32> = Vector2::new(p2.x as f32, p2.y as f32);
        let pf3: Vector2<f32> = Vector2::new(p3.x as f32, p3.y as f32);

        // split the triangle into two: a bottom flat one and a top flat one
        let x4 = (pf1.x + (pf1.y - pf2.y) / (pf1.y - pf3.y) * (pf3.x - pf1.x)) as usize;
        let p4: Vector2<usize> = Vector2::new(x4, p2.y);
        fill_bottom_flat_triangle(p1, p2, p4, color, buffer);
        fill_top_flat_triangle(p2, p4, p3, color, buffer);
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
    // fn test_draw_line_usize() {
    // crashes for some reason
    // let p1: Vector2<usize> = Vector2::new(0, 1);
    // let p2: Vector2<usize> = Vector2::new(6, 4);
    // let mut db = DisplayBuffer::new(1024, 768, 1);
    // let color = Color {
    //     r: 1,
    //     g: 0,
    //     b: 0,
    //     a: 0,
    // };
    //
    // super::draw_line_usize(p1, p2, color, &mut db);
    //
    // assert_eq!(db.data[0], 1);
    //
    #[test]
    fn test_draw_triangle() {
        let p1 = Vector2::new(1, 1);
        let p2 = Vector2::new(1, 2);
        let p3 = Vector2::new(1, 3);

        let result = super::order_by_y(p2, p3, p1);
        assert!(result[0].y == 1);
        assert!(result[1].y == 2);
        assert!(result[2].y == 3);
    }
}
