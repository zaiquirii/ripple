use std::sync::Arc;
use egui_wgpu::ScreenDescriptor;
use macaw::vec3;
use winit::window::Window;
use crate::camera::{Camera, Projection};
use crate::{mesh, mesh_grid, simulation};
use crate::egui_renderer::EguiRenderer;
use crate::sim_renderer::SimRenderer;
use crate::simulation::DIVISIONS;

pub struct GfxState<'a> {
    surface: wgpu::Surface<'a>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,

    pub(crate) projection: Projection,

    pub sim: SimRenderer,
    pub egui_renderer: EguiRenderer,
}

impl<'a> GfxState<'a> {
    pub(crate) async fn new(
        window: Arc<Window>,
        fov_y: f32,
        prism: &mesh::Mesh,
        grid: &mesh_grid::MeshGrid,
    ) -> GfxState<'a> {
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
        let projection = Projection::new(size.width, size.height, fov_y, 0.1, 10000.0);

        surface.configure(&device, &config);
        let sim = SimRenderer::new(&device, &config, &prism, &grid, DIVISIONS);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,

            projection,

            sim,
            egui_renderer,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.projection.resize(new_size.width, new_size.height);
            self.sim.resize(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );
        self.sim.render(&view, &mut encoder);
        self.render_egui(&view, &mut encoder);
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }

    fn render_egui(
        &mut self,
        surface_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        let window = self.window.as_ref();
        self.egui_renderer.end_frame_and_draw(
            &self.device,
            &self.queue,
            encoder,
            window,
            &surface_view,
            screen_descriptor,
        );
    }
}