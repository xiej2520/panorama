use glam::{Mat4, Vec3, vec3};

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32, // "units" per second in world space
    pub sensitivity: f32,
    pub fov: f32, // degrees

    pub rotation_speed: f32, // degrees per s

    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, up: Vec3, pitch: f32, yaw: f32) -> Self {
        let mut res = Camera {
            yaw,
            pitch,
            speed: 2.5,
            sensitivity: 0.1,
            fov: 45.0,
            rotation_speed: 0.0,
            position,
            forward: Vec3::ZERO,
            up,
            right: Vec3::ZERO,
            world_up: up,
        };
        res.update_vectors();
        res
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward, self.up)
    }

    pub fn process_keyboard_movement(&mut self, direction: MoveDirection, delta_time: f32) {
        let dist = self.speed * delta_time;
        self.position += dist
            * match direction {
                MoveDirection::Forward => self.forward,
                MoveDirection::Backward => -self.forward,
                MoveDirection::Left => -self.right,
                MoveDirection::Right => self.right,
                MoveDirection::Up => self.up,
                MoveDirection::Down => -self.up,
            }
    }

    pub fn process_mouse_movement(&mut self, x_offset: f32, y_offset: f32) {
        self.yaw += self.sensitivity * x_offset;
        self.pitch -= self.sensitivity * y_offset;
        self.pitch = self.pitch.clamp(-89.9, 89.9);

        self.update_vectors();
    }

    pub fn process_mouse_scroll(&mut self, y_offset: f32) {
        self.fov = f32::clamp(self.fov - y_offset, 0.1, 179.9);
    }

    pub fn process_rotation(&mut self, delta_time_us: u128) {
        let delta = self.rotation_speed * (delta_time_us as f32 / 1_000_000.0);
        self.yaw += delta;
        self.update_vectors();
    }

    /// update camera basis vectors based on current yaw and pitch
    pub fn update_vectors(&mut self) {
        self.forward = vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        )
        .normalize();
        self.right = self.forward.cross(self.world_up).normalize();
        self.up = self.right.cross(self.forward).normalize();
    }

    pub fn reset_panorama_options(&mut self) {
        self.yaw = 0.0;
        self.pitch = 0.0;
        // minecraft uses 85.0 fov for panorama in 1.21.11
        self.fov = 85.0;
        // minecraft panorama rotates once every 90 seconds at default speed
        let period = 90.0;
        self.rotation_speed = 360.0 / period;
        self.update_vectors();
    }
}

pub enum MoveDirection {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}
