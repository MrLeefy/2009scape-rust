//! 2009Scape Rust Client
//!
//! A modern Rust/WASM rewrite of the RuneScape RT4 client.
//! Supports both native desktop and browser (PWA) via WebAssembly.

pub mod cache;
pub mod game;
pub mod net;
pub mod render;

use game::{Game, GameState};
use render::Renderer;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};
use log::info;
use std::sync::Arc;

/// Main application state.
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game: Game,
}

impl App {
    fn new() -> Self {
        App {
            window: None,
            renderer: None,
            game: Game::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("2009Scape — Rust Client")
            .with_inner_size(winit::dpi::LogicalSize::new(765, 503));

        let window = Arc::new(event_loop.create_window(attrs).expect("Failed to create window"));
        self.window = Some(window.clone());

        // Initialize renderer
        let window_ref: &'static Window = unsafe {
            // SAFETY: Window lives as long as the app (stored in Arc)
            &*(Arc::as_ptr(&window))
        };

        let renderer = pollster::block_on(Renderer::new(window_ref));
        match renderer {
            Ok(r) => {
                info!("Renderer initialized successfully");
                self.renderer = Some(r);
            }
            Err(e) => {
                log::error!("Failed to initialize renderer: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                // Phase 2: Handle keyboard input for login screen
                if event.state.is_pressed() {
                    if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) = event.logical_key {
                        event_loop.exit();
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                // Game tick
                self.game.tick();

                // Render frame
                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render(&self.game.state) {
                        log::error!("Render error: {}", e);
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    info!("╔══════════════════════════════════════╗");
    info!("║   2009Scape Rust Client v0.1.0       ║");
    info!("║   Native + Browser (WASM/PWA)        ║");
    info!("╚══════════════════════════════════════╝");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
