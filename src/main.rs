extern crate gl;
extern crate glfw;

mod ogl;

use crate::ogl::graphics::ShaderProgram;
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
layout (location = 0) in vec3 a_pos;

void main() {
    gl_Position = vec4(a_pos, 1.0f);
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
uniform vec3 our_color;

out vec4 frag_color;

void main() {
    frag_color = vec4(our_color, 1.0f);
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

unsafe fn configure_gl(window: &mut Window) {
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
}

unsafe fn setup_program() -> ShaderProgram {
    ShaderProgram::with_shaders(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
        .expect("Program setup failure")
}

fn setup_scene() -> (ShaderProgram, GLuint) {
    unsafe {
        let shader_program = setup_program();

        let scene_vertices = [
            0.75_f32, -0.75_f32, 0.0_f32, -0.75_f32, -0.75_f32, 0.0_f32, 0.0_f32, 0.75_f32, 0.0_f32,
        ];
        let scene_indices = [0_u32, 1_u32, 2_u32];

        let (mut scene_buffer_obj, mut scene_array_obj, mut scene_element_buffer_obj) =
            (0_u32, 0_u32, 0_u32);
        gl::GenVertexArrays(1, &mut scene_array_obj);
        gl::GenBuffers(1, &mut scene_buffer_obj);
        gl::GenBuffers(1, &mut scene_element_buffer_obj);

        // Bind VAO
        gl::BindVertexArray(scene_array_obj);

        // Setup vertices data and properties
        gl::BindBuffer(gl::ARRAY_BUFFER, scene_buffer_obj);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (scene_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &scene_vertices[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, scene_element_buffer_obj);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (scene_indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
            &scene_indices[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );
        // a_pos attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Unbind VAO
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        // ogl::PolygonMode(ogl::FRONT_AND_BACK, ogl::LINE);

        (shader_program, scene_array_obj)
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
                    unsafe {
                        configure_gl(&mut window);
                    }
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
    shader_program.use_program();
    let color_var_name = CString::new("our_color").unwrap();

    while !window.should_close() {
        // Process Events
        process_events(&mut window, &events);

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            shader_program.use_program();
            let color = [
                (glfw_obj.get_time() as f32).sin() * 0.5_f32 + 0.5_f32,
                (glfw_obj.get_time() as f32).cos() * 0.5_f32 + 0.5_f32,
                (glfw_obj.get_time() as f32).tanh() * 0.5_f32 + 0.5_f32,
            ];
            shader_program.set_vec3f(&color_var_name, color);
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
