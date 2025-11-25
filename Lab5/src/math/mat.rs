use core::ops::Mul;
use super::vec::{Vec3, Vec4};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat4 { pub m: [[f32;4];4] }

impl Default for Mat4 {
    fn default() -> Self { Self::identity() }
}

impl Mat4 {
    pub fn identity() -> Self {
        Self { m: [
            [1.0,0.0,0.0,0.0],
            [0.0,1.0,0.0,0.0],
            [0.0,0.0,1.0,0.0],
            [0.0,0.0,0.0,1.0],
        ]}
    }

    pub fn transpose(self) -> Self {
        let m = self.m;
        Self { m: [
            [m[0][0], m[1][0], m[2][0], m[3][0]],
            [m[0][1], m[1][1], m[2][1], m[3][1]],
            [m[0][2], m[1][2], m[2][2], m[3][2]],
            [m[0][3], m[1][3], m[2][3], m[3][3]],
        ]}
    }

    pub fn as_array(&self) -> &[[f32;4];4] { &self.m }
}

// Mat4 * Mat4
impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, o: Mat4) -> Mat4 {
        let mut r = [[0.0;4];4];
        for i in 0..4 {
            for j in 0..4 {
                r[i][j] = self.m[i][0]*o.m[0][j]
                        + self.m[i][1]*o.m[1][j]
                        + self.m[i][2]*o.m[2][j]
                        + self.m[i][3]*o.m[3][j];
            }
        }
        Mat4 { m: r }
    }
}

// Mat4 * Vec4
impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, v: Vec4) -> Vec4 {
        let m = self.m;
        Vec4 {
            x: m[0][0]*v.x + m[0][1]*v.y + m[0][2]*v.z + m[0][3]*v.w,
            y: m[1][0]*v.x + m[1][1]*v.y + m[1][2]*v.z + m[1][3]*v.w,
            z: m[2][0]*v.x + m[2][1]*v.y + m[2][2]*v.z + m[2][3]*v.w,
            w: m[3][0]*v.x + m[3][1]*v.y + m[3][2]*v.z + m[3][3]*v.w,
        }
    }
}

// transforms (Right-Handed)
pub fn translate(t: Vec3) -> Mat4 {
    let mut m = Mat4::identity();
    m.m[0][3] = t.x; m.m[1][3] = t.y; m.m[2][3] = t.z;
    m
}
pub fn scale(s: Vec3) -> Mat4 {
    Mat4 { m: [
        [s.x, 0.0, 0.0, 0.0],
        [0.0, s.y, 0.0, 0.0],
        [0.0, 0.0, s.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]}
}
pub fn rotation_x(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    Mat4 { m: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0,    c,   -s, 0.0],
        [0.0,    s,    c, 0.0],
        [0.0,  0.0,  0.0, 1.0],
    ]}
}
pub fn rotation_y(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    Mat4 { m: [
        [   c, 0.0,    s, 0.0],
        [ 0.0, 1.0,  0.0, 0.0],
        [  -s, 0.0,    c, 0.0],
        [ 0.0, 0.0,  0.0, 1.0],
    ]}
}
pub fn rotation_z(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    Mat4 { m: [
        [   c,   -s, 0.0, 0.0],
        [   s,    c, 0.0, 0.0],
        [ 0.0,  0.0, 1.0, 0.0],
        [ 0.0,  0.0, 0.0, 1.0],
    ]}
}

pub fn perspective_rh(fovy_radians: f32, aspect: f32, znear: f32, zfar: f32) -> Mat4 {
    let f = 1.0 / (0.5*fovy_radians).tan();
    let nf = 1.0 / (znear - zfar);
    Mat4 { m: [
        [f/aspect, 0.0, 0.0,                 0.0],
        [0.0,         f, 0.0,                 0.0],
        [0.0,       0.0, (zfar+znear)*nf, 2.0*znear*zfar*nf],
        [0.0,       0.0, -1.0,                0.0],
    ]}
}

pub fn ortho_rh(l: f32, r: f32, b: f32, t: f32, n: f32, fz: f32) -> Mat4 {
    Mat4 { m: [
        [2.0/(r-l), 0.0, 0.0, -(r+l)/(r-l)],
        [0.0, 2.0/(t-b), 0.0, -(t+b)/(t-b)],
        [0.0, 0.0, -2.0/(fz-n), -(fz+n)/(fz-n)],
        [0.0, 0.0, 0.0, 1.0],
    ]}
}

pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let f = (center - eye).normalize();          
    let s = f.cross(up).normalize();             
    let u = s.cross(f);                          
    Mat4 { m: [
        [ s.x,  s.y,  s.z, - (s.x*eye.x + s.y*eye.y + s.z*eye.z)],
        [ u.x,  u.y,  u.z, - (u.x*eye.x + u.y*eye.y + u.z*eye.z)],
        [-f.x, -f.y, -f.z,   (f.x*eye.x + f.y*eye.y + f.z*eye.z)],
        [ 0.0,  0.0,  0.0, 1.0],
    ]}
}

/// Viewport matrix 
pub fn viewport(x: f32, y: f32, w: f32, h: f32, depth: f32) -> Mat4 {
    Mat4 { m: [
        [ w/2.0,   0.0,     0.0, x + w/2.0],
        [  0.0, -h/2.0,     0.0, y + h/2.0], 
        [  0.0,   0.0, depth/2.0,   depth/2.0],
        [  0.0,   0.0,     0.0,        1.0],
    ]}
}
