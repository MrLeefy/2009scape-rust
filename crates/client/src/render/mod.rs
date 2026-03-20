//! wgpu-based renderer — handles both native and WASM rendering.

pub mod renderer2d;
pub mod renderer3d;
pub mod camera;

use renderer2d::Renderer2D;
use renderer3d::Renderer3D;
use camera::Camera3D;
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
    pub renderer_3d: Renderer3D,
    pub camera: Camera3D,
}

impl Renderer {
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
        let renderer_3d = Renderer3D::new(&device, format, config.width, config.height);
        let camera = Camera3D::new();

        info!("Renderer initialized: {}x{}, format: {:?}", config.width, config.height, format);

        Ok(Renderer {
            surface, device, queue, config,
            quality: RenderQuality::Standard,
            renderer_2d, renderer_3d, camera,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.renderer_2d.resize(&self.queue, width, height);
            self.renderer_3d.resize(&self.device, width, height);
        }
    }

    pub fn render(&mut self, game: &super::game::Game) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Encoder"),
        });

        match game.state {
            super::game::GameState::Login | super::game::GameState::Loading => {
                // 2D login/loading screen
                self.renderer_2d.begin();
                if game.state == super::game::GameState::Login {
                    self.draw_login_screen(game);
                } else {
                    self.draw_loading_screen(game);
                }

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("2D Pass"),
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
                    self.renderer_2d.flush(&self.device, &mut render_pass);
                }
            }

            super::game::GameState::InGame => {
                // 3D world rendering
                let aspect = self.config.width as f32 / self.config.height as f32;
                self.renderer_3d.update_camera(&self.queue, &self.camera, aspect);

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("3D Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.08, b: 0.15, a: 1.0 }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: self.renderer_3d.depth_view(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        ..Default::default()
                    });
                    self.renderer_3d.render(&mut render_pass);
                }

                // 2D HUD overlay on top
                self.renderer_2d.begin();
                self.draw_hud(game);

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("HUD Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });
                    self.renderer_2d.flush(&self.device, &mut render_pass);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn draw_login_screen(&mut self, game: &super::game::Game) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let r2d = &mut self.renderer_2d;

        r2d.fill_gradient_v(0.0, 0.0, w, h, [0.05, 0.05, 0.15, 1.0], [0.0, 0.0, 0.0, 1.0]);

        let panel_w = 350.0;
        let panel_h = 280.0;
        let px = (w - panel_w) / 2.0;
        let py = (h - panel_h) / 2.0;

        r2d.fill_rect(px, py, panel_w, panel_h, [0.1, 0.1, 0.2, 0.9]);
        r2d.stroke_rect(px, py, panel_w, panel_h, 2.0, [0.3, 0.3, 0.6, 1.0]);

        // Title
        r2d.fill_gradient_h(px + 2.0, py + 2.0, panel_w - 4.0, 40.0, [0.2, 0.15, 0.4, 1.0], [0.1, 0.1, 0.3, 1.0]);
        let gold = [0.85, 0.72, 0.2, 1.0];
        for i in 0..10 {
            r2d.fill_rect(px + 90.0 + (i as f32 * 18.0), py + 12.0, 14.0, 16.0, gold);
        }

        let field_x = px + 30.0;
        let field_w = panel_w - 60.0;

        // Username
        r2d.fill_rect(field_x, py + 65.0, 80.0, 16.0, [0.6, 0.6, 0.8, 0.8]);
        let ub = if game.active_field == 0 { [0.5, 0.7, 1.0, 1.0] } else { [0.3, 0.3, 0.5, 1.0] };
        r2d.fill_rect(field_x, py + 85.0, field_w, 30.0, [0.05, 0.05, 0.1, 1.0]);
        r2d.stroke_rect(field_x, py + 85.0, field_w, 30.0, 1.5, ub);
        if game.active_field == 0 && (game.tick / 30) % 2 == 0 {
            r2d.fill_rect(field_x + 6.0 + (game.username.len() as f32 * 8.0), py + 90.0, 2.0, 20.0, [1.0, 1.0, 1.0, 0.8]);
        }
        for (i, _) in game.username.chars().enumerate() {
            r2d.fill_rect(field_x + 6.0 + (i as f32 * 8.0), py + 93.0, 6.0, 14.0, [1.0, 1.0, 1.0, 0.9]);
        }

        // Password
        r2d.fill_rect(field_x, py + 130.0, 80.0, 16.0, [0.6, 0.6, 0.8, 0.8]);
        let pb = if game.active_field == 1 { [0.5, 0.7, 1.0, 1.0] } else { [0.3, 0.3, 0.5, 1.0] };
        r2d.fill_rect(field_x, py + 150.0, field_w, 30.0, [0.05, 0.05, 0.1, 1.0]);
        r2d.stroke_rect(field_x, py + 150.0, field_w, 30.0, 1.5, pb);
        if game.active_field == 1 && (game.tick / 30) % 2 == 0 {
            r2d.fill_rect(field_x + 6.0 + (game.password.len() as f32 * 8.0), py + 155.0, 2.0, 20.0, [1.0, 1.0, 1.0, 0.8]);
        }
        for i in 0..game.password.len() {
            r2d.fill_rect(field_x + 8.0 + (i as f32 * 8.0), py + 160.0, 5.0, 5.0, [1.0, 1.0, 1.0, 0.9]);
        }

        // Login button
        let btn_x = px + (panel_w - 120.0) / 2.0;
        r2d.fill_gradient_v(btn_x, py + 200.0, 120.0, 36.0, [0.15, 0.4, 0.15, 1.0], [0.08, 0.25, 0.08, 1.0]);
        r2d.stroke_rect(btn_x, py + 200.0, 120.0, 36.0, 1.5, [0.3, 0.6, 0.3, 1.0]);
        for i in 0..5 {
            r2d.fill_rect(btn_x + 28.0 + (i as f32 * 14.0), py + 210.0, 10.0, 16.0, [0.9, 0.8, 0.3, 1.0]);
        }

        // Status
        if !game.status_message.is_empty() {
            let msg_w = game.status_message.len() as f32 * 7.0;
            let color = if game.status_message.contains("Error") { [1.0, 0.3, 0.3, 1.0] }
                else if game.status_message.contains("Connecting") { [1.0, 1.0, 0.3, 1.0] }
                else { [0.7, 0.7, 0.7, 0.8] };
            r2d.fill_rect(px + (panel_w - msg_w) / 2.0, py + 250.0, msg_w, 14.0, color);
        }
    }

    fn draw_loading_screen(&mut self, game: &super::game::Game) {
        let w = self.config.width as f32;
        let h = self.config.height as f32;
        let r2d = &mut self.renderer_2d;

        r2d.fill_rect(0.0, 0.0, w, h, [0.02, 0.02, 0.05, 1.0]);
        let bx = (w - 300.0) / 2.0;
        let by = (h - 30.0) / 2.0;
        r2d.fill_rect(bx, by, 300.0, 30.0, [0.1, 0.1, 0.15, 1.0]);
        r2d.stroke_rect(bx, by, 300.0, 30.0, 2.0, [0.3, 0.3, 0.5, 1.0]);
        let progress = ((game.tick as f32 / 200.0) % 1.0).min(1.0);
        r2d.fill_gradient_h(bx + 2.0, by + 2.0, 296.0 * progress, 26.0, [0.2, 0.5, 0.2, 1.0], [0.1, 0.3, 0.1, 1.0]);
    }

    fn draw_hud(&mut self, game: &super::game::Game) {
        let w = self.config.width as f32;
        let r2d = &mut self.renderer_2d;

        // Minimap background (top-right)
        r2d.fill_rect(w - 160.0, 5.0, 155.0, 155.0, [0.1, 0.1, 0.15, 0.7]);
        r2d.stroke_rect(w - 160.0, 5.0, 155.0, 155.0, 1.0, [0.4, 0.35, 0.2, 1.0]);

        // Compass dot (center of minimap)
        r2d.fill_rect(w - 85.0, 80.0, 6.0, 6.0, [1.0, 0.0, 0.0, 1.0]);

        // Chat box (bottom)
        r2d.fill_rect(0.0, self.config.height as f32 - 140.0, 520.0, 140.0, [0.05, 0.05, 0.1, 0.8]);
        r2d.stroke_rect(0.0, self.config.height as f32 - 140.0, 520.0, 140.0, 1.0, [0.3, 0.3, 0.5, 0.6]);

        // Inventory panel (right side)
        r2d.fill_rect(w - 200.0, 170.0, 195.0, 260.0, [0.08, 0.08, 0.12, 0.8]);
        r2d.stroke_rect(w - 200.0, 170.0, 195.0, 260.0, 1.0, [0.4, 0.35, 0.2, 0.6]);

        // Tab buttons along inventory top
        for i in 0..7 {
            let tx = w - 195.0 + (i as f32 * 27.0);
            r2d.fill_rect(tx, 172.0, 25.0, 18.0, [0.15, 0.13, 0.1, 0.9]);
        }

        // Info text area
        r2d.fill_rect(5.0, 5.0, 200.0, 16.0, [0.3, 0.3, 0.3, 0.4]);
    }
}
