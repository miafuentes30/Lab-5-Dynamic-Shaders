use crate::math::{Vec3, Vec4, rotation_y};
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};
use super::common::*;

#[derive(Copy, Clone, Debug)]
pub struct Lava {
    pub glow: Vec3,
    pub rot_speed: f32,
}
impl Default for Lava {
    fn default() -> Self {
        Self { glow: Vec3::new(1.0, 0.45, 0.05), rot_speed: 0.5 }
    }
}

impl Lava {
    fn color_layers(&self, p_ws: Vec3, n_ws: Vec3, _view_dir: Vec3, u: &Uniforms) -> Vec3 {
        // base oscura 
        let base = Vec3::new(0.08, 0.04, 0.03);

        let n = fbm_3d(p_ws * 1.8 + Vec3::new(12.0, 4.0, -6.0), 5, 2.0, 0.5, u.planet.noise_scale*1.6);
        let veins = (n*6.0).sin().abs();

        let hot = saturate((veins - 0.6) * 3.5).powf(1.8);
        let emissive = self.glow * (0.8*hot + 0.2*fbm_3d(p_ws*3.0, 3, 2.0, 0.5, u.planet.noise_scale));

        let diff = lambert(n_ws, u.light_dir)*0.9 + 0.1;

        let col = base + Vec3::new(0.6,0.25,0.08)*veins*0.9 + emissive;
        (col * diff).clamp01()
    }
}

impl Shader for Lava {
    fn name(&self) -> &'static str { "LavaPlanet" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
    let orbit_rot = rotation_y(u.time * u.planet.rotation_speed * 0.9);
    let self_rot = rotation_y(u.time * self.rot_speed);
    let model = orbit_rot * u.model * self_rot;

        let clip = u.proj * u.view * model * Vec4::from3(vin.pos, 1.0);
        let pos_ws = (model * Vec4::from3(vin.pos, 1.0)).xyz();
        let nrm_ws = (model * Vec4::from3(vin.nrm, 0.0)).xyz().normalize();

        VertexOut { clip_pos: clip, pos_ws, nrm_ws, uv: vin.uv }
    }

    fn fragment(&mut self, vary: &crate::renderer::raster::Varyings, u: &Uniforms) -> Color {
        let view_dir = (u.camera_pos - vary.pos_ws).normalize();
        let c = self.color_layers(vary.pos_ws, vary.nrm_ws, view_dir, u);
        to_color(c)
    }
}
