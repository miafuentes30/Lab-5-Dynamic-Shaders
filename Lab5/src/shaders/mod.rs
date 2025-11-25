pub mod common;
pub mod noise; // ruido extra (Perlin/Simplex/Cellular)
pub mod flat;
pub mod star;
pub mod rocky_planet;
pub mod gas_giant;
pub mod scifi_planet;
pub mod lava;
pub mod ice;
pub mod rings_vs; // nombres originales
pub mod moon_vs;

use crate::renderer::pipeline::Shader;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderKind {
    Flat,
    Rocky,
    Gas,
    SciFi,
    Lava,
    Ice,
    Star,
    Rings,
    Moon,
}

pub fn make_shader(kind: ShaderKind) -> Box<dyn Shader> {
    match kind {
        ShaderKind::Flat  => Box::new(flat::Flat::default()),
        ShaderKind::Rocky => Box::new(rocky_planet::Rocky::default()),
        ShaderKind::Gas   => Box::new(gas_giant::Gas::default()),
        ShaderKind::SciFi => Box::new(scifi_planet::SciFi::default()),
        ShaderKind::Lava  => Box::new(lava::Lava::default()),
        ShaderKind::Ice   => Box::new(ice::Ice::default()),
        ShaderKind::Star  => Box::new(star::Star::default()),
        ShaderKind::Rings => Box::new(rings_vs::Rings::default()),
        ShaderKind::Moon  => Box::new(moon_vs::Moon::default()),
    }
}