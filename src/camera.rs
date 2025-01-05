use macaw::{Mat4, Plane3, Vec2, Vec3, vec4, Vec4Swizzles};

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
    fov_y: f32,
    /// In Radians
    z_near: f32,
    z_far: f32,
}

impl Projection {
    /// fov_y in radians
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
        let height = 25.0;
        // Mat4::orthographic_lh(-height, height, -height * self.aspect_ratio, height * self.aspect_ratio, self.z_near, self.z_far)
        Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far)
    }
}

// Do no understand this yet, taken from https://stettj.com/projecting-screen-coordinates-onto-a-3d-plane

fn homogenous_to_world(point: Vec3, proj: Mat4, view: Mat4) -> Vec3 {
    let transform = (proj * view).inverse();
    let _world = transform * point.extend(1.0);
    _world.xyz() * (1.0 / _world.w)
}

pub fn project_screen_onto_plane(screen: Vec2, plane: Plane3, proj: Mat4, view: Mat4) -> Option<Vec3> {
    let origin = homogenous_to_world(screen.extend(0.0), proj, view);
    let end = homogenous_to_world(screen.extend(1.0), proj, view);
    let ray_vector = (end - origin).normalize();
    let (intersects, t) = plane.intersect_ray(origin, ray_vector);
    if intersects {
        Some(origin + t * ray_vector)
    } else {
        None
    }
}