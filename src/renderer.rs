use std::sync::Arc;
use egui_wgpu::ScreenDescriptor;
use macaw::{Mat4, vec3};
use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, CommandEncoder, Device, Extent3d, ImageCopyTexture, ImageDataLayout, RenderPipeline, Texture, TextureFormat};
use wgpu::BindingResource::{Sampler, TextureView};
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::camera::{Camera, Projection};
use crate::{mesh, mesh_grid, simulation, texture};
use crate::egui_renderer::EguiRenderer;
use crate::sim_renderer::{RenderMode, SimRenderer};
use crate::simulation::DIVISIONS;

pub struct GfxState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,

    mesh_vertex_buffer: wgpu::Buffer,
    mesh_index_buffer: wgpu::Buffer,
    mesh_index_count: u32,
    instance_buffer: wgpu::Buffer,

    depth_texture: texture::Texture,

    pub camera: Camera,
    pub(crate) projection: Projection,

    camera_bind_group: BindGroup,
    projection_buffer: wgpu::Buffer,
    pub sim_texture_bind_group: BindGroup,
    pub sim_texture_size: Extent3d,
    pub sim_texture: Texture,
    pub pipeline_2d: Pipeline2D,
    // pub pipeline_prism: PipelinePrism,
    pub instance_count: u32,

    pub sim: SimRenderer,
    pub egui_renderer: EguiRenderer,
    pub(crate) scale_factor: f32,
}

impl<'a> GfxState<'a> {
    pub(crate) async fn new(window: Arc<Window>, fov_y: f32) -> GfxState<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::FLOAT32_FILTERABLE,
                required_limits: Default::default(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        ).await.unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let egui_renderer = EguiRenderer::new(&device, config.format, None, 1, &window);

        let sim_texture_size = wgpu::Extent3d {
            width: simulation::DIVISIONS,
            height: simulation::DIVISIONS,
            depth_or_array_layers: 1,
        };
        let sim_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: sim_texture_size,
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

        let sim_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let sim_texture_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &sim_texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: TextureView(&sim_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: Sampler(&sim_texture_sampler),
                    }
                ],
                label: Some("sim_texture_bind_group"),
            },
        );

        let projection = Projection::new(size.width, size.height, fov_y, 0.1, 1000.0);
        let camera = Camera::new(vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 0.0));
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Projection Matrix"),
                contents: bytemuck::cast_slice(&camera.calc_matrix().to_cols_array()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            layout: &camera_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        surface.configure(&device, &config);
        let cube = mesh::cube();
        let mesh_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&cube.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let mesh_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&cube.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let grid = mesh_grid::MeshGrid::square_grid(simulation::PRISM_SIZE as usize);
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("mesh instance buffer"),
                contents: bytemuck::cast_slice(&grid.instances),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let pipeline_2d = Pipeline2D::new(
            &device,
            config.format,
            &sim_texture_bind_group_layout,
        );

        // let pipeline_prism = PipelinePrism::new(
        //     &device,
        //     config.format,
        //     &camera_bind_group_layout,
        //     &sim_texture_bind_group_layout,
        // );

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            &config,
            "depth texture",
        );

        let sim = SimRenderer::new(&device, &config, &cube, &grid, DIVISIONS);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            mesh_vertex_buffer,
            mesh_index_buffer,
            mesh_index_count: cube.indices.len() as u32,
            instance_buffer,
            instance_count: grid.instances.len() as u32,

            camera,
            projection,
            projection_buffer: camera_buffer,
            camera_bind_group,

            sim_texture_size,
            sim_texture,
            sim_texture_bind_group,

            pipeline_2d,
            depth_texture,

            sim,
            egui_renderer,
            scale_factor: 1.0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.projection.resize(new_size.width, new_size.height);

            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth texture",
            );
        }
    }

    pub fn update_sim_texture(&mut self, data: &[u8]) {
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.sim_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * DIVISIONS),
                rows_per_image: Some(DIVISIONS),
            },
            self.sim_texture_size,
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let matrix = self.projection.calc_matrix() * self.camera.calc_matrix();
        self.queue.write_buffer(&self.projection_buffer, 0, bytemuck::cast_slice(&matrix.to_cols_array()));

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );
        self.sim.render(&view, &mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.render_egui(&view);

        output.present();
        Ok(())
    }

    fn render_egui(&mut self, surface_view: &wgpu::TextureView) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: self.window.scale_factor() as f32 * self.scale_factor,
        };
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None }
        );

        let window = self.window.as_ref();

        {
            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        // surface_texture.present();
    }
}

struct Pipeline2D {
    pipeline: RenderPipeline,
}

impl Pipeline2D {
    fn new(
        device: &Device,
        surface_format: TextureFormat,
        sim_texture_layout: &BindGroupLayout,
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
                    format: surface_format,
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
        encoder: &mut CommandEncoder,
        sim_texture_bind_group: &BindGroup,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
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
