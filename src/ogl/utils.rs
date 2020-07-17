use gl::types::*;
use std::ffi::CString;
use std::ptr;

pub unsafe fn build_shader(shader: &str, shader_type: GLenum) -> Result<GLuint, String> {
    let shader = CString::new(shader.as_bytes()).unwrap();
    let shader_id = gl::CreateShader(shader_type);
    gl::ShaderSource(shader_id, 1, &shader.as_ptr(), ptr::null());
    gl::CompileShader(shader_id);
    match get_shader_compile_status(shader_id) {
        Ok(_) => Ok(shader_id),
        Err(msg) => Err(msg),
    }
}

pub unsafe fn clean_shader(shader_id: GLuint) {
    gl::DeleteShader(shader_id);
}

pub unsafe fn build_program(
    vertex_shader_id: GLuint,
    fragment_shader_id: GLuint,
) -> Result<GLuint, String> {
    let program_id = gl::CreateProgram();
    gl::AttachShader(program_id, vertex_shader_id);
    gl::AttachShader(program_id, fragment_shader_id);
    gl::LinkProgram(program_id);

    let mut link_success = gl::FALSE as GLint;
    let mut link_log = Vec::with_capacity(512);
    link_log.set_len(link_log.capacity() - 1);
    gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut link_success);
    if link_success != gl::TRUE as GLint {
        gl::GetProgramInfoLog(
            program_id,
            512,
            ptr::null_mut(),
            link_log.as_mut_ptr() as *mut GLchar,
        );
        Err(format!(
            "Program build failed: {}",
            String::from_utf8(link_log).unwrap()
        ))
    } else {
        Ok(program_id)
    }
}

unsafe fn get_shader_compile_status(shader_id: GLuint) -> Result<(), String> {
    let mut compile_success = gl::FALSE as GLint;
    let mut compile_log = Vec::with_capacity(512);
    compile_log.set_len(compile_log.capacity() - 1);
    gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut compile_success);
    if compile_success != gl::TRUE as GLint {
        gl::GetShaderInfoLog(
            shader_id,
            512,
            ptr::null_mut(),
            compile_log.as_mut_ptr() as *mut GLchar,
        );
        Err(format!(
            "Shader compilation failed: {}",
            String::from_utf8(compile_log).unwrap()
        ))
    } else {
        Ok(())
    }
}
