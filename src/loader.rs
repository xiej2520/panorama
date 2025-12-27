use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use glow::{HasContext, NativeTexture, PixelUnpackData};
use image::{DynamicImage, ImageReader};
use zip::ZipArchive;

use crate::ExpectErr;

pub struct PanoramaPath {
    root: PathBuf,
    path_type: PanoramaPathType,
}
pub enum PanoramaPathType {
    BaseFolder,
    PathInPack,
    ZipPack,
}

const PATH_IN_PACK: &str = "assets/minecraft/textures/gui/title/background";

// right: root.join("panorama_3.png"),  // +x
// left: root.join("panorama_1.png"),   // -x
// top: root.join("panorama_4.png"),    // +y
// bottom: root.join("panorama_5.png"), // -y
// back: root.join("panorama_0.png"),   // +z
// front: root.join("panorama_2.png"),  // -z

impl PanoramaPath {
    pub fn find_paths(root: &Path) -> Result<Self, Box<str>> {
        let path_type = if root.is_file() && root.extension().is_some_and(|s| s == "zip") {
            let reader = BufReader::new(
                File::open(root)
                    .map_err(|e| format!("failed to open file {}: {e:?}", root.display()))?,
            );
            let mut zip = zip::ZipArchive::new(reader)
                .map_err(|e| format!("failed to open zip file: {e}"))?;

            for i in 0..=5 {
                zip.by_name(&format!("{PATH_IN_PACK}/panorama_{i}.png"))
                    .map_err(|e| {
                        format!("failed to find {PATH_IN_PACK}/panorama{i}.png in zip file: {e:?}")
                    })?;
            }
            PanoramaPathType::ZipPack
        }
        // check base directory, and resource pack panorama path
        else if (0..=5).all(|i| {
            root.join(format!("{PATH_IN_PACK}/panorama_{i}.png"))
                .exists()
        }) {
            PanoramaPathType::PathInPack
        } else if (0..=5).all(|i| root.join(format!("panorama_{i}.png")).exists()) {
            PanoramaPathType::BaseFolder
        } else {
            return Err(format!(
                "did not find panorama files in {} or {}/{PATH_IN_PACK}",
                root.display(),
                root.display(),
            )
            .into_boxed_str());
        };
        Ok(Self {
            root: root.to_path_buf(),
            path_type,
        })
    }

    pub fn iter(&self) -> FaceIter<'_> {
        FaceIter {
            faces: self,
            index: 0,
        }
    }
}

pub struct FaceIter<'a> {
    faces: &'a PanoramaPath,
    index: u8,
}

fn map_texture_number_to_file_number(texture_number: u8) -> Option<i32> {
    Some(match texture_number {
        0 => 3, // right  // +x
        1 => 1, // left   // -x
        2 => 4, // top    // +y
        3 => 5, // bottom // -y
        4 => 0, // back   // +z
        5 => 2, // front  // -z
        _ => None?,
    })
}

impl<'a> Iterator for FaceIter<'a> {
    type Item = DynamicImage;

    fn next(&mut self) -> Option<Self::Item> {
        let image_number = map_texture_number_to_file_number(self.index)?;
        let result = match self.faces.path_type {
            PanoramaPathType::BaseFolder => {
                ImageReader::open(self.faces.root.join(format!("panorama_{image_number}.png")))
                    .ok()?
                    .decode()
                    .ok()?
            }
            PanoramaPathType::PathInPack => ImageReader::open(
                self.faces
                    .root
                    .join(format!("{PATH_IN_PACK}/panorama_{image_number}.png")),
            )
            .ok()?
            .decode()
            .ok()?,
            PanoramaPathType::ZipPack => {
                let reader = BufReader::new(File::open(&self.faces.root).ok()?);
                let mut zip = ZipArchive::new(reader).ok()?;
                let mut zip_file = zip
                    .by_name(&format!("{PATH_IN_PACK}/panorama_{image_number}.png"))
                    .ok()?;
                use std::io::{Cursor, Read};
                let mut buf = vec![];
                zip_file.read_to_end(&mut buf).ok()?;
                let mut img = ImageReader::new(Cursor::new(&buf));
                img.set_format(image::ImageFormat::Png);
                img.decode().ok()?
            }
        };
        self.index += 1;
        Some(result)
    }
}

pub fn load_cubemap(gl: &glow::Context, path: &Path) -> NativeTexture {
    let faces =
        PanoramaPath::find_paths(&path).expect(&format!("expected to find panorama_<0 to 5>.png",));
    unsafe {
        let texture = gl.create_texture().expect_else_err();
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));
        for (i, face) in faces.iter().enumerate() {
            let img = face.fliph();
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

            println!(
                "Loaded cubemap face '{}': {width}x{height}",
                map_texture_number_to_file_number(i as u8).unwrap()
            );
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
