mod renderer;
mod mesh;
mod camera;
mod simulation;
mod mesh_grid;
mod texture;
mod egui_renderer;
mod sim_renderer;

use std::sync::Arc;
use egui::Widget;
use log::info;
use macaw::{Plane3, Vec2, vec2, vec3, Vec3Swizzles};
use winit::application::ApplicationHandler;
use winit::event::ElementState::Pressed;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::camera::Camera;
use crate::renderer::{GfxState};
use crate::sim_renderer::RenderMode;
use crate::simulation::WaveSimulation;

fn main() {
    run();
}

#[derive(Copy, Clone, PartialEq)]
enum PrismType {
    Square,
    Hex,
}

#[derive(Copy, Clone, PartialEq)]
struct RenderConfig {
    prism_type: PrismType,
    prism_height: f32,
    grid_size: usize,
    step_size: f32,
}

#[derive(Copy, Clone)]
struct CameraConfig {
    rotation_enabled: bool,
    rotation_speed: f32,
}

impl RenderConfig {
    pub fn differs(&self, other: RenderConfig) -> bool {
        self.prism_height != other.prism_height ||
            self.grid_size != other.grid_size ||
            self.step_size != other.step_size
    }
}

struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<GfxState<'a>>,
    rotation: f32,
    simulation: WaveSimulation,
    camera: Camera,
    render_config: RenderConfig,
    camera_config: CameraConfig,
    mouse_position: Vec2,
    show_settings: bool,
}

