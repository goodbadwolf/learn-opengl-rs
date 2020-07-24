extern crate gl;
extern crate glfw;

mod ogl;

use crate::ogl::graphics::{ShaderProgram, Texture};
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
layout (location = 1) in vec3 a_color;
layout (location = 2) in vec2 a_tex_coords;

out vec3 o_color;
out vec2 o_tex_coords;

void main() {
    gl_Position = vec4(a_pos, 1.0f);
    o_color = a_color;
    o_tex_coords = a_tex_coords;
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
uniform sampler2D a_texture;

in vec3 o_color;
in vec2 o_tex_coords;

out vec4 frag_color;

void main() {
    frag_color = texture(a_texture, o_tex_coords);
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

fn setup_scene() -> (ShaderProgram, GLuint, GLuint) {
    unsafe {
        let shader_program = setup_program();

        #[rustfmt::skip]
        let scene_vertices = [
            //  X         Y        Z        R        G        B       S        T
             0.5_f32,  0.5_f32, 0.0_f32, 1.0_f32, 0.0_f32, 0.0_f32, 1.0_f32, 1.0_f32,
             0.5_f32, -0.5_f32, 0.0_f32, 0.0_f32, 1.0_f32, 0.0_f32, 1.0_f32, 0.0_f32,
            -0.5_f32, -0.5_f32, 0.0_f32, 0.0_f32, 0.0_f32, 1.0_f32, 0.0_f32, 0.0_f32,
            -0.5_f32,  0.5_f32, 0.0_f32, 1.0_f32, 1.0_f32, 0.0_f32, 0.0_f32, 1.0_f32,
        ];

        #[rustfmt::skip]
        let scene_indices = [
            0, 1, 3, // First triangle
            1, 2, 3  // Second triangle
        ];

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
            &scene_indices[0] as *const i32 as *const c_void,
            gl::STATIC_DRAW,
        );

        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        // a_pos attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        // a_color attribute
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void
        );
        gl::EnableVertexAttribArray(1);
        // a_tex_coords attribute
        gl::VertexAttribPointer(
            2,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<GLfloat>()) as *const c_void
        );
        gl::EnableVertexAttribArray(2);


        // Unbind VAO
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        let mut container_texture = Texture::from_file("resources/images/container.jpg").expect("Failed loading texture file");
        container_texture.load();

        // ogl::PolygonMode(ogl::FRONT_AND_BACK, ogl::LINE);

        (shader_program, scene_array_obj, container_texture.id)
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

    let (shader_program, scene_array_obj, scene_tex_obj) = setup_scene();
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
            // gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, scene_tex_obj);
            gl::BindVertexArray(scene_array_obj);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
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
