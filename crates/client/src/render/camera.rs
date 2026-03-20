//! 3D Camera system matching RuneScape's angle conventions.
//!
//! RS uses 2048-unit angles (0-2047) and 128-unit tile coordinates.
//! Pitch: 128 (level) to 383 (looking down)
//! Yaw: 0-2047 wrapping, clockwise from north

use glam::{Mat4, Vec3};

/// RS-style camera.
pub struct Camera3D {
    pub x: f32,
    pub y: f32,  // height
    pub z: f32,
    pub pitch: f32,  // RS units (128-383)
    pub yaw: f32,    // RS units (0-2047)
    pub zoom: f32,
}

impl Camera3D {
    pub fn new() -> Self {
        Camera3D {
            x: 3200.0,   // center of a 50-tile region
            y: 400.0,    // above ground
            z: 3200.0,
            pitch: 200.0,
            yaw: 0.0,
            zoom: 600.0,
        }
    }

    /// Convert RS angles to radians.
    fn pitch_rad(&self) -> f32 {
        (self.pitch / 2048.0) * std::f32::consts::TAU
    }

    fn yaw_rad(&self) -> f32 {
        (self.yaw / 2048.0) * std::f32::consts::TAU
    }

    /// Build the view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        let pitch = self.pitch_rad();
        let yaw = self.yaw_rad();

        // Camera offset from focal point
        let dist = self.zoom;
        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        let eye = Vec3::new(
            self.x - sin_yaw * cos_pitch * dist,
            self.y + sin_pitch * dist,
            self.z - cos_yaw * cos_pitch * dist,
        );

        let target = Vec3::new(self.x, self.y - 100.0, self.z);

        Mat4::look_at_rh(eye, target, Vec3::Y)
    }

    /// Build the perspective projection matrix.
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_4, // 45° FOV
            aspect,
            1.0,      // near
            8000.0,   // far (50 tiles * 128 + margin)
        )
    }

    /// Combined view-projection matrix.
    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }

    /// Camera position for fog calculations.
    pub fn position(&self) -> Vec3 {
        let pitch = self.pitch_rad();
        let yaw = self.yaw_rad();
        let dist = self.zoom;

        Vec3::new(
            self.x - yaw.sin() * pitch.cos() * dist,
            self.y + pitch.sin() * dist,
            self.z - yaw.cos() * pitch.cos() * dist,
        )
    }

    /// Rotate camera with mouse drag.
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw = (self.yaw + dx * 3.0) % 2048.0;
        if self.yaw < 0.0 { self.yaw += 2048.0; }
        self.pitch = (self.pitch + dy * 2.0).clamp(128.0, 383.0);
    }

    /// Move camera with arrow keys.
    pub fn translate(&mut self, forward: f32, right: f32) {
        let yaw = self.yaw_rad();
        self.x += yaw.sin() * forward + yaw.cos() * right;
        self.z += yaw.cos() * forward - yaw.sin() * right;
    }
}
