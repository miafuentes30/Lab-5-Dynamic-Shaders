use crate::math::{Vec3, Vec4, rotation_y};
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};
use super::common::*;

#[derive(Copy, Clone, Debug)]
pub struct Gas {
    pub main_a: Vec3,
    pub main_b: Vec3,
    pub band_freq: f32,
}
impl Default for Gas {
    fn default() -> Self {
        Self {
            main_a: Vec3::new(0.85, 0.70, 0.55), // beige
            main_b: Vec3::new(0.65, 0.50, 0.40), // café
            band_freq: 6.0,
        }
    }
}

impl Gas {
    fn lat_from_normal(n_ws: Vec3) -> f32 {
        latitude(n_ws) // [0,1]
    }

    fn color_layers(&self, p_ws: Vec3, n_ws: Vec3, view_dir: Vec3, u: &Uniforms) -> Vec3 {
        // Bandas por latitud + turbulencia
        let lat = Self::lat_from_normal(n_ws); // 0 en sur, 1 en norte
        let phi = lat*std::f32::consts::TAU*self.band_freq;

        let turb = fbm_3d(p_ws + Vec3::new(3.2,7.7,1.5), 4, 2.0, 0.5, u.planet.noise_scale*1.4);
        let s = (phi + turb*3.5).sin()*0.5 + 0.5; // 0..1 ondulado

        let bands = lerp3(self.main_a, self.main_b, s);

        // Mancha  que rota con el planeta
        let _spot_phase = (u.time*0.4).sin()*0.5 + 0.5;
        let spot_dir = Vec3::new(1.0, 0.0, 0.0);
        let dot_spot = saturate(n_ws.dot(spot_dir));
        let spot = (dot_spot.powf(50.0)) * 0.6; 
        let bands_spot = bands * (1.0 - spot) + Vec3::new(0.7,0.35,0.2)*spot;

        // Suave rim
        let rim_k = rim(n_ws, view_dir, 2.2) * 0.35;

        (bands_spot + Vec3::new(0.3,0.35,0.4)*rim_k).clamp01()
    }
}

impl Shader for Gas {
    fn name(&self) -> &'static str { "GasGiant" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
        let rot = rotation_y(u.time * u.planet.rotation_speed * 0.7);
        let model = rot * u.model;

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
