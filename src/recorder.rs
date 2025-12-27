pub struct VideoWriter {
    ffmpeg: std::process::Child,
    stdin: std::process::ChildStdin,
    width: i32,
    height: i32,
    buf: Box<[u8]>, // 3 * width * height bytes, RGB data
}

impl VideoWriter {
    pub fn new(width: u32, height: u32, output: &str, fps: u32) -> Self {
        // width and height must be divisible by 2 for yuv420p, truncate
        let (width, height) = (width as i32 / 2 * 2, height as i32 / 2 * 2);

        #[rustfmt::skip]
        let mut ffmpeg = std::process::Command::new("ffmpeg")
            .args([
                "-y",                     // overwrite output
                // input
                "-f", "rawvideo",
                "-pix_fmt", "rgb24",
                "-s", &format!("{}x{}", width, height),
                "-r", &fps.to_string(),   // *input* frame rate, 1 image = 1 frame
                "-i", "-",                // stdin
                // output
                "-c:v", "libx264",
                "-preset", "slow",
                "-crf", "20",
                "-pix_fmt", "yuv420p",    // required for mp4 compatibility
                "-vf", "vflip",           // flip vertically (OpenGL framebuffer)
                "-r", &fps.to_string(),   // *output* frame rate
                output,
            ])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg");

        let stdin = ffmpeg.stdin.take().expect("Failed to open stdin");
        // SAFETY: zero is valid u8 and RGB
        let buf = unsafe { Box::new_uninit_slice((width * height * 3) as usize).assume_init() };

        Self {
            ffmpeg,
            stdin,
            width,
            height,
            buf,
        }
    }

    pub fn write_frame(&mut self, gl: &glow::Context) {
        use glow::{HasContext, PixelPackData};

        unsafe {
            gl.read_pixels(
                0,
                0,
                self.width,
                self.height,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                PixelPackData::Slice(Some(&mut self.buf)),
            )
        };

        std::io::Write::write_all(&mut self.stdin, &self.buf).expect("Failed to write frame");
    }

    pub fn finish(mut self) {
        drop(self.stdin); // VERY IMPORTANT: signals EOF to ffmpeg
        self.ffmpeg.wait().expect("ffmpeg failed");
        println!("finished recording");
    }
}
