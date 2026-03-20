//! 2009Scape Rust Client

pub mod audio;
pub mod cache;
pub mod combat;
pub mod entity;
pub mod game;
pub mod input;
pub mod net;
pub mod render;
pub mod skills;
pub mod web;

use game::Game;
use render::Renderer;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};
use winit::keyboard::{Key, NamedKey};
use log::info;
use std::sync::Arc;
use std::collections::HashSet;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game: Game,
    keys_held: HashSet<String>,
}

impl App {
    fn new() -> Self {
        App {
            window: None,
            renderer: None,
            game: Game::new(),
            keys_held: HashSet::new(),
        }
    }

    fn process_camera_movement(&mut self) {
        if self.game.state != game::GameState::InGame { return; }
        let renderer = match &mut self.renderer { Some(r) => r, None => return };

        let speed = 16.0;
        let mut forward = 0.0f32;
        let mut right = 0.0f32;

        if self.keys_held.contains("w") || self.keys_held.contains("ArrowUp") { forward += speed; }
        if self.keys_held.contains("s") || self.keys_held.contains("ArrowDown") { forward -= speed; }
        if self.keys_held.contains("a") || self.keys_held.contains("ArrowLeft") { right -= speed; }
        if self.keys_held.contains("d") || self.keys_held.contains("ArrowRight") { right += speed; }

        if forward != 0.0 || right != 0.0 {
            renderer.camera.translate(forward, right);
        }

        // Camera rotation with Q/E
        if self.keys_held.contains("q") { renderer.camera.rotate(-8.0, 0.0); }
        if self.keys_held.contains("e") { renderer.camera.rotate(8.0, 0.0); }

        // Camera pitch with R/F
        if self.keys_held.contains("r") { renderer.camera.rotate(0.0, -3.0); }
        if self.keys_held.contains("f") { renderer.camera.rotate(0.0, 3.0); }

        // Zoom with Z/X
        if self.keys_held.contains("z") { renderer.camera.zoom = (renderer.camera.zoom - 10.0).max(100.0); }
        if self.keys_held.contains("x") { renderer.camera.zoom = (renderer.camera.zoom + 10.0).min(2000.0); }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }

        let attrs = WindowAttributes::default()
            .with_title("2009Scape — Rust Client")
            .with_inner_size(winit::dpi::LogicalSize::new(765, 503));

        let window = Arc::new(event_loop.create_window(attrs).expect("Failed to create window"));
        self.window = Some(window.clone());

        let window_ref: &'static Window = unsafe { &*(Arc::as_ptr(&window)) };

        match pollster::block_on(Renderer::new(window_ref)) {
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
                let key_str = match &event.logical_key {
                    Key::Character(c) => c.to_string(),
                    Key::Named(n) => format!("{:?}", n),
                    _ => String::new(),
                };

                if event.state.is_pressed() {
                    self.keys_held.insert(key_str.clone());

                    match &event.logical_key {
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        Key::Named(NamedKey::Tab) => self.game.on_tab(),
                        Key::Named(NamedKey::Enter) => {
                            if self.game.state == game::GameState::Login {
                                // Quick-enter: skip to InGame for testing
                                if self.game.username.is_empty() {
                                    self.game.state = game::GameState::InGame;
                                    self.game.status_message = "Entered world (test mode)".to_string();
                                    info!("Entered InGame state (test mode)");
                                } else {
                                    self.game.on_enter();
                                }
                            }
                        }
                        Key::Named(NamedKey::Backspace) => self.game.on_backspace(),
                        Key::Character(c) => {
                            if self.game.state == game::GameState::Login {
                                for ch in c.chars() {
                                    self.game.on_char(ch);
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    self.keys_held.remove(&key_str);
                }
            }

            WindowEvent::RedrawRequested => {
                self.game.tick();
                self.process_camera_movement();

                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render(&self.game) {
                        log::error!("Render error: {}", e);
                    }
                }

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
    info!("║   2009Scape Rust Client v0.2.0       ║");
    info!("║   Native + Browser (WASM/PWA)        ║");
    info!("╚══════════════════════════════════════╝");
    info!("Controls:");
    info!("  Login: Type username/password, Tab, Enter");
    info!("  World: WASD/Arrows=move, Q/E=rotate, R/F=pitch, Z/X=zoom");
    info!("  Press Enter with empty username to skip to world view");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
