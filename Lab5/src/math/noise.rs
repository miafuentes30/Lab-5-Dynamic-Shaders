use crate::math::Vec3;

// NOISE TYPE SELECTOR

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Cellular,
}

impl Default for NoiseType {
    fn default() -> Self { NoiseType::Perlin }
}


// PERLIN NOISE (Original)
#[inline]
fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut n = x.wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(z.wrapping_mul(1013904223));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    let h = n ^ (n >> 16);
    (h as f32 / i32::MAX as f32) * 0.5 + 0.5
}

fn fade(t: f32) -> f32 { t*t*t*(t*(t*6.0 - 15.0) + 10.0) }

pub fn perlin_3d(p: Vec3) -> f32 {
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let iz = p.z.floor() as i32;
    let fx = p.x - ix as f32;
    let fy = p.y - iy as f32;
    let fz = p.z - iz as f32;

    let u = fade(fx);
    let v = fade(fy);
    let w = fade(fz);

    let c000 = hash(ix,   iy,   iz);
    let c100 = hash(ix+1, iy,   iz);
    let c010 = hash(ix,   iy+1, iz);
    let c110 = hash(ix+1, iy+1, iz);
    let c001 = hash(ix,   iy,   iz+1);
    let c101 = hash(ix+1, iy,   iz+1);
    let c011 = hash(ix,   iy+1, iz+1);
    let c111 = hash(ix+1, iy+1, iz+1);

    let x00 = c000*(1.0-u) + c100*u;
    let x10 = c010*(1.0-u) + c110*u;
    let x01 = c001*(1.0-u) + c101*u;
    let x11 = c011*(1.0-u) + c111*u;

    let y0 = x00*(1.0-v) + x10*v;
    let y1 = x01*(1.0-v) + x11*v;

    y0*(1.0-w) + y1*w
}



// SIMPLEX NOISE (3D)

const F3: f32 = 1.0 / 3.0;
const G3: f32 = 1.0 / 6.0;

#[inline]
fn simplex_hash(i: i32) -> i32 {
    let mut n = i.wrapping_mul(1619);
    n = (n >> 13) ^ n;
    n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(60493).wrapping_add(19990303)).wrapping_add(1376312589)
}

#[inline]
fn simplex_grad3(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

pub fn simplex_3d(p: Vec3) -> f32 {
    let (x, y, z) = (p.x, p.y, p.z);
    
    // Skew the input space
    let s = (x + y + z) * F3;
    let i = (x + s).floor();
    let j = (y + s).floor();
    let k = (z + s).floor();
    
    let t = (i + j + k) * G3;
    let x0 = x - (i - t);
    let y0 = y - (j - t);
    let z0 = z - (k - t);
    
    // Determine which simplex we're in
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 { (1, 0, 0, 1, 1, 0) }
        else if x0 >= z0 { (1, 0, 0, 1, 0, 1) }
        else { (0, 0, 1, 1, 0, 1) }
    } else {
        if y0 < z0 { (0, 0, 1, 0, 1, 1) }
        else if x0 < z0 { (0, 1, 0, 0, 1, 1) }
        else { (0, 1, 0, 1, 1, 0) }
    };
    
    let x1 = x0 - i1 as f32 + G3;
    let y1 = y0 - j1 as f32 + G3;
    let z1 = z0 - k1 as f32 + G3;
    let x2 = x0 - i2 as f32 + 2.0 * G3;
    let y2 = y0 - j2 as f32 + 2.0 * G3;
    let z2 = z0 - k2 as f32 + 2.0 * G3;
    let x3 = x0 - 1.0 + 3.0 * G3;
    let y3 = y0 - 1.0 + 3.0 * G3;
    let z3 = z0 - 1.0 + 3.0 * G3;
    
    let ii = i as i32 & 255;
    let jj = j as i32 & 255;
    let kk = k as i32 & 255;
    
    let gi0 = simplex_hash(ii + simplex_hash(jj + simplex_hash(kk)));
    let gi1 = simplex_hash(ii + i1 + simplex_hash(jj + j1 + simplex_hash(kk + k1)));
    let gi2 = simplex_hash(ii + i2 + simplex_hash(jj + j2 + simplex_hash(kk + k2)));
    let gi3 = simplex_hash(ii + 1 + simplex_hash(jj + 1 + simplex_hash(kk + 1)));
    
    let mut n = 0.0;
    
    let t0 = 0.6 - x0*x0 - y0*y0 - z0*z0;
    if t0 > 0.0 {
        let t0 = t0 * t0;
        n += t0 * t0 * simplex_grad3(gi0, x0, y0, z0);
    }
    
    let t1 = 0.6 - x1*x1 - y1*y1 - z1*z1;
    if t1 > 0.0 {
        let t1 = t1 * t1;
        n += t1 * t1 * simplex_grad3(gi1, x1, y1, z1);
    }
    
    let t2 = 0.6 - x2*x2 - y2*y2 - z2*z2;
    if t2 > 0.0 {
        let t2 = t2 * t2;
        n += t2 * t2 * simplex_grad3(gi2, x2, y2, z2);
    }
    
    let t3 = 0.6 - x3*x3 - y3*y3 - z3*z3;
    if t3 > 0.0 {
        let t3 = t3 * t3;
        n += t3 * t3 * simplex_grad3(gi3, x3, y3, z3);
    }
    
    // Scale to [0, 1]
    (32.0 * n + 1.0) * 0.5
}


// CELLULAR NOISE (Worley/Voronoi-like)

#[inline]
fn cellular_hash3(p: Vec3) -> Vec3 {
    let x = p.x.sin() * 43758.5453;
    let y = p.y.sin() * 22578.1459;
    let z = p.z.sin() * 19642.3490;
    Vec3::new(
        (x - x.floor()),
        (y - y.floor()),
        (z - z.floor())
    )
}

pub fn cellular_3d(p: Vec3) -> f32 {
    let pi = Vec3::new(p.x.floor(), p.y.floor(), p.z.floor());
    let pf = Vec3::new(p.x - pi.x, p.y - pi.y, p.z - pi.z);
    
    let mut min_dist: f32 = 1000.0;
    
    // Check 3x3x3 neighborhood
    for i in -1..=1 {
        for j in -1..=1 {
            for k in -1..=1 {
                let neighbor = Vec3::new(i as f32, j as f32, k as f32);
                let point = cellular_hash3(pi + neighbor);
                let diff = neighbor + point - pf;
                let dist = diff.length();
                
                min_dist = min_dist.min(dist);
            }
        }
    }
    
    // Normalize to [0, 1] range approximately
    (1.0 - min_dist.min(1.5) / 1.5).clamp(0.0, 1.0)
}

// UNIFIED INTERFACE

pub fn noise_3d(p: Vec3, noise_type: NoiseType) -> f32 {
    match noise_type {
        NoiseType::Perlin => perlin_3d(p),
        NoiseType::Simplex => simplex_3d(p),
        NoiseType::Cellular => cellular_3d(p),
    }
}

// FBM (Fractal Brownian Motion) 

pub fn fbm_3d(
    p: Vec3, 
    octaves: u32, 
    lacunarity: f32, 
    gain: f32, 
    scale: f32,
    noise_type: NoiseType
) -> f32 {
    let mut freq = scale;
    let mut amp = 1.0;
    let mut sum = 0.0;
    let mut max_val = 0.0;

    for _ in 0..octaves {
        sum += noise_3d(p * freq, noise_type) * amp;
        max_val += amp;
        freq *= lacunarity;
        amp *= gain;
    }

    sum / max_val
}