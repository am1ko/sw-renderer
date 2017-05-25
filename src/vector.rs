use std::ops::Sub;
use std::ops::Mul;

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

/*
pub struct Vector {
    dim: i32,
    data: Vec<f32>
}
*/

#[derive(Clone)]
pub struct Matrix {
    cols: i32,
    rows: i32,
    data: Vec<f32>
    //data: [f32, ..6] does not work
}

impl Matrix {
    pub fn identity(dim: i32) -> Matrix{
        let mut ret = Matrix::new(dim, dim);

        for c in 0..dim{
            for r in 0..dim{
                let i = (c*ret.rows + r) as usize;
                if c == r{
                    ret.data[i] = 1.0;
                }
                else {
                    ret.data[i] = 0.0;
                }
            }
        }

        return ret;
    }

    pub fn new(cols: i32, rows: i32) -> Matrix{
        let ret = Matrix{cols: cols, rows: rows, data: vec![0.0;(cols*rows) as usize]};
        return ret;
    }
}

impl Mul<f32> for Matrix {
    type Output = Matrix;
    fn mul(self, other: f32) -> Matrix {
        let mut ret = self.clone();

        // column-major order
        for c in 0..ret.cols{
            for r in 0..ret.rows {
                let i = (c*ret.rows + r) as usize;
                ret.data[i] = ret.data[i]*other;
            }
        }

        return ret;
    }
}

/*
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
*/

impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[test]
fn test_matrix_new() {
    // Arrange
    let m1 = Matrix::identity(3);

    // Assert
    assert_eq!(m1.data[0], 1.0);
    assert_eq!(m1.data[1], 0.0);
    assert_eq!(m1.data[2], 0.0);

    assert_eq!(m1.data[3], 0.0);
    assert_eq!(m1.data[4], 1.0);
    assert_eq!(m1.data[5], 0.0);

    assert_eq!(m1.data[6], 0.0);
    assert_eq!(m1.data[7], 0.0);
    assert_eq!(m1.data[8], 1.0);
}

#[test]
fn test_matrix_mult_by_scalar() {
    // Arrange
    let m1 = Matrix::identity(3);

    // Act
    let m2 = m1*3.0;

    // Assert
    /*
    assert_eq!(m1.data[0], 1.0);
    assert_eq!(m1.data[1], 0.0);
    assert_eq!(m1.data[2], 0.0);

    assert_eq!(m1.data[3], 0.0);
    assert_eq!(m1.data[4], 1.0);
    assert_eq!(m1.data[5], 0.0);

    assert_eq!(m1.data[6], 0.0);
    assert_eq!(m1.data[7], 0.0);
    assert_eq!(m1.data[8], 1.0);
    */

    assert_eq!(m2.data[0], 3.0);
    assert_eq!(m2.data[1], 0.0);
    assert_eq!(m2.data[2], 0.0);

    assert_eq!(m2.data[3], 0.0);
    assert_eq!(m2.data[4], 3.0);
    assert_eq!(m2.data[5], 0.0);

    assert_eq!(m2.data[6], 0.0);
    assert_eq!(m2.data[7], 0.0);
    assert_eq!(m2.data[8], 3.0);
}
