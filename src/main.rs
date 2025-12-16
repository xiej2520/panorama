use std::{f32::consts::PI, path::PathBuf, time::Duration};

use glam::{Mat3, Mat4, Quat, Vec3};
use glow::*;
use panorama::{
    ExpectErr, PrintErr,
    loader::{create_faces, load_cubemap},
};
use sdl2::{EventPump, event::Event, video::Window};

fn main() {
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

        let cubemap_texture = load_cubemap(&gl, create_faces(&PathBuf::new()));

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(cubemap_texture));
        let skybox_loc = gl.get_uniform_location(skybox_program, "skybox");
        gl.uniform_1_i32(skybox_loc.as_ref(), 0);

        //gl.enable(glow::DEPTH_TEST);

        let mut theta = 0.0f32;
        let fps_target = 30;
        let frame_sleep_time_ns = 1_000_000_000u32 / fps_target;
        // minecraft panorama rotates once every 90 seconds at default speed
        //let period = 90.0;
        let period = 20.0;
        let dtheta = -2.0 * PI / (period * fps_target as f32);

        let (width, height) = window.drawable_size();
        let mut video_writer = Some(VideoWriter::new(width, height, "output.mp4"));

        'render: loop {
            {
                if let ShouldQuit(true) = handle_events(&mut window, &mut event_pump) {
                    break 'render;
                }
            }
            let (width, height) = window.drawable_size();
            let aspect_ratio = width as f32 / height as f32;
            gl.viewport(0, 0, width as i32, height as i32);

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.depth_mask(false);
            gl.use_program(Some(skybox_program));
            // minecraft uses 85.0 fov for panorama in 1.21.11
            let fov = 85.0f32;
            let projection = Mat4::perspective_rh_gl(fov.to_radians(), aspect_ratio, 0.1, 100.0);

            let target = Quat::from_rotation_y(theta)
                * Quat::from_rotation_x(10.0f32.to_radians())
                * Vec3::Z;
            let camera_view = Mat4::look_at_rh(Vec3::ZERO, target, Vec3::Y);
            theta += dtheta;

            // remove translation
            let view = Mat4::from_mat3(Mat3::from_mat4(camera_view));

            let proj_loc = gl.get_uniform_location(skybox_program, "projection");
            let view_loc = gl.get_uniform_location(skybox_program, "view");
            gl.uniform_matrix_4_f32_slice(proj_loc.as_ref(), false, &projection.to_cols_array());
            gl.uniform_matrix_4_f32_slice(view_loc.as_ref(), false, &view.to_cols_array());

            gl.bind_vertex_array(Some(skybox_vao));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(cubemap_texture));
            let skybox_loc = gl.get_uniform_location(skybox_program, "skybox");
            gl.uniform_1_i32(skybox_loc.as_ref(), 0);
            gl.draw_elements(glow::TRIANGLES, INDICES.len() as i32, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
            gl.depth_mask(true);

            window.gl_swap_window();

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

            if theta < -2.0 * PI
                && let Some(video_writer) = video_writer.take()
            {
                video_writer.finish();
            } else if let Some(video_writer) = &mut video_writer {
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

const VERTEX_SHADER_CUBE_SOURCE: &str = r#"#version 330
layout (location = 0) in vec3 aPos;
out vec3 TexCoords;

uniform mat4 projection;
uniform mat4 view;

void main() {
  TexCoords = aPos;
  gl_Position = projection * view * vec4(aPos, 1.0);
}"#;
const FRAGMENT_SHADER_CUBE_SOURCE: &str = r#"#version 330
out vec4 FragColor;

in vec3 TexCoords;
uniform samplerCube skybox;

void main() {
  FragColor = texture(skybox, TexCoords);
}"#;

struct ShouldQuit(bool);

fn handle_events(window: &mut Window, event_pump: &mut EventPump) -> ShouldQuit {
    use sdl2::keyboard::Keycode;
    for event in event_pump.poll_iter() {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::ESCAPE),
                ..
            }
            | Event::Quit { .. } => return ShouldQuit(true),
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
            _ => {}
        }
    }
    ShouldQuit(false)
}

pub struct VideoWriter {
    ffmpeg: std::process::Child,
    stdin: std::process::ChildStdin,
}

impl VideoWriter {
    pub fn new(width: u32, height: u32, output: &str) -> Self {
        #[rustfmt::skip]
        let mut ffmpeg = std::process::Command::new("ffmpeg")
            .args([
                "-y",                     // overwrite output
                "-f", "rawvideo",
                "-pix_fmt", "rgba",
                "-s", &format!("{}x{}", width, height),
                "-r", "60",               // frame rate
                "-i", "-",
                "-c:v", "libx264",
                "-preset", "slow",
                "-crf", "20",
                "-pix_fmt", "yuv420p",    // required for mp4 compatibility
                "-vf", "vflip", // flip vertically
                output,
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg");

        let stdin = ffmpeg.stdin.take().expect("Failed to open stdin");

        Self { ffmpeg, stdin }
    }

    pub fn write_frame(&mut self, frame: &[u8]) {
        std::io::Write::write_all(&mut self.stdin, frame).expect("Failed to write frame");
    }

    pub fn finish(mut self) {
        drop(self.stdin); // VERY IMPORTANT: signals EOF to ffmpeg
        self.ffmpeg.wait().expect("ffmpeg failed");
    }
}
