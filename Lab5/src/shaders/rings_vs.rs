use crate::math::{Vec3, Vec4, rotation_y};
use crate::math::mat::rotation_x;
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};
use super::common::*;

#[derive(Copy, Clone, Debug)]
pub struct Rings {
    pub inner: f32, // radio interno
    pub outer: f32, // radio externo
    pub tilt: f32,  // inclinación en radianes
}

impl Default for Rings {
    fn default() -> Self {
        Self { inner: 0.75, outer: 1.6, tilt: 0.4 }
    }
}

impl Shader for Rings {
    fn name(&self) -> &'static str { "RingsShader" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
    // Transformar la esfera en “disco”:
    let mut p = vin.pos;
    p.y *= 0.03;            // aplana
    p.x *= 1.6; p.z *= 1.6; // ensancha
    let tilt_m = rotation_x(self.tilt);
    let rot = rotation_y(u.time * u.planet.rotation_speed * 0.7);
    let model = rot * tilt_m * u.model;

        let clip = u.proj * u.view * model * Vec4::from3(p, 1.0);
        let pos_ws = (model * Vec4::from3(p, 1.0)).xyz();
        let nrm_ws = Vec3::new(0.0, 1.0, 0.0); 

        VertexOut { clip_pos: clip, pos_ws, nrm_ws, uv: vin.uv }
    }

    fn fragment(&mut self, vary: &crate::renderer::raster::Varyings, u: &Uniforms) -> Color {
        let r = (vary.pos_ws.x * vary.pos_ws.x + vary.pos_ws.z * vary.pos_ws.z).sqrt();
        let bands = (r * 8.0 + fbm_3d(vary.pos_ws * 0.5 + Vec3::new(1.2,0.0,2.3), 3, 2.0, 0.5, 0.8)).sin() * 0.5 + 0.5;
        let streaks = (r * 120.0 + fbm_3d(vary.pos_ws * 5.0, 2, 2.0, 0.5, 1.4)).sin() * 0.5 + 0.5;
        let col_a = Vec3::new(0.92, 0.88, 0.78);
        let col_b = Vec3::new(0.66, 0.60, 0.48);
        let mut base = lerp3(col_a, col_b, bands);

        base = base * (0.9 + 0.45 * streaks);

        let tx = u.model.m[0][3];
        if tx < -1.5 {
            base = base + Vec3::new(0.06, 0.08, 0.02);
        } else if tx > -1.5 && tx < 0.5 {
            base = base + Vec3::new(0.04, 0.02, 0.0);
        }

        let inner = if self.inner > 0.0 { self.inner } else { 0.75 };
        let outer = if self.outer > 0.0 { self.outer } else { 1.6 };
        let edge_in = saturate((r - (inner - 0.04)) / 0.04);
        let edge_out = 1.0 - saturate((r - outer) / 0.08);
        let band_alpha = (edge_in * edge_out).clamp(0.0, 1.0);
        let dust = saturate(1.0 - (r - inner) * 0.8);
        let alpha = band_alpha * (0.25 + 0.75 * dust);
        let diff = lambert(vary.nrm_ws, u.light_dir) * 0.9 + 0.1;

        to_color((base * diff * alpha).clamp01())
    }
}
