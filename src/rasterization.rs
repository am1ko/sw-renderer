use core::{Color, DisplayBuffer};
use na::{Vector2};

pub fn draw_line(p1: Vector2<f32>, p2: Vector2<f32>, color: Color, buffer: &mut DisplayBuffer) {

    let threshold = 1.0;
    let sub = p2 - p1;
    let dist = (sub.x + sub.y).abs().sqrt();

    if dist > threshold {
        let middle = p1 + sub / 2.0;
        if (middle.x >= 0.0 && middle.x <= buffer.width as f32) &&
        (middle.y >= 0.0 && middle.y <= buffer.height as f32) {
            buffer.set_pixel(middle.x as usize, middle.y as usize, color);

            draw_line(p1, middle, color, buffer);
            draw_line(middle, p2, color, buffer);
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