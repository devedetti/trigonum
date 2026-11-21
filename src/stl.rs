// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fs::File, path::Path};

use glam::Vec3;

use crate::{
    i18n::{fill, gettext},
    mesh_info::MeshInfo,
};

/// Malla lista para subir a la GPU: vértices intercalados [px, py, pz, nx, ny, nz],
/// tres por triángulo (sin índices, normales planas por cara).
pub struct Mesh {
    pub data: Vec<f32>,
    pub info: MeshInfo,
}

pub fn load(path: &Path) -> Result<Mesh, String> {
    let file = File::open(path).map_err(|e| fill(gettext("Cannot open the file: {error}"), &[("error", &e)]))?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    if len == 0 {
        return Err(gettext("The file is empty"));
    }
    // SAFETY: mapeo de solo lectura; si otro proceso trunca el archivo mientras leemos
    // podría fallar, pero la recarga espera a que terminen las escrituras.
    let map = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;
    parse(&map)
}

pub fn parse(bytes: &[u8]) -> Result<Mesh, String> {
    let mut builder = Builder::default();
    if bytes.len() >= 84 {
        let n = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
        let expected = 84usize.saturating_add(n.saturating_mul(50));
        let looks_ascii = bytes.starts_with(b"solid") && expected != bytes.len();
        // Binario si el tamaño cuadra exacto, o si no es ASCII y alcanza para n triángulos
        // (algunos exportadores agregan basura al final).
        if !looks_ascii && expected <= bytes.len() {
            parse_binary(&bytes[84..expected], n, &mut builder);
            return builder.finish();
        }
    }
    if bytes.starts_with(b"solid") {
        parse_ascii(bytes, &mut builder)?;
        return builder.finish();
    }
    Err(gettext("This does not look like a valid STL file"))
}

fn parse_binary(body: &[u8], n: usize, b: &mut Builder) {
    b.reserve(n);
    let f = |s: &[u8], i: usize| f32::from_le_bytes(s[i * 4..i * 4 + 4].try_into().unwrap());
    for rec in body.as_chunks::<50>().0.iter().take(n) {
        let v = |k: usize| Vec3::new(f(rec, 3 + k * 3), f(rec, 4 + k * 3), f(rec, 5 + k * 3));
        let file_normal = Vec3::new(f(rec, 0), f(rec, 1), f(rec, 2));
        b.push(v(0), v(1), v(2), file_normal);
    }
}

fn parse_ascii(bytes: &[u8], b: &mut Builder) -> Result<(), String> {
    let mut tokens = bytes
        .split(|c| c.is_ascii_whitespace())
        .filter(|t| !t.is_empty());
    let mut verts: [Vec3; 3] = [Vec3::ZERO; 3];
    let mut count = 0;
    let num = |t: Option<&[u8]>| -> Result<f32, String> {
        t.and_then(|t| std::str::from_utf8(t).ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| gettext("Invalid number in ASCII STL file"))
    };
    while let Some(tok) = tokens.next() {
        if tok.eq_ignore_ascii_case(b"vertex") {
            let v = Vec3::new(num(tokens.next())?, num(tokens.next())?, num(tokens.next())?);
            verts[count] = v;
            count += 1;
            if count == 3 {
                b.push(verts[0], verts[1], verts[2], Vec3::ZERO);
                count = 0;
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct Builder {
    data: Vec<f32>,
    info: MeshInfo,
}

impl Builder {
    fn reserve(&mut self, tris: usize) {
        self.data.reserve(tris.min(50_000_000) * 18);
    }

    fn push(&mut self, a: Vec3, b: Vec3, c: Vec3, file_normal: Vec3) {
        if !(a.is_finite() && b.is_finite() && c.is_finite()) {
            return;
        }
        // Normal calculada desde la geometría: muchos exportadores dejan la del archivo en cero.
        let mut n = (b - a).cross(c - a).normalize_or_zero();
        if n == Vec3::ZERO {
            n = file_normal.normalize_or_zero();
        }
        for v in [a, b, c] {
            self.data.extend_from_slice(&[v.x, v.y, v.z, n.x, n.y, n.z]);
        }
        self.info.add_triangle(a, b, c);
    }

    fn finish(self) -> Result<Mesh, String> {
        if self.info.triangles == 0 {
            return Err(gettext("The file contains no triangles"));
        }
        Ok(Mesh { data: self.data, info: self.info })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cubo de 10 mm con caras orientadas hacia afuera.
    fn cube() -> Vec<[Vec3; 3]> {
        let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * 10.0;
        let quads = [
            [p(0., 0., 0.), p(0., 1., 0.), p(1., 1., 0.), p(1., 0., 0.)],
            [p(0., 0., 1.), p(1., 0., 1.), p(1., 1., 1.), p(0., 1., 1.)],
            [p(0., 0., 0.), p(1., 0., 0.), p(1., 0., 1.), p(0., 0., 1.)],
            [p(0., 1., 0.), p(0., 1., 1.), p(1., 1., 1.), p(1., 1., 0.)],
            [p(0., 0., 0.), p(0., 0., 1.), p(0., 1., 1.), p(0., 1., 0.)],
            [p(1., 0., 0.), p(1., 1., 0.), p(1., 1., 1.), p(1., 0., 1.)],
        ];
        quads
            .iter()
            .flat_map(|[a, b, c, d]| [[*a, *b, *c], [*a, *c, *d]])
            .collect()
    }

    fn binary(tris: &[[Vec3; 3]], header: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; 80];
        out[..header.len()].copy_from_slice(header);
        out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
        for t in tris {
            out.extend_from_slice(&[0u8; 12]);
            for v in t {
                for c in v.to_array() {
                    out.extend_from_slice(&c.to_le_bytes());
                }
            }
            out.extend_from_slice(&[0u8; 2]);
        }
        out
    }

    fn ascii(tris: &[[Vec3; 3]]) -> Vec<u8> {
        let mut s = String::from("solid cube\n");
        for t in tris {
            s += "  facet normal 0 0 0\n    outer loop\n";
            for v in t {
                s += &format!("      vertex {} {} {}\n", v.x, v.y, v.z);
            }
            s += "    endloop\n  endfacet\n";
        }
        s += "endsolid cube\n";
        s.into_bytes()
    }

    fn check(mesh: &Mesh) {
        let i = &mesh.info;
        assert_eq!(i.triangles, 12);
        assert_eq!(mesh.data.len(), 12 * 18);
        assert_eq!(i.min, Vec3::ZERO);
        assert_eq!(i.size(), Vec3::splat(10.0));
        assert!((i.volume() - 1000.0).abs() < 1e-6, "volumen {}", i.volume());
        assert!((i.area - 600.0).abs() < 1e-6, "área {}", i.area);
    }

    #[test]
    fn binary_cube() {
        check(&parse(&binary(&cube(), b"binary")).unwrap());
    }

    #[test]
    fn binary_with_solid_header() {
        check(&parse(&binary(&cube(), b"solid but actually binary")).unwrap());
    }

    #[test]
    fn ascii_cube() {
        check(&parse(&ascii(&cube())).unwrap());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse(b"hola").is_err());
        assert!(parse(b"solid vacio\nendsolid\n").is_err());
    }
}
