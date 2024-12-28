mod renderer;
mod mesh;

use std::sync::Arc;
use log::info;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use crate::renderer::GfxState;

fn main() {
    run();
}

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<GfxState<'a>>
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
            self.window = Some(window.clone());

            // let state = pollster
            let state = pollster::block_on(GfxState::new(window.clone()));
            self.renderer = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let state = self.renderer.as_mut().unwrap();
        if state.input(&event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close button pressed: stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
                use wgpu::SurfaceError as SE;
                match state.render() {
                    Ok(_) => {}
                    Err(SE::Lost | SE::Outdated) => state.resize(state.size),
                    Err(SE::OutOfMemory) => {
                        log::error!("OutOfMemory");
                        event_loop.exit();
                    }
                    Err(SE::Timeout) => {
                        log::warn!("Surface timeout");
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
            }
            _ => {}
        }
    }
}


pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
