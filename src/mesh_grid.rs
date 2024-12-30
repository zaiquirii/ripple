use bytemuck::{Pod, Zeroable};
use macaw::{Vec2, vec2};
use wgpu::{VertexAttribute, VertexStepMode};
use crate::simulation;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct Instance {
    pub position: Vec2,
    pub uv: Vec2,
}

impl Instance {
    const VERTEX_ATTRIB: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
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
                    uv: vec2(x as f32, y as f32) * uv_step,
                })
            }
        }
        Self {
            instances
        }
    }
}