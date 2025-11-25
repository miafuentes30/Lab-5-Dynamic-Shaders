use crate::math::Vec3;
use crate::renderer::buffers::Color;
use super::noise::{noise_3d, NoiseType};

// Helpers comunes para shaders

#[inline]
pub fn saturate(x: f32) -> f32 { x.clamp(0.0, 1.0) }

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a)*t }

#[inline]
pub fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    Vec3::new(
        lerp(a.x, b.x, t),
        lerp(a.y, b.y, t),
        lerp(a.z, b.z, t),
    )
}

#[inline]
pub fn to_color(v: Vec3) -> Color {
    Color::from_f32_rgb(v.x, v.y, v.z)
}

#[inline]
pub fn lambert(n: Vec3, l: Vec3) -> f32 { n.normalize().dot(l.normalize()).max(0.0) }

#[inline]
pub fn rim(n: Vec3, view: Vec3, power: f32) -> f32 { (1.0 - n.normalize().dot(view.normalize()).max(0.0)).powf(power) }

#[inline]
pub fn specular(n: Vec3, l: Vec3, view: Vec3, power: f32) -> f32 {
    let h = (l + view).normalize();
    n.dot(h).max(0.0).powf(power)
}

/// FBM con tipo explícito
pub fn fbm_3d_type(p: Vec3, oct: i32, lac: f32, gain: f32, scale: f32, noise_type: NoiseType) -> f32 {
    let mut freq = scale;
    let mut amp = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for _ in 0..oct {
        sum += noise_3d(p * freq, noise_type) * amp;
        norm += amp;
        freq *= lac;
        amp *= gain;
    }
    sum / norm.max(1e-6)
}

/// Wrapper Perlin por defecto (firma antigua usada por otros shaders)
pub fn fbm_3d(p: Vec3, oct: i32, lac: f32, gain: f32, scale: f32) -> f32 {
    fbm_3d_type(p, oct, lac, gain, scale, NoiseType::Perlin)
}

/// Gradiente por latitud usando la normal Y en espacio mundo (reinstaurado)
#[inline]
pub fn latitude(v: Vec3) -> f32 { (v.y * 0.5) + 0.5 }