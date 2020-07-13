extern crate gl;
extern crate glfw;

use gl::types::*;
use glfw::{Action, Context, Glfw, InitError, Key, Window, WindowEvent};
use std::ffi::CString;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use std::{mem, process, ptr};

const INIT_WIDTH: u32 = 800;
const INIT_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 pos;

void main() {
    gl_Position = vec4(pos.x, pos.y, pos.z, 1.0f);
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
out vec4 FragColor;

void main() {
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}
"#;

fn configure_glfw() -> Result<Glfw, InitError> {
    match glfw::init(glfw::FAIL_ON_ERRORS) {
        Ok(mut glfw_obj) => {
            glfw_obj.window_hint(glfw::WindowHint::OpenGlProfile(
                glfw::OpenGlProfileHint::Core,
            ));
            glfw_obj.window_hint(glfw::WindowHint::ContextVersion(3, 3));
            #[cfg(target_os = "macos")]
            glfw_obj.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
            Ok(glfw_obj)
        }
        Err(e) => Err(e),
    }
}

fn create_window(glfw_obj: &mut Glfw) -> Option<(Window, Receiver<(f64, WindowEvent)>)> {
    match glfw_obj.create_window(
        INIT_WIDTH,
        INIT_HEIGHT,
        "Learn OpenGL",
        glfw::WindowMode::Windowed,
    ) {
        Some((mut window, events)) => {
            window.make_current();
            window.set_key_polling(true);
            window.set_framebuffer_size_polling(true);
            Some((window, events))
        }
        None => None,
    }
}

fn configure_gl(window: &mut Window) {
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
}

unsafe fn setup_program() -> GLuint {
    let vertex_shader_obj = gl::CreateShader(gl::VERTEX_SHADER);
    let vertex_shader_src = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
    gl::ShaderSource(
        vertex_shader_obj,
        1,
        &vertex_shader_src.as_ptr(),
        ptr::null(),
    );
    gl::CompileShader(vertex_shader_obj);
    if let ShaderCompileStatus::FAILURE(err_msg) = get_shader_compile_status(vertex_shader_obj) {
        eprintln!("Vertex shader compilation failed with error: {}", err_msg);
        process::exit(1);
    }

    let fragment_shader_obj = gl::CreateShader(gl::FRAGMENT_SHADER);
    let fragment_shader_src = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
    gl::ShaderSource(
        fragment_shader_obj,
        1,
        &fragment_shader_src.as_ptr(),
        ptr::null(),
    );
    gl::CompileShader(fragment_shader_obj);
    if let ShaderCompileStatus::FAILURE(err_msg) = get_shader_compile_status(fragment_shader_obj) {
        eprintln!("Fragment shader compilation failed with error: {}", err_msg);
        process::exit(1);
    }

    let shader_program = gl::CreateProgram();
    gl::AttachShader(shader_program, vertex_shader_obj);
    gl::AttachShader(shader_program, fragment_shader_obj);
    gl::LinkProgram(shader_program);

    let mut link_success = gl::FALSE as GLint;
    let mut link_log = Vec::with_capacity(512);
    link_log.set_len(link_log.capacity() - 1);
    gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut link_success);
    if link_success != gl::TRUE as GLint {
        gl::GetProgramInfoLog(
            shader_program,
            512,
            ptr::null_mut(),
            link_log.as_mut_ptr() as *mut GLchar,
        );
        eprintln!(
            "Shader program linking failed with error: {}",
            String::from_utf8(link_log).unwrap()
        );
        process::exit(1);
    }

    gl::DeleteShader(vertex_shader_obj);
    gl::DeleteShader(fragment_shader_obj);

    shader_program
}

enum ShaderCompileStatus {
    SUCCESSFUL,
    FAILURE(String),
}

unsafe fn get_shader_compile_status(shader_obj: GLuint) -> ShaderCompileStatus {
    let mut compile_success = gl::FALSE as GLint;
    let mut compile_log = Vec::with_capacity(512);
    compile_log.set_len(compile_log.capacity() - 1);
    gl::GetShaderiv(shader_obj, gl::COMPILE_STATUS, &mut compile_success);
    if compile_success != gl::TRUE as GLint {
        gl::GetShaderInfoLog(
            shader_obj,
            512,
            ptr::null_mut(),
            compile_log.as_mut_ptr() as *mut GLchar,
        );
        ShaderCompileStatus::FAILURE(String::from_utf8(compile_log).unwrap())
    } else {
        ShaderCompileStatus::SUCCESSFUL
    }
}

fn setup_scene() -> (GLuint, GLuint) {
    unsafe {
        let shader_program = setup_program();

        let vertices: [f32; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

        let (mut triangle_buffer_obj, mut triangle_array_obj) = (0, 0);
        gl::GenVertexArrays(1, &mut triangle_array_obj);
        gl::GenBuffers(1, &mut triangle_buffer_obj);

        // Bind objects
        gl::BindVertexArray(triangle_array_obj);
        gl::BindBuffer(gl::ARRAY_BUFFER, triangle_buffer_obj);

        // Setup buffer data and properties
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Unbind objects
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        (shader_program, triangle_array_obj)
    }
}

pub fn main() {
    let mut glfw_obj;
    let mut window;
    let events;

    match configure_glfw() {
        Ok(glfw_result) => {
            glfw_obj = glfw_result;
            match create_window(&mut glfw_obj) {
                Some(result) => {
                    window = result.0;
                    events = result.1;
                    configure_gl(&mut window);
                }
                None => {
                    eprintln!("Exiting due to GLFW Window creation failure.");
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("GLFW initialization failed with error: {}", e);
            process::exit(1);
        }
    }

    let (shader_program, scene_array_obj) = setup_scene();

    while !window.should_close() {
        // Process Events
        process_events(&mut window, &events);

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            gl::BindVertexArray(scene_array_obj);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        // Swap buffer and poll events
        window.swap_buffers();
        glfw_obj.poll_events();
    }
}

fn process_events(window: &mut Window, events: &Receiver<(f64, WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
            },
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}
