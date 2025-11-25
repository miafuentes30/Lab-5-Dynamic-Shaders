use image::{RgbaImage, Rgba};

#[derive(Copy, Clone, Debug, Default)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b, a: 255 } }

    pub fn from_f32_rgb(r: f32, g: f32, b: f32) -> Self {
        fn to8(x: f32) -> u8 { (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 }
        Self::rgb(to8(r), to8(g), to8(b))
    }

    pub fn to_rgba(self) -> [u8;4] { [self.r, self.g, self.b, self.a] }
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub color: Vec<Color>,
    pub depth: Vec<f32>, 
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width, height,
            color: vec![Color::rgb(0,0,0); width*height],
            depth: vec![f32::INFINITY; width*height],
        }
    }

    #[inline] pub fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 { return None; }
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height { return None; }
        Some(y*self.width + x)
    }

    pub fn clear_color(&mut self, c: Color) {
        self.color.fill(c);
    }

    pub fn clear_depth(&mut self) {
        self.depth.fill(f32::INFINITY);
    }

    #[inline]
    pub fn put_pixel(&mut self, x: i32, y: i32, z: f32, c: Color) {
        if let Some(i) = self.idx(x, y) {
            if z < self.depth[i] {
                self.depth[i] = z;
                self.color[i] = c;
            }
        }
    }

    pub fn save_png(&self, path: &str) -> Result<(), String> {
        let mut img = RgbaImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y*self.width + x;
                let px = self.color[i].to_rgba();
                img.put_pixel(x as u32, y as u32, Rgba(px));
            }
        }
        img.save(path).map_err(|e| format!("No pude guardar '{}': {}", path, e))
    }
}
