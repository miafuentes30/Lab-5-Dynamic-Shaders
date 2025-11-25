use crate::math::{Vec2, Vec3, Vec4};
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};

#[derive(Copy, Clone, Debug, Default)]
pub struct Flat;

impl Shader for Flat {
    fn name(&self) -> &'static str { "FlatDebug" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
        let clip = u.proj * u.view * u.model * Vec4::from3(vin.pos, 1.0);
        let pos_ws = (u.model * Vec4::from3(vin.pos, 1.0)).xyz();
        let nrm_ws = (u.model * Vec4::from3(vin.nrm, 0.0)).xyz();
        VertexOut { clip_pos: clip, pos_ws, nrm_ws, uv: vin.uv }
    }

    fn fragment(&mut self, _vary: &crate::renderer::raster::Varyings, _u: &Uniforms) -> Color {
        Color::rgb(230, 150, 80)
    }
}
