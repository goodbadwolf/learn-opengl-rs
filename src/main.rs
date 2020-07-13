extern crate gl;
extern crate glfw;

use glfw::{Action, Context, Glfw, InitError, Key, Window, WindowEvent};
use std::process;
use std::sync::mpsc::Receiver;

const INIT_WIDTH: u32 = 800;
const INIT_HEIGHT: u32 = 600;

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
                None => panic!("Exiting due to GLFW Window creation failure."),
            }
        }
        Err(e) => {
            eprintln!("GLFW initialization failed with error: {}", e);
            process::exit(1);
        }
    }

    while !window.should_close() {
        // Process Events
        process_events(&mut window, &events);

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
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
