use macaw::Mat4;
use wgpu::util::DeviceExt;
use crate::mesh::{Mesh, UploadedMesh};
use crate::mesh_grid::{MeshGrid, UploadedMeshGrid};
use crate::{mesh, mesh_grid, texture};

pub enum RenderMode {
    Texture,
    Prism,
}

pub struct SimRenderer {
    prism: UploadedMesh,
    grid: UploadedMeshGrid,
    sim_data: SimTextureData,
    pipeline_prism: PipelinePrism,
    pipeline_2d: Pipeline2D,
    pub(crate) render_mode: RenderMode,
}

impl SimRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        prism: &Mesh,
        grid: &MeshGrid,
        sim_divisions: u32,
    ) -> Self {
        let prism = prism.push_to_device(device);
        let grid = grid.push_to_device(device);
        let sim_data = SimTextureData::new(device, sim_divisions);

        let pipeline_prism = PipelinePrism::new(
            device,
            surface_config,
            &sim_data.bind_group_layout,
        );

        let pipeline_2d = Pipeline2D::new(
            device,
            surface_config,
            &sim_data.bind_group_layout,
        );

        Self {
            prism,
            grid,
            sim_data,
            pipeline_prism,
            pipeline_2d,
            render_mode: RenderMode::Prism,
        }
    }

    pub fn set_camera_transform(&self, queue: &wgpu::Queue, transform: Mat4) {
        queue.write_buffer(
            &self.pipeline_prism.camera_buffer,
            0,
            bytemuck::cast_slice(&transform.to_cols_array()))
    }

    pub fn update_sim_data(&self, queue: &wgpu::Queue, divisions: u32, data: &[u8]) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.sim_data.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * divisions),
                rows_per_image: Some(divisions),
            },
            self.sim_data.texture_size,
        );
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        match self.render_mode {
            RenderMode::Texture => {
                self.pipeline_2d.render(
                    view,
                    encoder,
                    &self.sim_data.bind_group,
                )
            }
            RenderMode::Prism => {
                self.pipeline_prism.render(
                    view,
                    encoder,
                    &self.sim_data.bind_group,
                    &self.prism,
                    &self.grid,
                );
            }
        }
    }
}

struct SimTextureData {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    texture_size: wgpu::Extent3d,
    texture: wgpu::Texture,
}

impl SimTextureData {
    pub fn new(device: &wgpu::Device, divisions: u32) -> Self {
        let texture_size = wgpu::Extent3d {
            width: divisions,
            height: divisions,
            depth_or_array_layers: 1,
        };
        let sim_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("sim_texture"),
                view_formats: &[],
            }
        );
        let sim_texture_view = sim_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sim_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("sim_texture_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&sim_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sim_texture_sampler),
                    }
                ],
                label: Some("sim_texture_bind_group"),
            },
        );
        Self {
            bind_group_layout,
            bind_group,
            texture_size,
            texture: sim_texture,
        }
    }
}

struct PipelinePrism {
    pipeline: wgpu::RenderPipeline,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: texture::Texture,
}

impl PipelinePrism {
    fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        sim_texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/pipeline_prism.wgsl"));
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Projection Matrix"),
                contents: bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prism render pipeline layout"),
            bind_group_layouts: &[
                &camera_layout,
                sim_texture_layout,
            ],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[mesh::vertex_desc(), mesh_grid::Instance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let depth_texture = texture::Texture::create_depth_texture(
            &device, &surface_config, "sim_renderer depth texture",
        );

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            depth_texture,
        }
    }

    fn render(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        sim_texture_group: &wgpu::BindGroup,
        prism: &UploadedMesh,
        grid: &UploadedMeshGrid,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, sim_texture_group, &[]);
        render_pass.set_vertex_buffer(0, prism.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, grid.instance_buffer.slice(..));
        render_pass.set_index_buffer(prism.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..prism.index_count, 0, 0..grid.instance_count);
    }
}

struct Pipeline2D {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline2D {
    fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        sim_texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/pipeline_2d.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline 2D Layout"),
            bind_group_layouts: &[
                sim_texture_layout,
            ],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("2D Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline
        }
    }

    fn render(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        sim_texture_bind_group: &wgpu::BindGroup,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("2d render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, sim_texture_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
