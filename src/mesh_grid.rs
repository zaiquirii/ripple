use bytemuck::{Pod, Zeroable};
use macaw::{UVec2, uvec2, Vec2, vec2};
use wgpu::util::DeviceExt;
use crate::simulation;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct Instance {
    pub position: Vec2,
    pub uv: UVec2,
}

impl Instance {
    const VERTEX_ATTRIB: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Uint32x2,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::VERTEX_ATTRIB,
        }
    }
}

pub struct MeshGrid {
    pub instances: Vec<Instance>,
}

impl MeshGrid {
    pub fn square_grid(size: usize) -> Self {
        let pos_step = simulation::PRISM_STEP;
        let uv_step = 1.0 / (size as f32);
        let mut instances = Vec::new();
        for y in 0..size {
            for x in 0..size {
                instances.push(Instance {
                    position: vec2(x as f32, y as f32) * pos_step,
                    uv: uvec2(
                        (x as f32 * uv_step * simulation::DIVISIONS as f32) as u32,
                        (y as f32 * uv_step * simulation::DIVISIONS as f32) as u32,
                    ),
                })
            }
        }
        Self {
            instances
        }
    }

    pub fn push_to_device(&self, device: &wgpu::Device) -> UploadedMeshGrid {
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("mesh instance buffer"),
                contents: bytemuck::cast_slice(&self.instances),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        UploadedMeshGrid {
            instance_buffer,
            instance_count: self.instances.len() as u32,
        }
    }
}

pub struct UploadedMeshGrid {
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}