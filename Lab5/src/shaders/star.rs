use crate::math::{Vec3, Vec4, rotation_y};
use crate::renderer::{buffers::Color, uniforms::Uniforms, pipeline::{Shader, VertexIn, VertexOut}};
use crate::shaders::noise::cellular_3d;
use super::common::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Star;

impl Star {
    fn gradient(&self, t: f32) -> Vec3 {
        // Gradiente más suave con transiciones mejoradas
        let c0 = Vec3::new(0.85, 0.25, 0.05); // Base naranja más suave
        let c1 = Vec3::new(1.00, 0.50, 0.10); // Naranja cálido
        let c2 = Vec3::new(1.00, 0.80, 0.30); // Amarillo dorado
        let c3 = Vec3::new(1.00, 0.95, 0.85); // Blanco cálido suave
        
        if t < 0.33 { 
            lerp3(c0, c1, smoothstep(0.0, 0.33, t)) 
        } else if t < 0.66 { 
            lerp3(c1, c2, smoothstep(0.33, 0.66, t)) 
        } else { 
            lerp3(c2, c3, smoothstep(0.66, 1.0, t)) 
        }
    }

    fn emission_color(&self, intensity: f32, temp_norm: f32) -> Vec3 {
        let base = self.gradient(intensity);
        let hot_bias = lerp3(base, Vec3::new(1.0, 0.98, 0.92), temp_norm * 0.3);
        hot_bias
    }

    fn surface_intensity(&self, p_ws: Vec3, t: f32, params: &crate::renderer::uniforms::StarParams) -> f32 {
        let noise_type = params.noise_type;
        let scale = params.noise_scale;
        
        // Ruido más suave y orgánico usando el tipo seleccionado
        let n1 = fbm_3d_type(
            p_ws + Vec3::new(t*0.08, t*0.05, -t*0.03), 
            4, 2.0, 0.55, scale, noise_type
        );
        let n2 = fbm_3d_type(
            p_ws*1.8 + Vec3::new(-t*0.06, t*0.09, t*0.03), 
            3, 2.0, 0.55, scale*0.8, noise_type
        );
        let n3 = fbm_3d_type(
            p_ws*3.2 + Vec3::new(t*0.2, -t*0.15, t*0.12), 
            2, 2.0, 0.5, scale*0.6, noise_type
        );
        
        let combo = n1*0.5 + n2*0.3 + n3*0.2;
        let pulsate = (t*0.4).sin()*0.1 + 0.9; // Pulsación más sutil
        saturate(combo * pulsate)
    }

    fn flare_term(&self, p_ws: Vec3, t: f32, params: &crate::renderer::uniforms::StarParams) -> f32 {
        let scale = params.noise_scale;
        let intensity = params.flare_intensity;
        
        // Flares más suaves y naturales
        let base = if params.use_cellular_flares {
            // Cellular da un efecto más "celular" / orgánico para flares
            cellular_3d(p_ws*4.5 + Vec3::new(t*0.6, -t*0.4, t*0.3))
        } else {
            fbm_3d_type(
                p_ws*4.5 + Vec3::new(t*0.6, -t*0.4, t*0.3), 
                3, 2.0, 0.6, scale*0.8, params.noise_type
            )
        };
        
        let ridge = (base*1.5 - 0.75).abs();
        saturate(ridge.powf(2.5) * intensity * 0.6)
    }
}

impl Shader for Star {
    fn name(&self) -> &'static str { "Star" }

    fn vertex(&mut self, vin: VertexIn, u: &Uniforms) -> VertexOut {
        let t = u.time;
        let params = &u.star;
        let self_rot = rotation_y(t * params.rot_speed);
        let model = u.model * self_rot;

        let base_pos = vin.pos;
        let p_local = base_pos.normalize();
        
        // Distorsión usando el tipo de ruido seleccionado
        let flare = fbm_3d_type(
            p_local + Vec3::new(t*0.15, -t*0.12, t*0.08), 
            3, 2.0, 0.6, params.noise_scale*1.5, 
            params.noise_type
        );
        
        let flare_ridge = (flare*1.8 - 0.9).abs().powf(3.5) * 0.08 * params.flare_intensity;
        let wave = (t*0.6 + base_pos.length()*4.0).sin()*0.015;
        let radial_scale = 1.0 + flare_ridge + wave;
        let displaced = base_pos * radial_scale;

        let clip = u.proj * u.view * model * Vec4::from3(displaced, 1.0);
        let pos_ws = (model * Vec4::from3(displaced, 1.0)).xyz();
        let nrm_ws = (model * Vec4::from3(vin.nrm, 0.0)).xyz().normalize();

        VertexOut { clip_pos: clip, pos_ws, nrm_ws, uv: vin.uv }
    }

    fn fragment(&mut self, vary: &crate::renderer::raster::Varyings, u: &Uniforms) -> Color {
        let t = u.time;
        let params = &u.star;
        let view_dir = (u.camera_pos - vary.pos_ws).normalize();

        let intensity = self.surface_intensity(vary.pos_ws * 0.9, t, params);
        let flare = self.flare_term(vary.pos_ws, t, params);
        let base_col = self.emission_color(intensity, params.temp_norm);

        // Emisión más balanceada
        let emission = (intensity.powf(1.8) * 0.7 + flare * 0.9).min(2.5);

        // Difuso más presente
        let diff = 0.3 + 0.7*lambert(vary.nrm_ws, u.light_dir);

        // Glow de borde más suave
        let glow = rim(vary.nrm_ws, view_dir, 2.5) * 0.5;
        let glow_col = Vec3::new(1.0, 0.65, 0.25) * glow;

        // Mezcla más equilibrada
        let mut col = base_col * diff * 0.5 + base_col * emission * 0.5 + glow_col * 0.8;

        col = col.clamp01();
        to_color(col)
    }
}

// Helper para transiciones suaves
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}