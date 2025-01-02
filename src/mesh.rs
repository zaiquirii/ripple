use macaw::{Vec3, vec3};
use wgpu::util::DeviceExt;

#[derive(Default, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl Mesh {
    // points need to be defined CCW
    pub fn push_quad(&mut self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3) {
        let start_index = self.vertices.len() as u32;
        self.vertices.push(p0);
        self.vertices.push(p1);
        self.vertices.push(p2);
        self.vertices.push(p3);

        let indices: [u32; 6] = [
            0, 1, 3,
            1, 2, 3
        ];
        for i in indices {
            self.indices.push(i + start_index)
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

    // front back (Z swaps)
    mesh.push_quad(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), vec3(0.0, height, 0.0));
    mesh.push_quad(vec3(0.0, 0.0, 1.0), vec3(0.0, height, 1.0), vec3(1.0, height, 1.0), vec3(1.0, 0.0, 1.0));

    // left right (X swaps)
    mesh.push_quad(vec3(0.0, 0.0, 0.0), vec3(0.0, height, 0.0), vec3(0.0, height, 1.0), vec3(0.0, 0.0, 1.0));
    mesh.push_quad(vec3(1.0, 0.0, 0.0), vec3(1.0, 0.0, 1.0), vec3(1.0, height, 1.0), vec3(1.0, height, 0.0));

    // top bottom (Y swaps)
    mesh.push_quad(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0));
    mesh.push_quad(vec3(0.0, height, 0.0), vec3(1.0, height, 0.0), vec3(1.0, height, 1.0), vec3(0.0, height, 1.0));

    mesh
}

const VERTEX_ATTRIB: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
    0 => Float32x3,
];

pub fn vertex_desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRIB,
    }
}