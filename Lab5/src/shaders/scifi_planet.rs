use crate::math::{Vec2, Vec3, Vec4, rotation_y};
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};
use super::common::*;

#[derive(Copy, Clone, Debug)]
pub struct SciFi {
    pub layer0: Vec3,
    pub layer1: Vec3,
    pub layer2: Vec3,
    pub layer3: Vec3,
    pub glow_color: Vec3,
    pub noise_scale: f32,
}
impl Default for SciFi {
    fn default() -> Self {
        Self {
            layer0: Vec3::new(0.95, 0.12, 0.7), 
            layer1: Vec3::new(0.2, 1.0, 0.35),  
            layer2: Vec3::new(0.05, 0.95, 1.0), 
            layer3: Vec3::new(0.75, 0.18, 1.0), 
            glow_color: Vec3::new(0.45, 0.85, 1.0),
            noise_scale: 1.0,
        }
    }
}

impl SciFi {
    fn four_layer_gradient(&self, n_ws: Vec3, u: &Uniforms) -> Vec3 {
        let mut t = 0.5 + 0.5 * n_ws.y; // 0..1

        let noise = fbm_3d(n_ws * 6.0 + Vec3::new(u.time * 0.12, 0.0, u.time * 0.07), 4, 2.0, 0.5, self.noise_scale * u.planet.noise_scale);
        t = (t + noise * 0.08).clamp(0.0, 1.0);

        let centers = [0.125_f32, 0.375_f32, 0.625_f32, 0.875_f32];
        let s = 0.12_f32; // gaussian 

        let mut w0 = (-((t - centers[0])*(t - centers[0])/(2.0*s*s))).exp();
        let mut w1 = (-((t - centers[1])*(t - centers[1])/(2.0*s*s))).exp();
        let mut w2 = (-((t - centers[2])*(t - centers[2])/(2.0*s*s))).exp();
        let mut w3 = (-((t - centers[3])*(t - centers[3])/(2.0*s*s))).exp();

        let sum = w0 + w1 + w2 + w3 + 1e-6;
        w0 /= sum; w1 /= sum; w2 /= sum; w3 /= sum;

        self.layer0 * w0 + self.layer1 * w1 + self.layer2 * w2 + self.layer3 * w3
    }
}

impl Shader for SciFi {
    fn name(&self) -> &'static str { "SciFiPlanet" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
        let rot = rotation_y(u.time * u.planet.rotation_speed * 1.2);
        let model = rot * u.model;

        let clip = u.proj * u.view * model * Vec4::from3(vin.pos, 1.0);
        let pos_ws = (model * Vec4::from3(vin.pos, 1.0)).xyz();
        let nrm_ws = (model * Vec4::from3(vin.nrm, 0.0)).xyz().normalize();

        VertexOut { clip_pos: clip, pos_ws, nrm_ws, uv: vin.uv }
    }

    fn fragment(&mut self, vary: &crate::renderer::raster::Varyings, u: &Uniforms) -> Color {
        let view_dir = (u.camera_pos - vary.pos_ws).normalize();
        let col = self.four_layer_gradient(vary.nrm_ws, u);


        let diff = lambert(vary.nrm_ws, u.light_dir);
        let lit = col * (0.45 + 0.55 * diff);
        let rim_k = rim(vary.nrm_ws, view_dir, 4.0);
        let rim_color = self.glow_color * rim_k * 0.9;
        let band_t = 0.5 + 0.5 * vary.nrm_ws.y;
        let band_noise = fbm_3d(vary.pos_ws * 3.0 + Vec3::new(u.time*0.6, 0.0, 0.0), 3, 2.0, 0.5, self.noise_scale);
        let band = ((band_t * 10.0 + band_noise*2.0).fract() - 0.5).abs();
        let band_emis = (1.0 - (band * 20.0).clamp(0.0,1.0)).powf(2.0) * 0.6;
        let emis = self.glow_color * band_emis;

        to_color((lit + rim_color + emis).clamp01())
    }
}
