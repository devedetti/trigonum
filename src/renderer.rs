// SPDX-License-Identifier: GPL-3.0-or-later
use std::{ffi::c_void, sync::OnceLock};

use glam::Mat4;
use glow::HasContext;

/// GTK ya enlaza libepoxy, que expone cada función GL como una variable puntero
/// `epoxy_<nombre>` con un resolvedor que funciona tanto con EGL como con GLX.
fn gl_proc(name: &str) -> *const c_void {
    static LIB: OnceLock<Option<libloading::Library>> = OnceLock::new();
    let lib = LIB.get_or_init(|| unsafe { libloading::Library::new("libepoxy.so.0").ok() });
    let Some(lib) = lib else { return std::ptr::null() };
    let sym = format!("epoxy_{name}\0");
    unsafe {
        match lib.get::<*const *const c_void>(sym.as_bytes()) {
            Ok(var) => {
                let var: *const *const c_void = *var;
                if var.is_null() { std::ptr::null() } else { *var }
            }
            Err(_) => std::ptr::null(),
        }
    }
}

const VERT: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
uniform mat4 uProj;
uniform mat4 uView;
out vec3 vNormal;
out vec3 vBary;
void main() {
    gl_Position = uProj * uView * vec4(aPos, 1.0);
    vNormal = mat3(uView) * aNormal;
    int i = gl_VertexID % 3;
    vBary = vec3(i == 0, i == 1, i == 2);
}
"#;

const FRAG: &str = r#"#version 330 core
in vec3 vNormal;
in vec3 vBary;
uniform int uWire;
uniform vec3 uColor;
uniform vec3 uWireColor;
out vec4 fragColor;
void main() {
    vec3 n = normalize(vNormal);
    if (!gl_FrontFacing) n = -n;
    // Luz ligada a la cámara (espacio de vista), levemente arriba a la derecha.
    vec3 l = normalize(vec3(0.35, 0.45, 1.0));
    float diff = max(dot(n, l), 0.0);
    vec3 h = normalize(l + vec3(0.0, 0.0, 1.0));
    float spec = pow(max(dot(n, h), 0.0), 48.0) * 0.25;
    vec3 c = uColor * (0.22 + 0.78 * diff) + vec3(spec);
    if (uWire == 1) {
        vec3 d = fwidth(vBary);
        vec3 a = smoothstep(vec3(0.0), d * 1.3, vBary);
        float edge = min(min(a.x, a.y), a.z);
        c = mix(uWireColor, c, edge);
    }
    fragColor = vec4(c, 1.0);
}
"#;

pub struct Renderer {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    count: i32,
}

impl Renderer {
    /// Debe llamarse con el contexto GL del `GLArea` activo.
    pub fn new() -> Result<Self, String> {
        unsafe {
            let gl = glow::Context::from_loader_function(gl_proc);
            let program = gl.create_program()?;
            let mut shaders = Vec::new();
            for (kind, src) in [(glow::VERTEX_SHADER, VERT), (glow::FRAGMENT_SHADER, FRAG)] {
                let s = gl.create_shader(kind)?;
                gl.shader_source(s, src);
                gl.compile_shader(s);
                if !gl.get_shader_compile_status(s) {
                    return Err(format!("Shader: {}", gl.get_shader_info_log(s)));
                }
                gl.attach_shader(program, s);
                shaders.push(s);
            }
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(format!("Programa GL: {}", gl.get_program_info_log(program)));
            }
            for s in shaders {
                gl.detach_shader(program, s);
                gl.delete_shader(s);
            }

            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let stride = 6 * 4;
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, 12);
            gl.enable_vertex_attrib_array(1);
            gl.bind_vertex_array(None);

            Ok(Self { gl, program, vao, vbo, count: 0 })
        }
    }

    pub fn upload(&mut self, data: &[f32]) {
        let bytes =
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) };
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
        }
        self.count = (data.len() / 6) as i32;
    }

    pub fn draw(&self, view: Mat4, proj: Mat4, wire: bool, dark: bool) {
        let gl = &self.gl;
        unsafe {
            let bg = if dark { [0.13, 0.13, 0.15] } else { [0.93, 0.93, 0.94] };
            gl.clear_color(bg[0], bg[1], bg[2], 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            if self.count == 0 {
                return;
            }
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.use_program(Some(self.program));
            let loc = |n: &str| gl.get_uniform_location(self.program, n);
            gl.uniform_matrix_4_f32_slice(loc("uProj").as_ref(), false, &proj.to_cols_array());
            gl.uniform_matrix_4_f32_slice(loc("uView").as_ref(), false, &view.to_cols_array());
            gl.uniform_1_i32(loc("uWire").as_ref(), wire as i32);
            gl.uniform_3_f32(loc("uColor").as_ref(), 0.55, 0.66, 0.82);
            let wc = if dark { 0.05 } else { 0.15 };
            gl.uniform_3_f32(loc("uWireColor").as_ref(), wc, wc, wc + 0.03);
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, self.count);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    pub fn destroy(self) {
        unsafe {
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_program(self.program);
        }
    }
}
