use crate::math::{Vec3, Vec4};
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};
use super::common::*;

#[derive(Copy, Clone, Debug)]
pub struct Moon {
    pub radius: f32,    // radio de órbita
    pub scale:  f32,    
}
impl Default for Moon {
    fn default() -> Self {
        Self { radius: 2.4, scale: 0.35 }
    }
}

impl Shader for Moon {
    fn name(&self) -> &'static str { "MoonShader" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
        let angle = u.time * 0.4;
        let center = (u.model * Vec4::from3(Vec3::new(0.0, 0.0, 0.0), 1.0)).xyz();
        let offset = Vec3::new(self.radius*angle.cos(), 0.45, self.radius*angle.sin());

        let p = center + Vec3::new(vin.pos.x*self.scale, vin.pos.y*self.scale, vin.pos.z*self.scale) + offset;

        let clip = u.proj * u.view * Vec4::from3(p, 1.0);
        let pos_ws = p;
        let nrm_ws = Vec3::new(vin.nrm.x, vin.nrm.y, vin.nrm.z).normalize();

        VertexOut { clip_pos: clip, pos_ws, nrm_ws, uv: vin.uv }
    }

    fn fragment(&mut self, vary: &crate::renderer::raster::Varyings, u: &Uniforms) -> Color {
        // Luna 
        let f = fbm_3d(vary.pos_ws*0.9, 4, 2.0, 0.5, 1.2);
        let albedo = lerp3(Vec3::new(0.45,0.45,0.47), Vec3::new(0.75,0.75,0.78), f);
        let diff = lambert(vary.nrm_ws, u.light_dir)*0.85 + 0.15;
        let view_dir = (u.camera_pos - vary.pos_ws).normalize();
        let rim_k = rim(vary.nrm_ws, view_dir, 2.0)*0.25;

        to_color((albedo*diff + Vec3::new(0.9,0.9,1.0)*rim_k).clamp01())
    }
}
