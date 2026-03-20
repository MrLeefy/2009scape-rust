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
use input::InputState;
use render::Renderer;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};
use winit::keyboard::{Key, NamedKey};
use log::{info, warn};
use std::sync::Arc;
use std::collections::HashSet;
use std::sync::mpsc;
use net::protocol::PacketHandler;

/// Messages from the async login/network thread to the game loop.
enum NetMessage {
    LoginSuccess { player_id: u16, staff_level: u8, member: bool },
    LoginFailed(String),
    ServerPacket { opcode: u8, data: Vec<u8> },
    Disconnected,
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game: Game,
    keys_held: HashSet<String>,
    input: InputState,
    /// Channel receiver for network messages.
    net_rx: Option<mpsc::Receiver<NetMessage>>,
    /// Whether a login attempt is already in flight.
    login_in_flight: bool,
    /// Keepalive counter (every 25 ticks = ~15s).
    keepalive_counter: u32,
}

impl App {
    fn new() -> Self {
        App {
            window: None,
            renderer: None,
            game: Game::new(),
            keys_held: HashSet::new(),
            input: InputState::new(),
            net_rx: None,
            login_in_flight: false,
            keepalive_counter: 0,
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

        // Scroll wheel zoom
        if self.input.scroll_delta != 0.0 {
            renderer.camera.zoom = (renderer.camera.zoom - self.input.scroll_delta * 20.0).clamp(100.0, 2000.0);
        }
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
                            // H key toggles HD mode when in-game
                            if c.as_str() == "h" && self.game.state == game::GameState::InGame {
                                if let Some(renderer) = &mut self.renderer {
                                    renderer.toggle_quality();
                                    let mode_name = match renderer.quality {
                                        render::RenderQuality::Standard => "SD",
                                        render::RenderQuality::HighDetail => "HD",
                                    };
                                    self.game.status_message = format!("Render mode: {}", mode_name);
                                }
                            }
                            for ch in c.chars() {
                                self.game.on_char(ch);
                            }
                        }
                        _ => {}
                    }
                } else {
                    self.keys_held.remove(&key_str);
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.input.on_move(position.x as f32, position.y as f32);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let is_left = button == winit::event::MouseButton::Left;
                let is_right = button == winit::event::MouseButton::Right;
                if state.is_pressed() {
                    if is_left { self.input.on_left_press(); }
                    if is_right { self.input.on_right_press(); }
                } else {
                    if is_left { self.input.on_left_release(); }
                    if is_right { self.input.on_right_release(); }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 / 30.0,
                };
                self.input.on_scroll(d);
            }

            WindowEvent::RedrawRequested => {
                // Check if login was requested and spawn async login task
                if self.game.login_requested && !self.login_in_flight {
                    self.game.login_requested = false;
                    self.login_in_flight = true;
                    self.game.status_message = "Connecting to server...".to_string();

                    let username = self.game.username.clone();
                    let password = self.game.password.clone();
                    let (tx, rx) = mpsc::channel();
                    self.net_rx = Some(rx);

                    // Spawn async login + packet loop on a background thread
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("Failed to create tokio runtime");

                        rt.block_on(async move {
                            let host = "localhost";
                            let port = 43594;
                            info!("Attempting login to {}:{} as {}", host, port, username);

                            match net::login::login(host, port, &username, &password).await {
                                Ok(session) => {
                                    let _ = tx.send(NetMessage::LoginSuccess {
                                        player_id: session.player_id,
                                        staff_level: session.staff_mod_level,
                                        member: session.player_member,
                                    });

                                    // Packet receive loop
                                    let mut stream = session.stream;
                                    let mut in_cipher = session.in_cipher;
                                    loop {
                                        use tokio::io::AsyncReadExt;
                                        let raw_opcode = match stream.read_u8().await {
                                            Ok(v) => v,
                                            Err(_) => {
                                                let _ = tx.send(NetMessage::Disconnected);
                                                break;
                                            }
                                        };

                                        // Decode ISAAC-encrypted opcode
                                        let opcode = raw_opcode.wrapping_sub(in_cipher.next_key() as u8);
                                        let pkt_len = net::protocol::PACKET_LENGTHS[opcode as usize];

                                        let length = match pkt_len {
                                            0 => 0usize,
                                            n if n > 0 => n as usize,
                                            -1 => match stream.read_u8().await {
                                                Ok(v) => v as usize,
                                                Err(_) => { let _ = tx.send(NetMessage::Disconnected); break; }
                                            },
                                            -2 => {
                                                let hi = match stream.read_u8().await {
                                                    Ok(v) => v,
                                                    Err(_) => { let _ = tx.send(NetMessage::Disconnected); break; }
                                                };
                                                let lo = match stream.read_u8().await {
                                                    Ok(v) => v,
                                                    Err(_) => { let _ = tx.send(NetMessage::Disconnected); break; }
                                                };
                                                ((hi as usize) << 8) | (lo as usize)
                                            }
                                            _ => 0,
                                        };

                                        if length > 0 && length < 65536 {
                                            let mut data = vec![0u8; length];
                                            match stream.read_exact(&mut data).await {
                                                Ok(_) => {
                                                    let _ = tx.send(NetMessage::ServerPacket { opcode, data });
                                                }
                                                Err(_) => {
                                                    let _ = tx.send(NetMessage::Disconnected);
                                                    break;
                                                }
                                            }
                                        } else if length == 0 {
                                            let _ = tx.send(NetMessage::ServerPacket { opcode, data: Vec::new() });
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(NetMessage::LoginFailed(e.to_string()));
                                }
                            }
                        });
                    });
                }

                // Process network messages — collect first to avoid borrow conflict
                let msgs: Vec<NetMessage> = self.net_rx.as_ref()
                    .map(|rx| {
                        let mut v = Vec::new();
                        while let Ok(m) = rx.try_recv() { v.push(m); }
                        v
                    })
                    .unwrap_or_default();

                let mut disconnected = false;
                for msg in msgs {
                    match msg {
                        NetMessage::LoginSuccess { player_id, staff_level, member } => {
                            info!("Login successful! player_id={}, staff={}, member={}", player_id, staff_level, member);
                            self.game.state = game::GameState::InGame;
                            self.game.status_message = format!("Welcome! (ID: {})", player_id);
                            self.login_in_flight = false;
                        }
                        NetMessage::LoginFailed(reason) => {
                            warn!("Login failed: {}", reason);
                            self.game.status_message = format!("Login failed: {}", reason);
                            self.login_in_flight = false;
                        }
                        NetMessage::ServerPacket { opcode, data } => {
                            self.game.packets.process_packet(opcode, &data);
                        }
                        NetMessage::Disconnected => {
                            warn!("Disconnected from server");
                            self.game.status_message = "Disconnected from server".to_string();
                            self.game.state = game::GameState::Login;
                            self.login_in_flight = false;
                            disconnected = true;
                        }
                    }
                }
                if disconnected {
                    self.net_rx = None;
                }

                // Check for logout
                if self.game.packets.should_logout {
                    self.game.packets.should_logout = false;
                    self.game.state = game::GameState::Login;
                    self.game.status_message = "You have been logged out.".to_string();
                    self.net_rx = None;
                }

                self.game.tick();
                self.process_camera_movement();

                if let Some(renderer) = &mut self.renderer {
                    if let Err(e) = renderer.render(&self.game) {
                        log::error!("Render error: {}", e);
                    }
                }

                self.input.end_frame();

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
    info!("  H key = toggle SD/HD rendering mode");

    // Try to load cache on startup
    let cache_dirs = [
        "C:\\Users\\baseb\\.2009scape\\cache",
        "C:\\2009scape\\cache",
        "cache",
    ];
    for dir in &cache_dirs {
        if std::path::Path::new(dir).join("main_file_cache.dat2").exists() {
            info!("Found cache at: {}", dir);
            match cache::Js5Cache::open(dir) {
                Ok(cache) => {
                    info!("Cache loaded: {} indices", cache.parsed_indices.len());
                    let mut loader = cache::loader::DefinitionLoader::new();
                    if let Ok(n) = loader.load_items(&cache) {
                        info!("Loaded {} item definitions", n);
                    }
                    if let Ok(n) = loader.load_npcs(&cache) {
                        info!("Loaded {} NPC definitions", n);
                    }
                }
                Err(e) => warn!("Failed to load cache: {}", e),
            }
            break;
        }
    }

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
