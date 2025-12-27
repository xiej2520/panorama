use std::{fs::File, io::BufWriter};

use image::{GenericImage, ImageBuffer, Rgb};

fn main() {
    let right = load_rgb8("panorama_3.png");
    let left = load_rgb8("panorama_1.png");
    let top = load_rgb8("panorama_4.png");
    let bottom = load_rgb8("panorama_5.png");
    let back = load_rgb8("panorama_0.png");
    let front = load_rgb8("panorama_2.png");

    let (width, height) = front.dimensions();
    let mut buf = ImageBuffer::new(width * 4, height * 3);

    #[allow(clippy::identity_op)]
    #[allow(clippy::erasing_op)]
    {
        buf.copy_from(&left, 1 * width, height)
            .expect("to copy left");
        buf.copy_from(&front, 2 * width, height)
            .expect("to copy front");
        buf.copy_from(&right, 3 * width, height)
            .expect("to copy right");
        buf.copy_from(&back, 0 * width, height)
            .expect("to copy back");
        buf.copy_from(&top, 0 * width, 0).expect("to copy top");
        buf.copy_from(&bottom, 0 * width, 2 * height)
            .expect("to copy bottom");
    }

    let out = File::create("skybox_out.png").expect("to open skybox_out.png");
    buf.write_to(&mut BufWriter::new(out), image::ImageFormat::Png)
        .expect("to write out file");
}

fn load_rgb8(path: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    image::ImageReader::open(path)
        .unwrap_or_else(|e| panic!("expected {path} to be opened: {e}"))
        .decode()
        .expect("to decode image")
        .to_rgb8()
}
