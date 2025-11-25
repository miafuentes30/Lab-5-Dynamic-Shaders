use crate::math::{Vec2, Vec3, Vec4, Mat4};
use crate::scene::Mesh;
use super::buffers::{Framebuffer, Color};
use super::raster::{RasterInput, Varyings, raster_triangle};
use super::uniforms::Uniforms;

/// Entrada al vertex shader
#[derive(Copy, Clone, Debug)]
pub struct VertexIn {
    pub pos: Vec3,
    pub nrm: Vec3,
    pub uv:  Vec2, 
}

/// Salida del vertex shader
#[derive(Copy, Clone, Debug)]
pub struct VertexOut {
    pub clip_pos: Vec4, 
    pub pos_ws: Vec3,  
    pub nrm_ws: Vec3,
    pub uv: Vec2,
}


pub trait Shader {
    fn name(&self) -> &'static str { "UnnamedShader" }

    /// Vertex: recibe atributos por-vértice + uniforms, devuelve clip_pos y varyings
    fn vertex(&mut self, vin: VertexIn, uniforms: &Uniforms) -> VertexOut;

    /// Fragment: recibe varyings interpolados + uniforms y devuelve Color
    fn fragment(&mut self, vary: &Varyings, uniforms: &Uniforms) -> Color;
}

pub fn draw_mesh(
    fb: &mut Framebuffer,
    mesh: &Mesh,
    shader: &mut dyn Shader,
    uniforms: &Uniforms,
    viewport: Mat4,
) {
    let _mvp = uniforms.proj * uniforms.view * uniforms.model;

    // Vertex stage
    let mut clip_positions: Vec<Vec4> = Vec::with_capacity(mesh.vertices.len());
    let mut vary_buff: Vec<(Vec3, Vec3, Vec2)> = Vec::with_capacity(mesh.vertices.len());

    for v in &mesh.vertices {
        let vin = VertexIn {
            pos: v.pos,
            nrm: v.nrm,
            uv:  Vec2::new(0.0, 0.0),
        };
        let vout = shader.vertex(vin, uniforms);
        clip_positions.push(vout.clip_pos);
        vary_buff.push((vout.pos_ws, vout.nrm_ws, vout.uv));
    }

    for tri in &mesh.indices {
        let idx = [tri.i0 as usize, tri.i1 as usize, tri.i2 as usize];

        // Clip coordinates
        let cp = [clip_positions[idx[0]], clip_positions[idx[1]], clip_positions[idx[2]]];

        if cp.iter().any(|p| p.w <= 0.0) { continue; }

        // NDC
        let ndc = [
            Vec4::new(cp[0].x/cp[0].w, cp[0].y/cp[0].w, cp[0].z/cp[0].w, 1.0),
            Vec4::new(cp[1].x/cp[1].w, cp[1].y/cp[1].w, cp[1].z/cp[1].w, 1.0),
            Vec4::new(cp[2].x/cp[2].w, cp[2].y/cp[2].w, cp[2].z/cp[2].w, 1.0),
        ];

        // Viewport 
        let sp = [
            viewport * ndc[0],
            viewport * ndc[1],
            viewport * ndc[2],
        ];

        let z = [sp[0].z, sp[1].z, sp[2].z];
        let inv_w = [1.0/cp[0].w, 1.0/cp[1].w, 1.0/cp[2].w];

        // Varyings
        let v0 = vary_buff[idx[0]];
        let v1 = vary_buff[idx[1]];
        let v2 = vary_buff[idx[2]];
        let v = [
            Varyings { pos_ws: v0.0, nrm_ws: v0.1.normalize(), uv: v0.2 },
            Varyings { pos_ws: v1.0, nrm_ws: v1.1.normalize(), uv: v1.2 },
            Varyings { pos_ws: v2.0, nrm_ws: v2.1.normalize(), uv: v2.2 },
        ];

        let rin = RasterInput { p: sp, z, inv_w, v };
        raster_triangle(fb, &rin, |_x, _y, _z, vary| {
            shader.fragment(&vary, uniforms)
        });
    }
}