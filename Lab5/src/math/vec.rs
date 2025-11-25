use core::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign, Neg};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 { pub x: f32, pub y: f32 }
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec4 { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE:  Self = Self { x: 1.0, y: 1.0 };

    pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn dot(self, o: Self) -> f32 { self.x*o.x + self.y*o.y }
    pub fn length(self) -> f32 { self.dot(self).sqrt() }
    pub fn normalize(self) -> Self { let l = self.length(); if l>0.0 { self / l } else { self } }
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE:  Self = Self { x: 1.0, y: 1.0, z: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn from_scalar(v: f32) -> Self { Self { x: v, y: v, z: v } }
    pub fn dot(self, o: Self) -> f32 { self.x*o.x + self.y*o.y + self.z*o.z }
    pub fn cross(self, o: Self) -> Self {
        Self {
            x: self.y*o.z - self.z*o.y,
            y: self.z*o.x - self.x*o.z,
            z: self.x*o.y - self.y*o.x,
        }
    }
    pub fn length(self) -> f32 { self.dot(self).sqrt() }
    pub fn normalize(self) -> Self { let l = self.length(); if l>0.0 { self / l } else { self } }
    pub fn clamp01(self) -> Self { Self{ x: self.x.clamp(0.0,1.0), y: self.y.clamp(0.0,1.0), z: self.z.clamp(0.0,1.0)} }
    pub fn hadamard(self, o: Self) -> Self { Self { x: self.x*o.x, y: self.y*o.y, z: self.z*o.z } }
    pub fn xy(self) -> Vec2 { Vec2::new(self.x, self.y) }
    pub fn xz(self) -> Vec2 { Vec2::new(self.x, self.z) }
}

impl Vec4 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE:  Self = Self { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
    pub fn from3(v: Vec3, w: f32) -> Self { Self { x: v.x, y: v.y, z: v.z, w } }
    pub fn xyz(self) -> Vec3 { Vec3::new(self.x, self.y, self.z) }
}

// Operators Vec2
impl Add for Vec2 { type Output=Self; fn add(self, o:Self)->Self{ Self{ x:self.x+o.x, y:self.y+o.y }}}
impl Sub for Vec2 { type Output=Self; fn sub(self, o:Self)->Self{ Self{ x:self.x-o.x, y:self.y-o.y }}}
impl Mul<f32> for Vec2 { type Output=Self; fn mul(self, s:f32)->Self{ Self{ x:self.x*s, y:self.y*s }}}
impl Div<f32> for Vec2 { type Output=Self; fn div(self, s:f32)->Self{ Self{ x:self.x/s, y:self.y/s }}}
impl AddAssign for Vec2 { fn add_assign(&mut self, o:Self){ self.x+=o.x; self.y+=o.y; } }
impl SubAssign for Vec2 { fn sub_assign(&mut self, o:Self){ self.x-=o.x; self.y-=o.y; } }
impl MulAssign<f32> for Vec2 { fn mul_assign(&mut self, s:f32){ self.x*=s; self.y*=s; } }
impl DivAssign<f32> for Vec2 { fn div_assign(&mut self, s:f32){ self.x/=s; self.y/=s; } }
impl Neg for Vec2 { type Output=Self; fn neg(self)->Self{ Self{ x:-self.x, y:-self.y } }}

//  Operators Vec3
impl Add for Vec3 { type Output=Self; fn add(self, o:Self)->Self{ Self{ x:self.x+o.x, y:self.y+o.y, z:self.z+o.z }}}
impl Sub for Vec3 { type Output=Self; fn sub(self, o:Self)->Self{ Self{ x:self.x-o.x, y:self.y-o.y, z:self.z-o.z }}}
impl Mul<f32> for Vec3 { type Output=Self; fn mul(self, s:f32)->Self{ Self{ x:self.x*s, y:self.y*s, z:self.z*s }}}
impl Div<f32> for Vec3 { type Output=Self; fn div(self, s:f32)->Self{ Self{ x:self.x/s, y:self.y/s, z:self.z/s }}}
impl AddAssign for Vec3 { fn add_assign(&mut self, o:Self){ self.x+=o.x; self.y+=o.y; self.z+=o.z; } }
impl SubAssign for Vec3 { fn sub_assign(&mut self, o:Self){ self.x-=o.x; self.y-=o.y; self.z-=o.z; } }
impl MulAssign<f32> for Vec3 { fn mul_assign(&mut self, s:f32){ self.x*=s; self.y*=s; self.z*=s; } }
impl DivAssign<f32> for Vec3 { fn div_assign(&mut self, s:f32){ self.x/=s; self.y/=s; self.z/=s; } }
impl Neg for Vec3 { type Output=Self; fn neg(self)->Self{ Self{ x:-self.x, y:-self.y, z:-self.z } }}

// Operators Vec4
impl Add for Vec4 { type Output=Self; fn add(self, o:Self)->Self{ Self{ x:self.x+o.x, y:self.y+o.y, z:self.z+o.z, w:self.w+o.w }}}
impl Sub for Vec4 { type Output=Self; fn sub(self, o:Self)->Self{ Self{ x:self.x-o.x, y:self.y-o.y, z:self.z-o.z, w:self.w-o.w }}}
impl Mul<f32> for Vec4 { type Output=Self; fn mul(self, s:f32)->Self{ Self{ x:self.x*s, y:self.y*s, z:self.z*s, w:self.w*s }}}
impl Div<f32> for Vec4 { type Output=Self; fn div(self, s:f32)->Self{ Self{ x:self.x/s, y:self.y/s, z:self.z/s, w:self.w/s }}}

impl AddAssign for Vec4 { fn add_assign(&mut self, o:Self){ self.x+=o.x; self.y+=o.y; self.z+=o.z; self.w+=o.w; } }
impl SubAssign for Vec4 { fn sub_assign(&mut self, o:Self){ self.x-=o.x; self.y-=o.y; self.z-=o.z; self.w-=o.w; } }
impl MulAssign<f32> for Vec4 { fn mul_assign(&mut self, s:f32){ self.x*=s; self.y*=s; self.z*=s; self.w*=s; } }
impl DivAssign<f32> for Vec4 { fn div_assign(&mut self, s:f32){ self.x/=s; self.y/=s; self.z/=s; self.w/=s; } }

impl Neg for Vec4 { type Output=Self; fn neg(self)->Self{ Self{ x:-self.x, y:-self.y, z:-self.z, w:-self.w } }}
