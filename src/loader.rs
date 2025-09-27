use std::path::{Path, PathBuf};

use glow::{HasContext, NativeTexture, PixelUnpackData};

use crate::ExpectErr;

pub fn create_faces(root: &Path) -> Faces {
    Faces {
        right: root.join("panorama_1.png"),
        left: root.join("panorama_3.png"),
        top: root.join("panorama_4.png"),
        bottom: root.join("panorama_5.png"),
        front: root.join("panorama_0.png"),
        back: root.join("panorama_2.png"),
    }
}

pub struct Faces {
    right: PathBuf,
    left: PathBuf,
    top: PathBuf,
    bottom: PathBuf,
    front: PathBuf,
    back: PathBuf,
}

struct FaceIter<'a> {
    faces: &'a Faces,
    index: u8,
}

impl<'a> Iterator for FaceIter<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(self.faces.right.as_path()),
            1 => Some(self.faces.left.as_path()),
            2 => Some(self.faces.top.as_path()),
            3 => Some(self.faces.bottom.as_path()),
            4 => Some(self.faces.front.as_path()),
            5 => Some(self.faces.back.as_path()),
            _ => None,
        };
        self.index += 1;
        result
    }
}

impl Faces {
    fn iter(&self) -> FaceIter<'_> {
        FaceIter {
            faces: self,
            index: 0,
        }
    }
}

pub fn load_cubemap(gl: &glow::Context, faces: Faces) -> NativeTexture {
    unsafe {
        let texture = gl.create_texture().expect_else_err();
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));
        for (i, face) in faces.iter().enumerate() {
            let img = image::ImageReader::open(face)
                .unwrap_or_else(|_| panic!("Failed to open '{}'", face.display()))
                .decode()
                .unwrap_or_else(|_| panic!("Failed to decode image '{}'", face.display()));
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

            println!("Loaded cubemap face '{}': {width}x{height}", face.display());
        }
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
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
