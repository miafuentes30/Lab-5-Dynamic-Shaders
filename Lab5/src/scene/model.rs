use std::{fs::File, io::{BufRead, BufReader}};
use crate::math::{Vec3};

/// Un vertice con posición y normal 
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub pos: Vec3,
    pub nrm: Vec3,
}

/// Triángulo indexado
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    pub i0: u32,
    pub i1: u32,
    pub i2: u32,
}

/// Malla 
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<Triangle>,
}

impl Mesh {
    pub fn is_empty(&self) -> bool { self.vertices.is_empty() || self.indices.is_empty() }

    pub fn recompute_normals(&mut self) {
        // Inicializa en cero
        for v in &mut self.vertices { v.nrm = Vec3::ZERO; }

        for tri in &self.indices {
            let a = self.vertices[tri.i0 as usize].pos;
            let b = self.vertices[tri.i1 as usize].pos;
            let c = self.vertices[tri.i2 as usize].pos;
            let n = (b - a).cross(c - a).normalize();
            self.vertices[tri.i0 as usize].nrm += n;
            self.vertices[tri.i1 as usize].nrm += n;
            self.vertices[tri.i2 as usize].nrm += n;
        }
        // Normaliza
        for v in &mut self.vertices { v.nrm = v.nrm.normalize(); }
    }
}

/// Carga un .obj **sin materiales/uvs** con formato de cara `v//vn` o `v`
/// - Soporta: `v x y z`, `vn x y z`, `f a//na b//nb c//nc` (triangulado)
/// - Si no hay `vn`, recalcula normales.
/// - Indices de .obj son 1-based (positivos). No soporta negativos.
pub fn load_obj(path: &str) -> Result<Mesh, String> {
    let file = File::open(path).map_err(|e| format!("No pude abrir {}: {}", path, e))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals:   Vec<Vec3> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices:  Vec<Triangle> = Vec::new();

    use std::collections::HashMap;
    #[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
    struct Key { v: u32, n: i32 } 
    let mut dedup: HashMap<Key, u32> = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Error leyendo {}: {}", path, e))?;
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') { continue; }

        if s.starts_with("v ") {
            // v x y z
            let mut it = s.split_whitespace();
            it.next(); // "v"
            let x: f32 = it.next().ok_or("v incompleto")?.parse().map_err(|_|"v.x inválido")?;
            let y: f32 = it.next().ok_or("v incompleto")?.parse().map_err(|_|"v.y inválido")?;
            let z: f32 = it.next().ok_or("v incompleto")?.parse().map_err(|_|"v.z inválido")?;
            positions.push(Vec3::new(x,y,z));
        } else if s.starts_with("vn ") {
            // vn x y z
            let mut it = s.split_whitespace();
            it.next(); // "vn"
            let x: f32 = it.next().ok_or("vn incompleto")?.parse().map_err(|_|"vn.x inválido")?;
            let y: f32 = it.next().ok_or("vn incompleto")?.parse().map_err(|_|"vn.y inválido")?;
            let z: f32 = it.next().ok_or("vn incompleto")?.parse().map_err(|_|"vn.z inválido")?;
            normals.push(Vec3::new(x,y,z).normalize());
        } else if s.starts_with('f') {
            // f a//na b//nb c//nc   
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() < 4 { return Err(format!("Cara inválida: {}", s)); }

            // Convierte cada “token de vértice” a (v_idx, vn_idx|-1)
            let mut face_idx: Vec<u32> = Vec::new(); 
            for p in &parts[1..] {
                let (v_i, vn_i_opt) = parse_face_token(p)?;
                let key = Key{ v: v_i, n: vn_i_opt.unwrap_or(-1) };
                let idx = if let Some(&found) = dedup.get(&key) {
                    found
                } else {
                    // Creamos nuevo Vertex
                    let pos = positions.get((v_i-1) as usize)
                        .ok_or_else(|| format!("Índice v fuera de rango en {}", s))?;
                    let nrm = if let Some(vn_i) = vn_i_opt {
                        // vn_i es 1-based
                        *normals.get((vn_i-1) as usize)
                            .ok_or_else(|| format!("Índice vn fuera de rango en {}", s))?
                    } else {
                        Vec3::ZERO 
                    };
                    let new_index = vertices.len() as u32;
                    vertices.push(Vertex{ pos: *pos, nrm });
                    dedup.insert(key, new_index);
                    new_index
                };
                face_idx.push(idx);
            }

            for i in 2..face_idx.len() {
                indices.push(Triangle{ i0: face_idx[0], i1: face_idx[i-1], i2: face_idx[i] });
            }
        } else {
            // Ignorar otras líneas
            continue;
        }
    }

    let mut mesh = Mesh { vertices, indices };

    let had_normals = mesh.vertices.iter().any(|v| v.nrm.length() > 0.0);
    if !had_normals {
        mesh.recompute_normals();
    }
    Ok(mesh)
}

/// Parsea un token de cara:
/// - "a//na" -> (a, Some(na))
/// - "a"     -> (a, None)
/// - "a/b/c" o "a/b" los rechazamos para mantener sencillo (no hay vt)
// Acepta: "a/b/c" (v/vt/vn), "a//c" (v//vn), "a/b" (v/vt), "a" (v)
// Ignora vt; usa vn si está.
fn parse_face_token(tok: &str) -> Result<(u32, Option<i32>), String> {
    let parts: Vec<&str> = tok.split('/').collect();
    match parts.len() {
        1 => {
            // "a"
            let v: u32 = parts[0].parse().map_err(|_| format!("v inválido en '{}'", tok))?;
            Ok((v, None))
        }
        2 => {
            // "a/b"  (sin vn)
            let v: u32 = parts[0].parse().map_err(|_| format!("v inválido en '{}'", tok))?;
            Ok((v, None))
        }
        3 => {
            // "a/b/c" o "a//c"
            let v: u32 = parts[0].parse().map_err(|_| format!("v inválido en '{}'", tok))?;
            let vn_opt = if parts[2].is_empty() {
                None
            } else {
                Some(parts[2].parse::<i32>().map_err(|_| format!("vn inválido en '{}'", tok))?)
            };
            Ok((v, vn_opt))
        }
        _ => Err(format!("Token de cara no soportado: '{}'", tok)),
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_faces_basic() {
        assert_eq!(parse_face_token("3//7").unwrap(), (3, Some(7)));
        assert_eq!(parse_face_token("12").unwrap(), (12, None));
        assert!(parse_face_token("1/2/3").is_err());
    }
}
