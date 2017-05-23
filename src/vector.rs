use std::ops::Sub;

#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector2 {
    pub fn rotate(&mut self, a: f32){
        // transformation
        // unit vector i lands to cos(a)*i + sin(a)*j
        // unit vector j lands to -sin(a)*i + cos(a)*j
        // [cos(a), -sin(a)][x]
        // [sin(a),  cos(a)][y]
        //   ^        ^      ^
        // new i    new j  input

        let new_x = a.cos() * self.x + -1.0 * a.sin() * self.y;
        let new_y = a.sin() * self.x + a.cos() * self.y;
        self.x = new_x;
        self.y = new_y;
    }
}

impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}