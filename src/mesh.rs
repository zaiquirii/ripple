use bytemuck::{Pod, Zeroable};
use macaw::{Vec2, vec2, Vec3, vec3};
use wgpu::TextureAspect::Plane2;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
}

fn vertex(position: Vec3, normal: Vec3) -> Vertex {
    Vertex {
        position,
        normal,
    }
}

#[derive(Default, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    // points need to be defined CCW
    pub fn push_quad(&mut self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3) {
        let normal = (p2 - p0).cross(p1 - p0).normalize();
        let start_index = self.vertices.len() as u32;
        self.vertices.push(vertex(p0, normal));
        self.vertices.push(vertex(p1, normal));
        self.vertices.push(vertex(p2, normal));
        self.vertices.push(vertex(p3, normal));

        let indices: [u32; 6] = [
            0, 1, 3,
            1, 2, 3
        ];
        for i in indices {
            self.indices.push(i + start_index)
        }
    }

    pub fn push_vert_quad(&mut self, start: Vec2, end: Vec2, height: f32) {
        self.push_quad(
            vec3(start.x, 0.0, start.y),
            vec3(start.x, -height, start.y),
            vec3(end.x, -height, end.y),
            vec3(end.x, 0.0, end.y),
        )
    }

    pub fn push_vert_walls(&mut self, points: &[Vec2], height: f32) {
        for window in points.windows(2) {
            self.push_vert_quad(window[0], window[1], height);
        }
        self.push_vert_quad(*points.last().unwrap(), points[0], height);
    }

    pub fn push_tri(&mut self, p0: Vec3, p1: Vec3, p2: Vec3) {
        let normal = (p2 - p0).cross(p1 - p0).normalize();
        let start_index = self.vertices.len() as u32;
        self.vertices.push(vertex(p0, normal));
        self.vertices.push(vertex(p1, normal));
        self.vertices.push(vertex(p2, normal));

        self.indices.push(start_index);
        self.indices.push(start_index + 1);
        self.indices.push(start_index + 2);
    }

    pub fn push_polygon(&mut self, points: &[Vec2]) {
        let start = points[0];
        for window in points[1..].windows(2) {
            self.push_tri(
                vec3(start.x, 0.0, start.y),
                vec3(window[0].x, 0.0, window[0].y),
                vec3(window[1].x, 0.0, window[1].y),
            )
        }
    }

    pub fn push_to_device(&self, device: &wgpu::Device) -> UploadedMesh {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        UploadedMesh {
            vertex_buffer,
            vertex_count: self.vertices.len() as u32,
            index_buffer,
            index_count: self.indices.len() as u32,
        }
    }
}

pub struct UploadedMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub fn square_prism(height: f32) -> Mesh {
    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };

    // Walls
    let points = vec![
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 1.0),
    ];
    mesh.push_vert_walls(&points, height);
    mesh.push_polygon(&points);

    mesh
}

pub fn hex_prism(height: f32) -> Mesh {
    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };

    // Pointy top hexagons
    let w_2 = 3.0_f32.sqrt() * 0.5 * 0.5;
    let h_2 = 0.5;

    // Walls
    let points = vec![
        vec2(0.0, h_2),
        vec2(-w_2, h_2 / 2.0),
        vec2(-w_2, h_2 / -2.0),
        vec2(0.0, -h_2),
        vec2(w_2, h_2 / -2.0),
        vec2(w_2, h_2 / 2.0),
    ];
    mesh.push_vert_walls(&points, height);
    mesh.push_polygon(&points);
    mesh
}

const VERTEX_ATTRIB: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
    0 => Float32x3,
    3 => Float32x3,
];

pub fn vertex_desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRIB,
    }
}