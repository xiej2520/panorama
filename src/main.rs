use std::{path::PathBuf, time::Duration};

use argh::FromArgs;
use glam::{Mat4, Vec3};
use glow::*;
use panorama::{ExpectErr, PrintErr, camera::Camera, loader::load_cubemap, recorder::VideoWriter};
use sdl2::{EventPump, event::Event, video::Window};

#[derive(FromArgs)]
/// panorama
struct Args {
    /// path to panorama_<x>.png image files or resource pack
    #[argh(positional)]
    path: Option<String>,

    /// fps target
    #[argh(option, default = "30")]
    fps: u32,
}

fn main() {
    let args: Args = argh::from_env();
    let path = PathBuf::from(args.path.unwrap_or_default());

    unsafe {
        let (gl, mut window, mut event_pump, _context) = create_sdl2_context();
        //window.subsystem().gl_set_swap_interval(SwapInterval::VSync).print_err();

        let skybox_program =
            create_program(&gl, VERTEX_SHADER_CUBE_SOURCE, FRAGMENT_SHADER_CUBE_SOURCE);
        let skybox_vao = gl.create_vertex_array().expect_else_err();
        let skybox_vbo = gl.create_buffer().expect_else_err();
        let skybox_ebo = gl.create_buffer().expect_else_err();

        gl.bind_vertex_array(Some(skybox_vao));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(skybox_vbo));
        let vertices_u8 = core::slice::from_raw_parts(
            VERTICES.as_ptr() as *const u8,
            VERTICES.len() * size_of::<f32>(),
        );
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(skybox_ebo));
        let indices_u8 = core::slice::from_raw_parts(
            INDICES.as_ptr() as *const u8,
            INDICES.len() * size_of::<u32>(),
        );
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STATIC_DRAW);

        gl.bind_vertex_array(None);

        let cubemap_texture = load_cubemap(&gl, &path);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(cubemap_texture));
        let skybox_loc = gl.get_uniform_location(skybox_program, "skybox");
        gl.uniform_1_i32(skybox_loc.as_ref(), 0);

        // inside cube, don't need depth test
        //gl.enable(glow::DEPTH_TEST);
        //gl.depth_mask(false);

        let fps_target = args.fps;
        let frame_sleep_time_ns = 1_000_000_000u32 / fps_target;

        let start_time = std::time::Instant::now();

        let mut state = State {
            should_quit: false,
            last_time: start_time.elapsed(),
            video_writer: None,
            camera: Camera::new(Vec3::ZERO, Vec3::Y, 0.0, 0.0),
            auto_rotate: true,
        };
        // minecraft uses 85.0 fov for panorama in 1.21.11
        state.camera.fov = 85.0;
        // minecraft panorama rotates once every 90 seconds at default speed
        let period = 90.0;
        state.camera.rotation_speed = 360.0 / period;

        'render: loop {
            handle_events(&mut window, &mut event_pump, &mut state);
            if state.should_quit {
                break 'render;
            }

            let (width, height) = window.drawable_size();
            let aspect_ratio = width as f32 / height as f32;
            gl.viewport(0, 0, width as i32, height as i32);

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(skybox_program));
            let projection =
                Mat4::perspective_rh_gl(state.camera.fov.to_radians(), aspect_ratio, 0.1, 100.0);

            {
                let time = start_time.elapsed();
                if state.auto_rotate {
                    state
                        .camera
                        .process_rotation((time - state.last_time).as_micros());
                }
                state.last_time = time;
            }

            let view = state.camera.view_matrix();

            let proj_loc = gl.get_uniform_location(skybox_program, "projection");
            let view_loc = gl.get_uniform_location(skybox_program, "view");
            gl.uniform_matrix_4_f32_slice(proj_loc.as_ref(), false, projection.as_ref());
            gl.uniform_matrix_4_f32_slice(view_loc.as_ref(), false, view.as_ref());

            gl.bind_vertex_array(Some(skybox_vao));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(cubemap_texture));
            let skybox_loc = gl.get_uniform_location(skybox_program, "skybox");
            gl.uniform_1_i32(skybox_loc.as_ref(), 0);

            gl.draw_elements(glow::TRIANGLES, INDICES.len() as i32, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);

            window.gl_swap_window();

            if let Some(video_writer) = state.video_writer.as_mut() {
                let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize]; // 4 RGBA
                gl.read_pixels(
                    0,
                    0,
                    width as i32,
                    height as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    PixelPackData::Slice(Some(&mut buffer)),
                );

                video_writer.write_frame(&buffer);
            }

            std::thread::sleep(Duration::new(0, frame_sleep_time_ns));
        }

        gl.delete_program(skybox_program);
        gl.delete_vertex_array(skybox_vao);
        gl.delete_buffer(skybox_vbo);
        gl.delete_buffer(skybox_ebo);

        gl.delete_texture(cubemap_texture);
    }
}

