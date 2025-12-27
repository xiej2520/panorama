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
        println!("finished recording");
    }
}
