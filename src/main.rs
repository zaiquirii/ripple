mod renderer;
mod mesh;
mod camera;
mod simulation;
mod mesh_grid;
mod texture;
mod egui_renderer;
mod sim_renderer;

use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use egui::{SliderOrientation, Widget};
use log::info;
use macaw::{Vec2, vec2, Vec3, vec3};
use winit::application::ApplicationHandler;
use winit::event::ElementState::Pressed;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::renderer::{GfxState};
use crate::sim_renderer::RenderMode;
use crate::simulation::WaveSimulation;

fn main() {
    run();
}

struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<GfxState<'a>>,
    rotation: f32,
    simulation: WaveSimulation,

    grid_size: usize,

    mouse_position: Vec2,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            rotation: 0.0,
            simulation: WaveSimulation::new(simulation::DIVISIONS),
            mouse_position: Vec2::ZERO,
            grid_size: 128,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let mut renderer = self.renderer.as_mut().unwrap();
        let window = renderer.window.clone();
        renderer.egui_renderer.handle_input(
            &window,
            event);

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let size = self.renderer.as_mut().unwrap().size;
                self.mouse_position = vec2(
                    position.x as f32 / size.width as f32,
                    position.y as f32 / size.height as f32,
                );
            }
            WindowEvent::MouseInput { state: Pressed, .. } => {
                self.simulation.poke_normalized(self.mouse_position);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Space),
                    state: Pressed,
                    repeat: false,
                    ..
                }, ..
            } => {
                let renderer = self.renderer.as_mut().unwrap();
                renderer.sim.render_mode = match renderer.sim.render_mode {
                    RenderMode::Texture => RenderMode::Prism,
                    RenderMode::Prism => RenderMode::Texture,
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
            .default_open(true)
            .show(renderer.egui_renderer.context(), |ui| {
                ui.label("Simulation");
                ui.add(egui::Slider::new(&mut self.simulation.damping, 0.0..=1.0).fixed_decimals(3).text("Damping"));
                ui.horizontal(|ui| {
                    ui.label("Damping");
                    ui.add(egui::Slider::new(&mut self.simulation.damping, 0.0..=1.0).fixed_decimals(3));
                });

                let mut sim_resolution = self.simulation.divisions();
                ui.add(egui::Slider::new(&mut sim_resolution, 4..=1024));
                if sim_resolution != self.simulation.divisions() {}


                let mut grid_size = self.grid_size;
                ui.add(
                    egui::Slider::new::<usize>(&mut grid_size, 5..=1024)
                );

                if ui.button("Button!").clicked() {
                    println!("boom!")
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Pixels per point: {}",
                        renderer.egui_renderer.context().pixels_per_point()
                    ));
                    if ui.button("-").clicked() {
                        renderer.scale_factor = (renderer.scale_factor - 0.1).max(0.3);
                    }
                    if ui.button("+").clicked() {
                        renderer.scale_factor = (renderer.scale_factor + 0.1).min(3.0);
                    }
                });
            });
    }

    pub fn update_divisions(&mut self, divisions: usize) {
        // Update simulation
        // Update grid
        //
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
            self.window = Some(window.clone());

            let state = pollster::block_on(GfxState::new(window.clone(), 90f32.to_radians()));
            self.renderer = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if self.input(&event) {
            return;
        }

        let step = simulation::PRISM_STEP;
        let center_point = simulation::PRISM_SIZE as f32 * step / 2.0;

        let renderer = self.renderer.as_mut().unwrap();
        renderer.camera.target.x = center_point;
        renderer.camera.target.z = center_point;
        self.rotation += 0.001;
        let mut pos = vec3(self.rotation.sin(), 0.0, -self.rotation.cos()) * center_point * 1.5;
        pos.x += center_point;
        pos.z += center_point;
        renderer.camera.position = pos;
        renderer.camera.position.y = simulation::PRISM_SIZE as f32 / 2.0;

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close button pressed: stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.egui_renderer.begin_frame(self.window.as_ref().unwrap());
                self.render_ui();

                use wgpu::SurfaceError as SE;
                self.simulation.advance();

                let renderer = self.renderer.as_mut().unwrap();
                let camera_transform = renderer.projection.calc_matrix() * renderer.camera.calc_matrix();
                renderer.sim.set_camera_transform(&renderer.queue, camera_transform);
                let (divisions, sim_data) = self.simulation.current_state();
                renderer.sim.update_sim_data(&renderer.queue, divisions, sim_data);
                // renderer.update_sim_texture(self.simulation.current_state_old());
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


