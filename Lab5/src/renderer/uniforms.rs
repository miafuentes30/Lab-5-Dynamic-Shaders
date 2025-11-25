use crate::math::{Vec3, Mat4};
use crate::shaders::noise::NoiseType;

#[derive(Copy, Clone, Debug)]
pub struct PlanetParams {
    pub base_color: super::buffers::Color,
    pub band_freq: f32,
    pub noise_scale: f32,
    pub rim_power: f32,
    pub rotation_speed: f32,
    pub has_rings: bool,
    pub has_moon: bool,
}

impl Default for PlanetParams {
    fn default() -> Self {
        Self {
            base_color: super::buffers::Color::rgb(180, 180, 200),
            band_freq: 6.0,
            noise_scale: 2.0,
            rim_power: 2.0,
            rotation_speed: 0.5,
            has_rings: false,
            has_moon: false,
        }
    }
}

/// Parámetros específicos de la estrella (ajustables en tiempo real)
#[derive(Copy, Clone, Debug)]
pub struct StarParams {
    pub temp_norm: f32,          // 0..1 temperatura relativa
    pub flare_intensity: f32,    // escala de llamaradas
    pub noise_scale: f32,        // escala espacial de ruido
    pub rot_speed: f32,          // velocidad de rotación
    pub noise_type: NoiseType,   // tipo de ruido a usar
    pub use_cellular_flares: bool, // si usar cellular específicamente para flares
}

impl Default for StarParams {
    fn default() -> Self {
        Self {
            temp_norm: 0.65,
            flare_intensity: 0.45,
            noise_scale: 1.5,
            rot_speed: 0.15,
            noise_type: NoiseType::Perlin,
            use_cellular_flares: false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    pub time: f32,
    pub light_dir: Vec3,
    pub view: Mat4,
    pub proj: Mat4,
    pub model: Mat4,
    pub camera_pos: Vec3,
    pub planet: PlanetParams,
    pub star: StarParams,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            time: 0.0,
            light_dir: Vec3::new(0.5, 0.7, 0.2).normalize(),
            view: Mat4::identity(),
            proj: Mat4::identity(),
            model: Mat4::identity(),
            camera_pos: Vec3::new(0.0, 0.0, 3.0),
            planet: PlanetParams::default(),
            star: StarParams::default(),
        }
    }
}