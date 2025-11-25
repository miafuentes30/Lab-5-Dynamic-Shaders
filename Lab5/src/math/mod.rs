pub mod vec;
pub mod mat;

pub use vec::{Vec2, Vec3, Vec4};
pub use mat::{Mat4, look_at_rh, perspective_rh, viewport, rotation_y};