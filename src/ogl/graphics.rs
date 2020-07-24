use gl::types::*;

use crate::ogl::utils::{build_program, build_shader, clean_shader};
use image::{DynamicImage, GenericImageView, Pixel, Rgb};
use std::ffi::{c_void, CStr};
use std::path::Path;

pub struct ShaderProgram {
    pub id: GLuint,
}

pub struct Texture {
    pub id: GLuint,
    pub width: u32,
    pub height: u32,
    data: Vec<[u8; 3]>,
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

impl Texture {
    pub unsafe fn from_file(file_path: &str) -> Result<Texture, String> {
        Self::load_data_from_file(file_path).and_then(|(width, height, data)| {
            let mut texture_obj_id: GLuint = 0;
            gl::GenTextures(1, &mut texture_obj_id);
            Ok(Texture {
                id: texture_obj_id,
                width,
                height,
                data,
            })
        })
    }

    pub unsafe fn load(&mut self) {
        gl::BindTexture(gl::TEXTURE_2D, self.id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            self.width as i32,
            self.height as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            self.data[0].as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        self.data.clear();
    }

    fn load_data_from_file(file_path: &str) -> Result<(u32, u32, Vec<[u8; 3]>), String> {
        match image::open(Path::new(file_path)) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let data: Vec<_> = img.into_rgb().pixels().map(|p| p.0).collect();
                Ok((width, height, data))
            }
            Err(err) => Err(err.to_string()),
        }
    }
}