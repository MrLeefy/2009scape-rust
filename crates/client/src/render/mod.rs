//! wgpu-based renderer — handles both native and WASM rendering.

pub mod renderer2d;

use renderer2d::Renderer2D;
use wgpu;
use winit::window::Window;
use anyhow::Result;
use log::info;

/// Quality preset for SD/HD rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderQuality {
    Standard,
    HighDetail,
}

/// Main renderer state.
pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub quality: RenderQuality,
    pub renderer_2d: Renderer2D,
}

impl Renderer {
    /// Initialize the wgpu renderer from a window.
    pub async fn new(window: &'static Window) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| anyhow::anyhow!("No suitable GPU adapter found"))?;

        info!("GPU adapter: {:?}", adapter.get_info().name);

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("RS2 Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ).await?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer_2d = Renderer2D::new(&device, &queue, format, config.width, config.height);

        info!("Renderer initialized: {}x{}, format: {:?}", config.width, config.height, format);

        Ok(Renderer {
            surface,
            device,
            queue,
            config,
            quality: RenderQuality::Standard,
            renderer_2d,
        })
    }

    /// Resize the render surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.renderer_2d.resize(&self.queue, width, height);
        }
    }

    /// Render a frame with the game state.
    pub fn render(&mut self, game: &super::game::Game) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Encoder"),
        });

        // Start 2D batch
        self.renderer_2d.begin();

        // Draw based on game state
        match game.state {
            super::game::GameState::Login => {
                self.draw_login_screen(game);
            }
            super::game::GameState::Loading => {
                self.draw_loading_screen(game);
            }
            super::game::GameState::InGame => {
                // Phase 3+: world rendering
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // Flush 2D quads
            self.renderer_2d.flush(&self.device, &mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn draw_login_screen(&mut self, game: &super::game::Game) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let r2d = &mut self.renderer_2d;

        // Background gradient (dark blue to black)
        r2d.fill_gradient_v(0.0, 0.0, w, h,
            [0.05, 0.05, 0.15, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        );

        // Center panel
        let panel_w = 350.0;
        let panel_h = 280.0;
        let px = (w - panel_w) / 2.0;
        let py = (h - panel_h) / 2.0;

        // Panel background
        r2d.fill_rect(px, py, panel_w, panel_h, [0.1, 0.1, 0.2, 0.9]);

        // Panel border
        r2d.stroke_rect(px, py, panel_w, panel_h, 2.0, [0.3, 0.3, 0.6, 1.0]);

        // Title bar
        r2d.fill_gradient_h(px + 2.0, py + 2.0, panel_w - 4.0, 40.0,
            [0.2, 0.15, 0.4, 1.0],
            [0.1, 0.1, 0.3, 1.0],
        );

        // "2009Scape" title (represented as colored blocks since we don't have font rendering yet)
        // Golden blocks spelling out "2009Scape" in a pixel-art style
        let title_y = py + 12.0;
        let gold = [0.85, 0.72, 0.2, 1.0];
        for i in 0..10 {
            let bx = px + 90.0 + (i as f32 * 18.0);
            r2d.fill_rect(bx, title_y, 14.0, 16.0, gold);
        }

        // Username field
        let field_x = px + 30.0;
        let field_w = panel_w - 60.0;

        // Username label area
        r2d.fill_rect(field_x, py + 65.0, 80.0, 16.0, [0.6, 0.6, 0.8, 0.8]);

        // Username input box
        let uname_active = game.active_field == 0;
        let uname_border = if uname_active { [0.5, 0.7, 1.0, 1.0] } else { [0.3, 0.3, 0.5, 1.0] };
        r2d.fill_rect(field_x, py + 85.0, field_w, 30.0, [0.05, 0.05, 0.1, 1.0]);
        r2d.stroke_rect(field_x, py + 85.0, field_w, 30.0, 1.5, uname_border);

        // Username text cursor (blinking)
        if uname_active && (game.tick / 30) % 2 == 0 {
            let cursor_x = field_x + 6.0 + (game.username.len() as f32 * 8.0);
            r2d.fill_rect(cursor_x, py + 90.0, 2.0, 20.0, [1.0, 1.0, 1.0, 0.8]);
        }

        // Username text (white blocks per character)
        for (i, _) in game.username.chars().enumerate() {
            r2d.fill_rect(field_x + 6.0 + (i as f32 * 8.0), py + 93.0, 6.0, 14.0, [1.0, 1.0, 1.0, 0.9]);
        }

        // Password label area  
        r2d.fill_rect(field_x, py + 130.0, 80.0, 16.0, [0.6, 0.6, 0.8, 0.8]);

        // Password input box
        let pass_active = game.active_field == 1;
        let pass_border = if pass_active { [0.5, 0.7, 1.0, 1.0] } else { [0.3, 0.3, 0.5, 1.0] };
        r2d.fill_rect(field_x, py + 150.0, field_w, 30.0, [0.05, 0.05, 0.1, 1.0]);
        r2d.stroke_rect(field_x, py + 150.0, field_w, 30.0, 1.5, pass_border);

        // Password cursor
        if pass_active && (game.tick / 30) % 2 == 0 {
            let cursor_x = field_x + 6.0 + (game.password.len() as f32 * 8.0);
            r2d.fill_rect(cursor_x, py + 155.0, 2.0, 20.0, [1.0, 1.0, 1.0, 0.8]);
        }

        // Password dots (asterisks)
        for i in 0..game.password.len() {
            r2d.fill_rect(field_x + 8.0 + (i as f32 * 8.0), py + 160.0, 5.0, 5.0, [1.0, 1.0, 1.0, 0.9]);
        }

        // Login button
        let btn_w = 120.0;
        let btn_h = 36.0;
        let btn_x = px + (panel_w - btn_w) / 2.0;
        let btn_y = py + 200.0;

        r2d.fill_gradient_v(btn_x, btn_y, btn_w, btn_h,
            [0.15, 0.4, 0.15, 1.0],
            [0.08, 0.25, 0.08, 1.0],
        );
        r2d.stroke_rect(btn_x, btn_y, btn_w, btn_h, 1.5, [0.3, 0.6, 0.3, 1.0]);

        // "LOGIN" text blocks
        let login_gold = [0.9, 0.8, 0.3, 1.0];
        for i in 0..5 {
            let lx = btn_x + 28.0 + (i as f32 * 14.0);
            r2d.fill_rect(lx, btn_y + 10.0, 10.0, 16.0, login_gold);
        }

        // Status message area
        let status_color = match &game.status_message {
            s if s.contains("Error") || s.contains("failed") => [1.0, 0.3, 0.3, 1.0],
            s if s.contains("Connecting") || s.contains("Loading") => [1.0, 1.0, 0.3, 1.0],
            s if s.contains("Success") => [0.3, 1.0, 0.3, 1.0],
            _ => [0.7, 0.7, 0.7, 0.8],
        };

        if !game.status_message.is_empty() {
            let msg_w = game.status_message.len() as f32 * 7.0;
            let msg_x = px + (panel_w - msg_w) / 2.0;
            r2d.fill_rect(msg_x, py + 250.0, msg_w, 14.0, status_color);
        }

        // Server info at bottom
        let info_y = h - 20.0;
        r2d.fill_rect(5.0, info_y, 200.0, 12.0, [0.4, 0.4, 0.4, 0.5]);
    }

    fn draw_loading_screen(&mut self, game: &super::game::Game) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let r2d = &mut self.renderer_2d;

        // Dark background
        r2d.fill_rect(0.0, 0.0, w, h, [0.02, 0.02, 0.05, 1.0]);

        // Loading bar background
        let bar_w = 300.0;
        let bar_h = 30.0;
        let bx = (w - bar_w) / 2.0;
        let by = (h - bar_h) / 2.0;

        r2d.fill_rect(bx, by, bar_w, bar_h, [0.1, 0.1, 0.15, 1.0]);
        r2d.stroke_rect(bx, by, bar_w, bar_h, 2.0, [0.3, 0.3, 0.5, 1.0]);

        // Loading bar fill (animated)
        let progress = ((game.tick as f32 / 200.0) % 1.0).min(1.0);
        let fill_w = (bar_w - 4.0) * progress;
        r2d.fill_gradient_h(bx + 2.0, by + 2.0, fill_w, bar_h - 4.0,
            [0.2, 0.5, 0.2, 1.0],
            [0.1, 0.3, 0.1, 1.0],
        );

        // "Loading..." text blocks
        let text_y = by - 25.0;
        for i in 0..10 {
            r2d.fill_rect((w / 2.0) - 50.0 + (i as f32 * 10.0), text_y, 7.0, 12.0, [0.7, 0.7, 0.7, 0.8]);
        }
    }
}
