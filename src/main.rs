use glam::{Mat3, Mat4, Quat, Vec3};
use glow::*;

fn main() {
    unsafe {
        let (gl, window, mut events_loop, _context) = create_sdl2_context();

        let program = create_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        gl.use_program(Some(program));

        let (vbo, vao) = create_vertex_buffer(&gl);

        set_uniform(&gl, program, "blue", 0.8);

        gl.clear_color(0.1, 0.2, 0.3, 1.0);

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

        let cubemap_texture = load_cubemap(&gl, &IMAGES);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(cubemap_texture));
        let location = gl.get_uniform_location(skybox_program, "skybox");
        gl.uniform_1_i32(location.as_ref(), 0);

        gl.enable(glow::DEPTH_TEST);

        let mut theta = 0.0f32;

        'render: loop {
            {
                for event in events_loop.poll_iter() {
                    if let sdl2::event::Event::Quit { .. } = event {
                        break 'render;
                    }
                }
            }
            let (width, height) = window.drawable_size();
            let aspect_ratio = width as f32 / height as f32;
            gl.viewport(0, 0, width as i32, height as i32);


            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(program));
            gl.bind_vertex_array(Some(vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_vertex_array(None);

            gl.depth_mask(false);
            gl.use_program(Some(skybox_program));
            let fov = 90.0f32;
            let projection = Mat4::perspective_rh_gl(fov.to_radians(), aspect_ratio, 0.1, 100.0);
            
            let target = Quat::from_rotation_y(theta) * Vec3::Z;
            let camera_view = Mat4::look_at_rh(Vec3::ZERO, target, Vec3::Y);
            theta -= 0.001;

            let view = Mat4::from_mat3(Mat3::from_mat4(camera_view));
            let proj_loc = gl.get_uniform_location(skybox_program, "projection");
            let view_loc = gl.get_uniform_location(skybox_program, "view");
            gl.uniform_matrix_4_f32_slice(proj_loc.as_ref(), false, &projection.to_cols_array());
            gl.uniform_matrix_4_f32_slice(view_loc.as_ref(), false, &view.to_cols_array());

            gl.bind_vertex_array(Some(skybox_vao));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(cubemap_texture));
            let location = gl.get_uniform_location(skybox_program, "skybox");
            gl.uniform_1_i32(location.as_ref(), 0);
            gl.draw_elements(glow::TRIANGLES, INDICES.len() as i32, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
            gl.depth_mask(true);

            window.gl_swap_window();
        }

        gl.delete_program(program);
        gl.delete_vertex_array(vao);
        gl.delete_buffer(vbo);

        gl.delete_program(skybox_program);
        gl.delete_vertex_array(skybox_vao);
        gl.delete_buffer(skybox_vbo);
        gl.delete_buffer(skybox_ebo);

        gl.delete_texture(cubemap_texture);
    }
}

unsafe fn create_sdl2_context() -> (
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
        .window("Hello triangle!", 1024, 768)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
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

unsafe fn create_vertex_buffer(gl: &glow::Context) -> (NativeBuffer, NativeVertexArray) {
    // This is a flat array of f32s that are to be interpreted as vec2s.
    let triangle_vertices = [0.5f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32];
    let triangle_vertices_u8: &[u8] = unsafe {
        core::slice::from_raw_parts(
            triangle_vertices.as_ptr() as *const u8,
            triangle_vertices.len() * core::mem::size_of::<f32>(),
        )
    };

    // We construct a buffer and upload the data
    unsafe {
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, triangle_vertices_u8, glow::STATIC_DRAW);

        // We now construct a vertex array to describe the format of the input buffer
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);

        (vbo, vao)
    }
}

unsafe fn set_uniform(gl: &glow::Context, program: NativeProgram, name: &str, value: f32) {
    unsafe {
        let uniform_location = gl.get_uniform_location(program, name);
        // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
        gl.uniform_1_f32(uniform_location.as_ref(), value)
    }
}

const VERTEX_SHADER_SOURCE: &str = r#"#version 330
  in vec2 in_position;
  out vec2 position;
  void main() {
    position = in_position;
    gl_Position = vec4(in_position - 0.5, 0.0, 3.0);
  }"#;
const FRAGMENT_SHADER_SOURCE: &str = r#"#version 330
  precision mediump float;
  in vec2 position;
  out vec4 color;
  uniform float blue;
  void main() {
    color = vec4(position, blue, 1.0);
  }"#;

const VERTICES: [f32; 24] = [
    -1.0, 1.0, -1.0, // Front top left
    -1.0, -1.0, -1.0, // Front bottom left
    1.0, -1.0, -1.0, // Front bottom right
    1.0, 1.0, -1.0, // Front top right
    -1.0, 1.0, 1.0, // Back top left
    -1.0, -1.0, 1.0, // Back bottom left
    1.0, -1.0, 1.0, // Back bottom right
    1.0, 1.0, 1.0, // Back top right
];

const INDICES: [u32; 36] = [
    0, 1, 2, 0, 2, 3, // Front face
    4, 5, 6, 4, 6, 7, // Back face
    4, 5, 1, 4, 1, 0, // Left face
    3, 2, 6, 3, 6, 7, // Right face
    4, 0, 3, 4, 3, 7, // Top face
    1, 5, 6, 1, 6, 2, // Bottom face
];
const IMAGES: [&str; 6] = [
    // right, left, top, bottom, front, back
    "panorama_1.png",
    "panorama_3.png",
    "panorama_4.png",
    "panorama_5.png",
    "panorama_0.png",
    "panorama_2.png",
];

fn load_cubemap(gl: &glow::Context, faces: &[&str]) -> NativeTexture {
    unsafe {
        let texture = gl.create_texture().expect_else_err();
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));
        for (i, &face) in faces.iter().enumerate() {
            let img = image::ImageReader::open(face)
                .expect(&format!("Failed to open '{}'", face))
                .decode()
                .unwrap();
            let img = img.to_rgb8();
            let (width, height) = img.dimensions();
            let data = img.into_raw();

            gl.tex_image_2d(
                glow::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                0,
                glow::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(&data)),
            );

            println!("Loaded cubemap face '{face}': {width}x{height}");
        }
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_WRAP_R,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.generate_mipmap(glow::TEXTURE_CUBE_MAP);
        texture
    }
}

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

trait ExpectErr<T> {
    fn expect_else_err(self) -> T;
}
impl<T, E: std::fmt::Display> ExpectErr<T> for Result<T, E> {
    fn expect_else_err(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }
}
