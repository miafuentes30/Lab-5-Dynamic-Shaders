use crate::math::{Vec2, Vec3, Vec4, rotation_y};
use crate::renderer::{
    buffers::Color,
    uniforms::Uniforms,
    pipeline::{Shader, VertexIn, VertexOut},
};
use super::common::*; 
use std::f32::consts::PI;

// Helpers locales 
#[inline]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
#[inline]
fn mix3(a: Vec3, b: Vec3, t: f32) -> Vec3 { a * (1.0 - t) + b * t }
#[inline]
fn fractf(x: f32) -> f32 { x - x.floor() }
#[inline]
fn rot_y(v: Vec3, a: f32) -> Vec3 {
    let c = a.cos(); let s = a.sin();
    Vec3::new(c*v.x + s*v.z, v.y, -s*v.x + c*v.z)
}
#[inline]
fn rot_y_uv(n: Vec3, a: f32) -> (f32,f32) {
    let r = rot_y(n, a);
    let u = r.z.atan2(r.x) / (2.0*PI) + 0.5;
    let v = r.y.asin() / PI + 0.5;
    (u,v)
}


// “fbm” sin texturas: suma de senos
fn fbm2(mut x: f32, mut y: f32, t: f32, oct: i32) -> f32 {
    let (mut a, mut v, mut fx, mut fy) = (0.5f32, 0.0f32, 3.0f32, 5.0f32);
    for _ in 0..oct {
        v += a * ((x*fx + y*fy + t*0.3).sin());
        a *= 0.55; fx *= 1.9; fy *= 1.7;
        x *= 1.2; y *= 1.2;
    }
    0.5 + 0.5 * v
}

// Variante ridge para relieves/cráteres
fn ridge2(x: f32, y: f32, t: f32) -> f32 {
    let mut v = 0.0;
    let mut a = 0.5;
    let mut fx = 4.0;
    let mut fy = 6.0;
    for _ in 0..5 {
        let s = ((x*fx + y*fy + t*0.2).sin()).abs();
        v += a * (1.0 - s);
        a *= 0.5; fx *= 1.8; fy *= 1.6;
    }
    v.clamp(0.0, 1.0)
}

// Shader: Marte 
#[derive(Copy, Clone, Debug)]
pub struct Rocky {
    pub scale: f32,
    pub rot_speed: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub orbit_world: bool,
    pub sand:   Vec3, 
    pub rust:   Vec3,  
    pub basalt: Vec3, 
    pub spec_intensity: f32,
    pub spec_power: f32,
}

impl Default for Rocky {
    fn default() -> Self {
        Self {
            scale: 1.0,
            rot_speed: 0.04,
            orbit_radius: 0.0,
            orbit_speed: 0.0,
            orbit_world: false,
            sand:   Vec3::new(0.84, 0.58, 0.38),
            rust:   Vec3::new(0.65, 0.30, 0.20),
            basalt: Vec3::new(0.30, 0.15, 0.12),
            spec_intensity: 0.35,
            spec_power: 28.0,
        }
    }
}

impl Rocky {
    #[inline]
    fn uv_from_normal(n: Vec3) -> (f32, f32) {
        let u = n.z.atan2(n.x) / (2.0*PI) + 0.5;
        let v = n.y.asin() / PI + 0.5;
        (u, v)
    }

    fn color_layers(&self, nrm_ws: Vec3, uv: Vec2, view_dir: Vec3, u: &Uniforms) -> Vec3 {
        let n = nrm_ws.normalize();
        let (mut uvs, vvs) = (uv.x, uv.y);
        // 1) BASE
        let lat = (vvs - 0.5).abs(); // 0 en ecuador
        let base_lat = mix3(self.rust, self.sand, smoothstep(0.0, 0.45, 0.5 - lat));
        let base = mix3(base_lat, self.basalt, 0.08);

        // 2) MANCHAS de albedo 
    let large = fbm2(uvs*1.0, vvs*1.0, u.time*0.15, 5);
    let small = fbm2(uvs*6.0, vvs*6.0, u.time*0.05, 4);
        let albedo_mask = smoothstep(0.45, 0.60, large) * (0.6 + 0.4*small);
        let with_albedo = mix3(base, self.basalt*0.9, albedo_mask*0.65);

        // 3) RELIEVE / CRÁTERES 
    let relief = ridge2(uvs*5.5, vvs*5.5, u.time*0.05);
    let micro  = fbm2(uvs*28.0, vvs*28.0, u.time*0.02, 3);
        let detail = (0.4*relief + 0.6*micro).clamp(0.0, 1.0);
        let rocky = with_albedo * (0.90 + 0.10*detail);
        let polar = smoothstep(0.70, 0.88, lat);
        let with_poles = mix3(rocky, Vec3::new(0.92, 0.92, 0.94), polar*0.75);

        // 4) LUZ
        let ndl = lambert(n, u.light_dir);
        let hemi = 0.18 + 0.82*ndl;
        let mut lit = with_poles * hemi;
        let l = (-u.light_dir).normalize();
        let refl = (n * (2.0 * n.dot(l)) - l).normalize();
        let spec = (refl.dot(view_dir).max(0.0)).powf(self.spec_power) * self.spec_intensity;
        lit += Vec3::new(1.0, 0.9, 0.8) * spec;

        let rim = (1.0 - n.dot(view_dir).max(0.0)).powf(3.0);
        lit += Vec3::new(1.0, 0.45, 0.25) * rim * 0.06;

        lit.clamp01()
    }
}

impl Shader for Rocky {
    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
    let orbit_rot = rotation_y(u.time * u.planet.rotation_speed * 0.7);
    let self_rot = rotation_y(u.time * self.rot_speed);
    let model = orbit_rot * u.model * self_rot;

    let clip = u.proj * u.view * model * Vec4::from3(vin.pos, 1.0);
    let pos_ws4 = model * Vec4::from3(vin.pos, 1.0);
    let nrm_ws  = (model * Vec4::from3(vin.nrm, 0.0)).xyz().normalize();

        let (su, sv) = Self::uv_from_normal(nrm_ws);
        VertexOut { clip_pos: clip, pos_ws: pos_ws4.xyz(), nrm_ws, uv: Vec2::new(su, sv) }
    }

    fn fragment(&mut self, vary: &crate::renderer::raster::Varyings, u: &Uniforms) -> Color {
        let view_dir = (u.camera_pos - vary.pos_ws).normalize();
        let c = self.color_layers(vary.nrm_ws, vary.uv, view_dir, u);
        to_color(c)
    }
}
