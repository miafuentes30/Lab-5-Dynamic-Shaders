use crate::math::{Vec2, Vec3, Vec4};

use super::buffers::{Framebuffer, Color};

#[derive(Copy, Clone, Debug, Default)]
pub struct Varyings {
    pub pos_ws: Vec3,
    pub nrm_ws: Vec3,
    pub uv: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub struct RasterInput {
    pub p: [Vec4; 3],       
    pub z: [f32; 3],        
    pub inv_w: [f32; 3],    
    pub v: [Varyings; 3],   
}

#[inline(always)]
fn barycentric(p0: (f32,f32), p1: (f32,f32), p2: (f32,f32), px: (f32,f32)) -> (f32,f32,f32,f32) {
    let (x0,y0) = p0; let (x1,y1) = p1; let (x2,y2) = p2; let (x,y) = px;
    let denom = (y1 - y2)*(x0 - x2) + (x2 - x1)*(y0 - y2);
    let w0 = ((y1 - y2)*(x - x2) + (x2 - x1)*(y - y2)) / denom;
    let w1 = ((y2 - y0)*(x - x2) + (x0 - x2)*(y - y2)) / denom;
    let w2 = 1.0 - w0 - w1;
    (w0,w1,w2,denom)
}

#[inline(always)]
fn inside_triangle(w0: f32, w1: f32, w2: f32) -> bool {
    w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
}

/// Interpolación en perspectiva de un atributo escalar
#[inline(always)]
fn persp_interp1(w: [f32;3], a: [f32;3]) -> f32 {
    let denom = w[0] + w[1] + w[2];
    (a[0]*w[0] + a[1]*w[1] + a[2]*w[2]) / denom
}

/// Interpolación en perspectiva de Vec2
#[inline(always)]
fn persp_interp2(w: [f32;3], a: [Vec2;3]) -> Vec2 {
    let denom = w[0] + w[1] + w[2];
    let ax = a[0].x*w[0] + a[1].x*w[1] + a[2].x*w[2];
    let ay = a[0].y*w[0] + a[1].y*w[1] + a[2].y*w[2];
    Vec2::new(ax/denom, ay/denom)
}

/// Interpolación en perspectiva de Vec3
#[inline(always)]
fn persp_interp3(w: [f32;3], a: [Vec3;3]) -> Vec3 {
    let denom = w[0] + w[1] + w[2];
    let ax = a[0].x*w[0] + a[1].x*w[1] + a[2].x*w[2];
    let ay = a[0].y*w[0] + a[1].y*w[1] + a[2].y*w[2];
    let az = a[0].z*w[0] + a[1].z*w[1] + a[2].z*w[2];
    Vec3::new(ax/denom, ay/denom, az/denom)
}

/// Rasteriza un triángulo usando barycentrics + zbuffer y llama a `shade_pixel(x,y, vary, z)`.
pub fn raster_triangle<F: FnMut(i32, i32, f32, Varyings) -> Color>(
    fb: &mut Framebuffer,
    tri: &RasterInput,
    mut shade_pixel: F,
) {
    if fb.width == 0 || fb.height == 0 {
        return;
    }

    // Bounding box 
    let fb_width = fb.width as i32;
    let fb_height = fb.height as i32;
    
    let min_x = tri.p.iter().map(|p| p.x).fold(f32::INFINITY, f32::min).floor().max(0.0) as i32;
    let max_x = tri.p.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max).ceil().min((fb_width - 1) as f32) as i32;
    let min_y = tri.p.iter().map(|p| p.y).fold(f32::INFINITY, f32::min).floor().max(0.0) as i32;
    let max_y = tri.p.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max).ceil().min((fb_height - 1) as f32) as i32;

    if min_x > max_x || min_y > max_y {
        return;
    }

    let p0 = (tri.p[0].x, tri.p[0].y);
    let p1 = (tri.p[1].x, tri.p[1].y);
    let p2 = (tri.p[2].x, tri.p[2].y);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (w0,w1,w2,_) = barycentric(p0,p1,p2, (x as f32 + 0.5, y as f32 + 0.5));
            if !inside_triangle(w0,w1,w2) { continue; }

            let w0p = w0 * tri.inv_w[0];
            let w1p = w1 * tri.inv_w[1];
            let w2p = w2 * tri.inv_w[2];
            let z = w0*tri.z[0] + w1*tri.z[1] + w2*tri.z[2];
            let pos_ws = persp_interp3([w0p,w1p,w2p], [tri.v[0].pos_ws, tri.v[1].pos_ws, tri.v[2].pos_ws]);
            let mut nrm_ws = persp_interp3([w0p,w1p,w2p], [tri.v[0].nrm_ws, tri.v[1].nrm_ws, tri.v[2].nrm_ws]).normalize();
            if !nrm_ws.length().is_finite() { nrm_ws = tri.v[0].nrm_ws; }
            
            let uv = persp_interp2([w0p,w1p,w2p], [tri.v[0].uv, tri.v[1].uv, tri.v[2].uv]);
            let vary = Varyings { pos_ws, nrm_ws, uv };
            let col = shade_pixel(x, y, z, vary);
            fb.put_pixel(x, y, z, col);
        }
    }
}