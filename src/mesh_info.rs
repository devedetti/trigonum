// SPDX-License-Identifier: GPL-3.0-or-later
use glam::Vec3;

/// Estadísticas del modelo, acumuladas mientras se parsea.
#[derive(Debug, Clone)]
pub struct MeshInfo {
    pub triangles: usize,
    pub min: Vec3,
    pub max: Vec3,
    /// mm³ (volumen firmado vía tetraedros contra el origen; se reporta en valor absoluto).
    signed_volume: f64,
    /// mm²
    pub area: f64,
}

impl Default for MeshInfo {
    fn default() -> Self {
        Self {
            triangles: 0,
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
            signed_volume: 0.0,
            area: 0.0,
        }
    }
}

impl MeshInfo {
    pub fn add_triangle(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        self.triangles += 1;
        self.min = self.min.min(a).min(b).min(c);
        self.max = self.max.max(a).max(b).max(c);
        let (a, b, c) = (a.as_dvec3(), b.as_dvec3(), c.as_dvec3());
        self.signed_volume += a.dot(b.cross(c)) / 6.0;
        self.area += (b - a).cross(c - a).length() / 2.0;
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn volume(&self) -> f64 {
        self.signed_volume.abs()
    }

}
