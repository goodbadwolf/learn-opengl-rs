mod math;
mod ogl;

use crate::ogl::graphics::{ShaderProgram, Texture};
use gl::types::*;
use glfw::{Action, Context, Glfw, InitError, Key, Window, WindowEvent};
use glm::{Mat4, Vec3};
use nalgebra_glm as glm;
use std::ffi::CString;
use std::os::raw::c_void;
use std::sync::mpsc::Receiver;
use std::{mem, process, ptr};

const INIT_WIDTH: u32 = 800;
const INIT_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_tex_coords;

uniform mat4 world_from_object;
uniform mat4 view_from_world;
uniform mat4 projection_from_view;

out vec2 o_tex_coords;

void main() {
    mat4 projection_from_object = projection_from_view * view_from_world * world_from_object;
    gl_Position = projection_from_object * vec4(a_pos, 1.0f);
    o_tex_coords = a_tex_coords;
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
uniform sampler2D a_texture1;
uniform sampler2D a_texture2;

in vec2 o_tex_coords;

out vec4 frag_color;

void main() {
    frag_color = mix(texture(a_texture1, o_tex_coords), texture(a_texture2, o_tex_coords), 0.2f);
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

fn setup_scene() -> (ShaderProgram, GLuint, Vec<GLuint>, Vec<Vec3>) {
    unsafe {
        let shader_program = setup_program();

        #[rustfmt::skip]
        let scene_vertices = [
            //    X         Y         Z        S        T
            -0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 0.0_f32,
            0.5_f32, -0.5_f32, -0.5_f32, 1.0_f32, 0.0_f32,
            0.5_f32,  0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
            0.5_f32,  0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
           -0.5_f32,  0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
           -0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 0.0_f32,

           -0.5_f32, -0.5_f32,  0.5_f32, 0.0_f32, 0.0_f32,
            0.5_f32, -0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
            0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 1.0_f32,
            0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 1.0_f32,
           -0.5_f32,  0.5_f32,  0.5_f32, 0.0_f32, 1.0_f32,
           -0.5_f32, -0.5_f32,  0.5_f32, 0.0_f32, 0.0_f32,

           -0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
           -0.5_f32,  0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
           -0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
           -0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
           -0.5_f32, -0.5_f32,  0.5_f32, 0.0_f32, 0.0_f32,
           -0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,

            0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
            0.5_f32,  0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
            0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
            0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
            0.5_f32, -0.5_f32,  0.5_f32, 0.0_f32, 0.0_f32,
            0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,

           -0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
            0.5_f32, -0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
            0.5_f32, -0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
            0.5_f32, -0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
           -0.5_f32, -0.5_f32,  0.5_f32, 0.0_f32, 0.0_f32,
           -0.5_f32, -0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,

           -0.5_f32,  0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
            0.5_f32,  0.5_f32, -0.5_f32, 1.0_f32, 1.0_f32,
            0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
            0.5_f32,  0.5_f32,  0.5_f32, 1.0_f32, 0.0_f32,
           -0.5_f32,  0.5_f32,  0.5_f32, 0.0_f32, 0.0_f32,
           -0.5_f32,  0.5_f32, -0.5_f32, 0.0_f32, 1.0_f32,
        ];

        #[rustfmt::skip]
        let scene_indices = [
            0, 1, 3, // First triangle
            1, 2, 3  // Second triangle
        ];

        #[rustfmt::skip]
        let cube_centers: [(f32, f32, f32); 10] = [
            ( 0.0_f32,   0.0_f32,   0.0_f32),
            ( 2.0_f32,   5.0_f32, -15.0_f32),
            (-1.5_f32,  -2.2_f32,  -2.0_f32),
            (-3.8_f32,  -2.0_f32, -12.3_f32),
            ( 2.4_f32,  -0.4_f32,  -3.5_f32),
            (-1.7_f32,   3.0_f32,  -7.5_f32),
            ( 1.3_f32,  -2.0_f32,  -2.5_f32),
            ( 1.5_f32,   2.0_f32,  -2.5_f32),
            ( 1.5_f32,   0.2_f32,  -1.5_f32),
            (-1.3_f32,   1.0_f32,  -1.5_f32),
        ];
        let mut cube_positions: Vec<Vec3> = vec![];
        for center in cube_centers.iter() {
            cube_positions.push(glm::vec3(center.0, center.1, center.2));
        }

        let (mut scene_buffer_obj, mut scene_array_obj, mut scene_element_buffer_obj) =
            (0_u32, 0_u32, 0_u32);
        gl::Enable(gl::DEPTH_TEST);

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

        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        // a_pos attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // a_tex_coords attribute
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);

        // Unbind VAO
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        let mut container_texture = Texture::from_file("resources/images/container.jpg", false)
            .expect("Failed loading texture file");
        container_texture.load();
        let mut face_texture = Texture::from_file("resources/images/awesomeface.png", false)
            .expect("Failed loading texture file");
        face_texture.load();

        shader_program.use_program();
        shader_program.set_int(&CString::new("a_texture1").unwrap(), 0);
        shader_program.set_int(&CString::new("a_texture2").unwrap(), 1);
        // ogl::PolygonMode(ogl::FRONT_AND_BACK, ogl::LINE);

        (
            shader_program,
            scene_array_obj,
            vec![container_texture.id, face_texture.id],
            cube_positions,
        )
    }
}

fn setup_coordinate_systems(_: &Glfw) -> (Mat4, Mat4) {
    let mut view_from_world = glm::Mat4::identity();
    view_from_world = glm::translate(&view_from_world, &glm::vec3(0.0_f32, 0.0_f32, -3.0_f32));

    let aspect_ratio = (INIT_WIDTH as f32) / (INIT_HEIGHT as f32);
    let angle = 45.0_f32;
    let projection_from_view =
        glm::perspective(aspect_ratio, angle.to_radians(), 0.1_f32, 100.0_f32);

    (view_from_world, projection_from_view)
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

    let (shader_program, scene_array_obj, scene_tex_objs, cube_positions) = setup_scene();
    let (view_from_world, projection_from_view) = setup_coordinate_systems(&glfw_obj);
    let world_from_object_name = CString::new("world_from_object").unwrap();
    let view_from_world_name = CString::new("view_from_world").unwrap();
    let projection_from_view_name = CString::new("projection_from_view").unwrap();

    while !window.should_close() {
        // Process Events
        process_events(&mut window, &events);

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            shader_program.use_program();
            for (tex_i, tex_obj) in scene_tex_objs.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + tex_i as u32);
                gl::BindTexture(gl::TEXTURE_2D, *tex_obj);
            }
            gl::BindVertexArray(scene_array_obj);
            shader_program.set_mat4f(&view_from_world_name, &view_from_world);
            shader_program.set_mat4f(&projection_from_view_name, &projection_from_view);
            for (i, position) in cube_positions.iter().enumerate() {
                let mut world_from_object = glm::Mat4::identity();
                let angle = (20.0_f32 * (i + 1) as f32 * glfw_obj.get_time() as f32).to_radians();
                world_from_object = glm::translate(&world_from_object, &position);
                world_from_object = glm::rotate(
                    &world_from_object,
                    angle,
                    &glm::vec3(1.0_f32, 0.3_f32, 0.5_f32),
                );
                shader_program.set_mat4f(&world_from_object_name, &world_from_object);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
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