fn create_sdl2_context() -> (
    glow::Context,
    sdl2::video::Window,
    sdl2::EventPump,
    sdl2::video::GLContext,
) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();
    let window = video
        .window("Panorama", 1024, 768)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| match video.gl_get_proc_address(s) {
            p if p.is_null() => panic!("Failed to get OpenGL video function"),
            s => s as *const _,
        })
    };
    let event_loop = sdl.event_pump().unwrap();

    (gl, window, event_loop, gl_context)
}

unsafe fn create_program(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> NativeProgram {
    let program = unsafe { gl.create_program().expect("Cannot create program") };

    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        unsafe {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, shader_source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }
    }

    unsafe {
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
    }

    program
}

#[rustfmt::skip]
const VERTICES: [f32; 24] = [
    -1.0, 1.0, -1.0,  // front top    left
    -1.0, -1.0, -1.0, // front bottom left
    1.0, -1.0, -1.0,  // front bottom right
    1.0, 1.0, -1.0,   // front top    right
    -1.0, 1.0, 1.0,   // back  top    left
    -1.0, -1.0, 1.0,  // back  bottom left
    1.0, -1.0, 1.0,   // back  bottom right
    1.0, 1.0, 1.0,    // back  top    right
];

const INDICES: [u32; 36] = [
    0, 1, 2, 0, 2, 3, // front face
    4, 5, 6, 4, 6, 7, // back face
    4, 5, 1, 4, 1, 0, // left face
    3, 2, 6, 3, 6, 7, // right face
    4, 0, 3, 4, 3, 7, // top face
    1, 5, 6, 1, 6, 2, // bottom face
];

const VERTEX_SHADER_CUBE_SOURCE: &str = include_str!("shader.vs");
const FRAGMENT_SHADER_CUBE_SOURCE: &str = include_str!("shader.fs");

struct State {
    should_quit: bool,
    last_time: Duration,
    video_writer: Option<VideoWriter>,
    camera: Camera,
    auto_rotate: bool,
}

fn handle_events(window: &mut Window, event_pump: &mut EventPump, state: &mut State) {
    use sdl2::keyboard::Keycode;
    for event in event_pump.poll_iter() {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::ESCAPE),
                ..
            }
            | Event::Quit { .. } => state.should_quit = true,
            Event::KeyDown {
                keycode: Some(Keycode::F11),
                ..
            } => {
                match window.fullscreen_state() {
                    sdl2::video::FullscreenType::Off => {
                        window.set_fullscreen(sdl2::video::FullscreenType::Desktop)
                    }
                    _ => window.set_fullscreen(sdl2::video::FullscreenType::Off),
                }
                .print_err();
            }
            Event::KeyDown {
                keycode: Some(Keycode::R),
                ..
            } => {
                if let Some(video_writer) = state.video_writer.take() {
                    video_writer.finish();
                } else {
                    let (width, height) = window.drawable_size();
                    state.video_writer = Some(VideoWriter::new(width, height, "output.mp4"));
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::LEFT),
                ..
            } => {
                state.camera.rotation_speed -= 0.5;
            }
            Event::KeyDown {
                keycode: Some(Keycode::RIGHT),
                ..
            } => {
                state.camera.rotation_speed += 0.5;
            }
            Event::KeyDown {
                keycode: Some(Keycode::SPACE),
                ..
            } => {
                state.auto_rotate = !state.auto_rotate;
            }
            Event::MouseMotion {
                mousestate,
                xrel,
                yrel,
                ..
            } => {
                if mousestate.is_mouse_button_pressed(sdl2::mouse::MouseButton::Right) {
                    state
                        .camera
                        .process_mouse_movement(-xrel as f32, -yrel as f32);
                }
            }
            Event::MouseWheel { precise_y, .. } => {
                state.camera.process_mouse_scroll(precise_y);
            }
            _ => {}
        }
    }
}
