use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use glow::{HasContext, NativeTexture, PixelUnpackData};
use image::{DynamicImage, GenericImageView, ImageReader};
use zip::ZipArchive;

use crate::ExpectErr;

pub struct PanoramaPath {
    root: PathBuf,
    path_type: PanoramaPathType,
}
pub enum PanoramaPathType {
    BaseFolder,
    PackFolder,
    PackZip,
}

const PATH_IN_PACK: &str = "assets/minecraft/textures/gui/title/background";

impl PanoramaPath {
    pub fn find_paths(root: &Path) -> Result<Self, Box<str>> {
        let path_type = if root.is_file() && root.extension().is_some_and(|s| s == "zip") {
            let reader = BufReader::new(
                File::open(root)
                    .map_err(|e| format!("failed to open file {}: {e:?}", root.display()))?,
            );
            let mut zip = zip::ZipArchive::new(reader)
                .map_err(|e| format!("failed to read zip file: {e}"))?;

            for i in 0..=5 {
                let file_path = format!("{PATH_IN_PACK}/panorama_{i}.png");
                zip.by_name(&file_path)
                    .map_err(|e| format!("failed to find {file_path} in zip file: {e:?}"))?;
            }
            PanoramaPathType::PackZip
        }
        // check base directory, and resource pack panorama path
        else if (0..=5).all(|i| {
            root.join(format!("{PATH_IN_PACK}/panorama_{i}.png"))
                .exists()
        }) {
            PanoramaPathType::PackFolder
        } else if (0..=5).all(|i| root.join(format!("panorama_{i}.png")).exists()) {
            PanoramaPathType::BaseFolder
        } else {
            return Err(format!(
                "no panorama images found in {} or {}/{PATH_IN_PACK}",
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

    pub fn iter(&self) -> PanoramaImageIter<'_> {
        PanoramaImageIter {
            path: self,
            index: 0,
        }
    }
}

pub struct PanoramaImageIter<'a> {
    path: &'a PanoramaPath,
    index: u8,
}

/// eg 0th texture is right face of cube, load panorama_3.png
fn map_texture_number_to_file_number(texture_number: u8) -> Option<i32> {
    Some(match texture_number {
        0 => 3, // right  // +x
        1 => 1, // left   // -x
        2 => 4, // top    // +y
        3 => 5, // bottom // -y
        4 => 0, // back   // +z
        5 => 2, // front  // -z
        _ => return None,
    })
}

impl<'a> PanoramaImageIter<'a> {
    // try block
    fn load_image(&mut self, image_number: i32) -> Result<DynamicImage, Box<str>> {
        let decode = match self.path.path_type {
            PanoramaPathType::BaseFolder => {
                let path = format!("panorama_{image_number}.png");
                ImageReader::open(self.path.root.join(&path))
                    .map_err(|e| format!("failed to open image {path}: {e}").into_boxed_str())?
                    .decode()
            }
            PanoramaPathType::PackFolder => {
                let path = format!("{PATH_IN_PACK}/panorama_{image_number}.png");
                ImageReader::open(self.path.root.join(&path))
                    .map_err(|e| format!("failed to open image {path}: {e}").into_boxed_str())?
                    .decode()
            }
            PanoramaPathType::PackZip => {
                let reader =
                    BufReader::new(File::open(&self.path.root).map_err(|e| e.to_string())?);
                let mut zip = ZipArchive::new(reader).map_err(|e| e.to_string())?;
                let mut zip_file = zip
                    .by_name(&format!("{PATH_IN_PACK}/panorama_{image_number}.png"))
                    .map_err(|e| e.to_string())?;
                use std::io::{Cursor, Read};
                let mut buf = vec![];
                zip_file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                let mut img = ImageReader::new(Cursor::new(&buf));
                img.set_format(image::ImageFormat::Png);
                img.decode()
            }
        };
        decode.map_err(|e| format!("failed to decode image: {e}").into_boxed_str())
    }
}

impl<'a> Iterator for PanoramaImageIter<'a> {
    type Item = Result<DynamicImage, Box<str>>;

    fn next(&mut self) -> Option<Self::Item> {
        let image_number = map_texture_number_to_file_number(self.index)?;
        let img = self.load_image(image_number);
        self.index += 1;
        Some(img)
    }
}

pub fn load_cubemap(gl: &glow::Context, path: &Path) -> NativeTexture {
    let paths = PanoramaPath::find_paths(path)
        .unwrap_or_else(|e| panic!("expected to find panorama_<0 to 5>.png: {e}"));
    unsafe {
        let texture = gl.create_texture().expect_else_err();
        gl.bind_texture(glow::TEXTURE_CUBE_MAP, Some(texture));

        for (i, img_result) in paths.iter().enumerate() {
            let file_number = map_texture_number_to_file_number(i as u8).unwrap();
            let (width, height, data) = match img_result {
                Ok(img) => {
                    let (width, height) = img.dimensions();
                    (width, height, img.fliph().to_rgb8().into_raw())
                }
                Err(e) => {
                    eprintln!("failed to load face panorama_{file_number}: {e}",);
                    let (width, height) = (1024, 1024);
                    let black = vec![0u8; width * height * 3];
                    (width as u32, height as u32, black)
                }
            };

            gl.tex_image_2d(
                glow::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                0,
                glow::RGB8 as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(&data)),
            );

            println!("Loaded cubemap face 'panorama_{file_number}': {width}x{height}",);
        }
        gl.tex_parameter_i32(
            glow::TEXTURE_CUBE_MAP,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
            //glow::LINEAR_MIPMAP_LINEAR as i32,
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
        // why mipmap set distance cube map
        //gl.generate_mipmap(glow::TEXTURE_CUBE_MAP);
        texture
    }
}
