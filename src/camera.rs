use macaw::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::look_at_lh(self.position, self.target, Vec3::Y)
    }
}

pub struct Projection {
    aspect_ratio: f32,
    fov_y: f32, // In radians
    z_near: f32,
    z_far: f32,
}

impl Projection {
    pub fn new(
        width: u32,
        height: u32,
        fov_y: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect_ratio: width as f32 / height as f32,
            fov_y,
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far)
    }
}