impl App<'_> {
    pub fn new() -> Self {
        let camera = Camera::new(vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 0.0));
        Self {
            window: None,
            renderer: None,
            rotation: 0.0,
            simulation: WaveSimulation::new(simulation::DIVISIONS),
            mouse_position: Vec2::ZERO,
            render_config: RenderConfig {
                prism_type: PrismType::Hex,
                prism_height: 1.0,
                grid_size: 20,
                step_size: 1.3,
            },
            camera_config: CameraConfig {
                rotation_enabled: false,
                rotation_speed: 0.25,
            },
            camera,
            show_settings: true,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let renderer = self.renderer.as_mut().unwrap();
        let window = renderer.window.clone();
        renderer.egui_renderer.handle_input(
            &window,
            event);

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let size = self.renderer.as_mut().unwrap().size;
                self.mouse_position = vec2(
                    position.x as f32 / size.width as f32 * 2.0 - 1.0,
                    -1.0 * (position.y as f32 / size.height as f32 * 2.0 - 1.0),
                );
                // println!("mouse coords: {:?}", self.mouse_position);
            }
            WindowEvent::MouseInput { state: Pressed, .. } => {
                let result = camera::project_screen_onto_plane(self.mouse_position, Plane3::ZX,
                                                               renderer.projection.calc_matrix(), self.camera.calc_matrix());
                if let Some(intersection) = result {
                    let plane_point = intersection.xz();
                    println!("mouse click result: {}", plane_point);
                    if self.render_config.prism_type == PrismType::Square {
                        let size = self.render_config.grid_size as f32 * self.render_config.step_size;
                        let normalized = (plane_point + (size / 2.0)) / size;
                        println!("normalized: {}", normalized);
                        self.simulation.poke_normalized(normalized);
                    } else {
                        let hexes = self.render_config.grid_size as f32 * 2.0 + 1.0;
                        let grid_width = hexes * self.render_config.step_size * 3.0_f32.sqrt() * 0.5;
                        println!("grid width: {}", grid_width);


                        // let grid_half_width = self.render_config.grid_size as f32 * self.render_config.step_size * 3.0_f32.sqrt();
                        let normalized = (plane_point + (grid_width / 2.0)) / grid_width;
                        println!("normalized: {}", normalized);
                        self.simulation.poke_normalized(normalized);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: physical_key,
                    state: Pressed,
                    repeat: false,
                    ..
                }, ..
            } => {
                match physical_key {
                    PhysicalKey::Code(KeyCode::Space) => {
                        let renderer = self.renderer.as_mut().unwrap();
                        renderer.sim.render_mode = match renderer.sim.render_mode {
                            RenderMode::Texture => RenderMode::Prism,
                            RenderMode::Prism => RenderMode::Texture,
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => self.show_settings = !self.show_settings,
                    _ => {}
                }
            }
            _ => {}
        }
        return false;
    }

    pub fn render_ui(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        egui::Window::new("Settings")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .open(&mut self.show_settings)
            .show(renderer.egui_renderer.context(), |ui| {
                ui.label("Simulation");
                ui.add(egui::Slider::new(&mut self.simulation.damping, 0.0..=1.0).fixed_decimals(3).text("Damping"));

                ui.separator();
                ui.label("Render");
                let mut config = self.render_config;
                egui::Slider::new::<usize>(&mut config.grid_size, 2..=148)
                    .integer()
                    .step_by(1.0)
                    .text("Grid Size")
                    .ui(ui);

                egui::Slider::new(&mut config.step_size, 1.0..=5.0)
                    .text("Step size")
                    .ui(ui);

                egui::Slider::new(&mut config.prism_height, 1.0..=256.0)
                    .text("Prism Height")
                    .ui(ui);

                if self.render_config.differs(config) {
                    self.render_config = config;
                    match self.render_config.prism_type {
                        PrismType::Square => {
                            let mesh = mesh::square_prism(config.prism_height);
                            let grid = mesh_grid::MeshGrid::square_grid(config.grid_size, config.step_size);
                            renderer.sim.update_prism(&renderer.device, &mesh);
                            renderer.sim.update_grid(&renderer.device, &grid);
                        }
                        PrismType::Hex => {
                            let mesh = mesh::hex_prism(config.prism_height);
                            let grid = mesh_grid::MeshGrid::hex_grid(config.grid_size, config.step_size);
                            renderer.sim.update_prism(&renderer.device, &mesh);
                            renderer.sim.update_grid(&renderer.device, &grid);
                        }
                    }
                }

                ui.separator();
                ui.label("Camera");

                egui::Checkbox::new(&mut self.camera_config.rotation_enabled, "Rotation Enabled").ui(ui);
                egui::Slider::new(&mut self.camera_config.rotation_speed, 0.0..=2.0)
                    .text("Rotation Speed")
                    .ui(ui);
            });
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
            self.window = Some(window.clone());

            let (mesh, grid) = match self.render_config.prism_type {
                PrismType::Square => {
                    let mesh = mesh::square_prism(self.render_config.prism_height);
                    let grid = mesh_grid::MeshGrid::square_grid(self.render_config.grid_size, self.render_config.step_size);
                    (mesh, grid)
                }
                PrismType::Hex => {
                    let mesh = mesh::hex_prism(self.render_config.prism_height);
                    let grid = mesh_grid::MeshGrid::hex_grid(self.render_config.grid_size, self.render_config.step_size);
                    (mesh, grid)
                }
            };
            let state = pollster::block_on(
                GfxState::new(
                    window.clone(), 60f32.to_radians(), &mesh, &grid)
            );
            self.renderer = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if self.input(&event) {
            return;
        }


        match event {
            WindowEvent::CloseRequested => {
                info!("Window close button pressed: stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let step = simulation::PRISM_STEP;
                let half_length = self.render_config.grid_size as f32 * step / 1.0;

                let renderer = self.renderer.as_mut().unwrap();
                if self.camera_config.rotation_enabled {
                    self.rotation += self.camera_config.rotation_speed.to_radians();
                }
                let pos = vec3(self.rotation.sin(), 0.0, -self.rotation.cos()) * half_length * 1.5;
                self.camera.position = pos;
                self.camera.position.y = self.render_config.grid_size as f32 + 5.0;/// 2.0;
                renderer.egui_renderer.begin_frame(self.window.as_ref().unwrap());
                self.render_ui();

                use wgpu::SurfaceError as SE;
                self.simulation.advance();

                let renderer = self.renderer.as_mut().unwrap();
                let camera_transform = renderer.projection.calc_matrix() * self.camera.calc_matrix();
                renderer.sim.set_camera_transform(&renderer.queue, camera_transform);
                let (divisions, sim_data) = self.simulation.current_state();
                renderer.sim.update_sim_data(&renderer.queue, divisions, sim_data);
                match renderer.render() {
                    Ok(_) => {}
                    Err(SE::Lost | SE::Outdated) => renderer.resize(renderer.size),
                    Err(SE::OutOfMemory) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
                    }
                    Err(SE::Timeout) => {
                        log::warn!("Surface timeout");
                    }
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                let renderer = self.renderer.as_mut().unwrap();
                renderer.resize(new_size);
            }
            _ => {}
        }
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}


