// SPDX-License-Identifier: GPL-3.0-or-later
use glam::{Mat4, Vec3, Vec4};

pub const FOV_Y: f32 = 35.0 * std::f32::consts::PI / 180.0;

/// Cámara orbital con Z hacia arriba (convención habitual en STL/impresión 3D).
pub struct Camera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { target: Vec3::ZERO, distance: 10.0, yaw: -1.0, pitch: 0.45, radius: 1.0 }
    }
}

impl Camera {
    /// Encuadra la caja `min..max` con la vista isométrica por defecto.
    pub fn fit(&mut self, min: Vec3, max: Vec3) {
        self.target = (min + max) * 0.5;
        self.radius = ((max - min).length() * 0.5).max(1e-3);
        self.distance = self.radius / (FOV_Y * 0.5).sin() * 1.1;
        self.yaw = -std::f32::consts::FRAC_PI_3;
        self.pitch = 0.45;
    }

    fn dir(&self) -> Vec3 {
        let (sp, cp) = self.pitch.sin_cos();
        let (sy, cy) = self.yaw.sin_cos();
        Vec3::new(cp * cy, cp * sy, sp)
    }

    pub fn eye(&self) -> Vec3 {
        self.target + self.dir() * self.distance
    }

    pub fn view(&self) -> Mat4 {
        look_at(self.eye(), self.target, Vec3::Z)
    }

    pub fn proj(&self, aspect: f32) -> Mat4 {
        let near = self.distance * 0.01;
        let far = self.distance * 10.0 + self.radius * 4.0;
        perspective(FOV_Y, aspect.max(1e-3), near, far)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        let lim = 89.0_f32.to_radians();
        self.yaw -= dx * 0.01;
        self.pitch = (self.pitch + dy * 0.01).clamp(-lim, lim);
    }

    /// `dx`, `dy` en píxeles lógicos; `height` es el alto del área en píxeles lógicos.
    pub fn pan(&mut self, dx: f32, dy: f32, height: f32) {
        let fwd = -self.dir();
        let right = fwd.cross(Vec3::Z).normalize_or_zero();
        let up = right.cross(fwd).normalize_or_zero();
        let scale = 2.0 * self.distance * (FOV_Y * 0.5).tan() / height.max(1.0);
        self.target += (-dx * right + dy * up) * scale;
    }

    pub fn zoom(&mut self, steps: f32) {
        self.distance = (self.distance * 1.15_f32.powf(steps))
            .clamp(self.radius * 0.02, self.radius * 200.0);
    }
}

/// Matriz de vista derecha (equivalente a gluLookAt).
fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let f = (center - eye).normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);
    Mat4::from_cols(
        Vec4::new(s.x, u.x, -f.x, 0.0),
        Vec4::new(s.y, u.y, -f.y, 0.0),
        Vec4::new(s.z, u.z, -f.z, 0.0),
        Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
    )
}

/// Proyección perspectiva estilo OpenGL (profundidad en [-1, 1]).
fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y * 0.5).tan();
    let nf = 1.0 / (near - far);
    Mat4::from_cols(
        Vec4::new(f / aspect, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f, 0.0, 0.0),
        Vec4::new(0.0, 0.0, (far + near) * nf, -1.0),
        Vec4::new(0.0, 0.0, 2.0 * far * near * nf, 0.0),
    )
}
