use gl::types::*;

use crate::ogl::utils::{build_program, build_shader, clean_shader};
use std::ffi::CStr;

pub struct ShaderProgram {
    pub id: GLuint,
}

impl ShaderProgram {
    pub fn with_shaders(
        vertex_shader_src: &str,
        fragment_shader_src: &str,
    ) -> Result<ShaderProgram, String> {
        unsafe {
            let mut vertex_shader: GLuint = 0;
            let mut fragment_shader: GLuint = 0;

            build_shader(vertex_shader_src, gl::VERTEX_SHADER)
                .and_then(|vertex_shader_id| {
                    vertex_shader = vertex_shader_id;
                    build_shader(fragment_shader_src, gl::FRAGMENT_SHADER)
                })
                .and_then(|fragment_shader_id| {
                    fragment_shader = fragment_shader_id;
                    build_program(vertex_shader, fragment_shader)
                })
                .and_then(|program_id| {
                    clean_shader(vertex_shader);
                    clean_shader(fragment_shader);
                    Ok(ShaderProgram { id: program_id })
                })
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_bool(&self, name: &CStr, value: bool) {
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value as i32);
        }
    }

    pub fn set_int(&self, name: &CStr, value: i32) {
        unsafe {
            gl::Uniform1i(gl::GetUniformLocation(self.id, name.as_ptr()), value);
        }
    }

    pub fn set_float(&self, name: &CStr, value: f32) {
        unsafe {
            gl::Uniform1f(gl::GetUniformLocation(self.id, name.as_ptr()), value);
        }
    }

    pub fn set_vec3f(&self, name: &CStr, value: [f32; 3]) {
        unsafe {
            gl::Uniform3fv(
                gl::GetUniformLocation(self.id, name.as_ptr()),
                1,
                value.as_ptr(),
            );
        }
    }
}
