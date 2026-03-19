//! wgpu-based renderer — handles both native and WASM rendering.
//!
//! Phase 1: Basic window + clear color + 2D sprite prep.
//! Phase 2+: Login screen, world tiles, models, HD shading.

use wgpu;
use winit::window::Window;
use anyhow::Result;
use log::info;

/// Quality preset for SD/HD rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderQuality {
    /// Flat shading, no lighting, lower textures
    Standard,
    /// Per-pixel lighting, shadows, anti-aliasing
    HighDetail,
}

/// Main renderer state.
pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub quality: RenderQuality,
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

        info!("Renderer initialized: {}x{}, format: {:?}", config.width, config.height, format);

        Ok(Renderer {
            surface,
            device,
            queue,
            config,
            quality: RenderQuality::Standard,
        })
    }

    /// Resize the render surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Render a frame.
    pub fn render(&mut self, game_state: &super::game::GameState) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Frame Encoder"),
        });

        // Clear color based on game state
        let clear_color = match game_state {
            super::game::GameState::Login => wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            super::game::GameState::Loading => wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 },
            super::game::GameState::InGame => wgpu::Color { r: 0.3, g: 0.5, b: 0.3, a: 1.0 },
        };

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // TODO Phase 2: Draw login screen sprites
            // TODO Phase 3: Draw world tiles and models
            // TODO Phase 4: Draw entities
            // TODO Phase 5: Draw UI overlays
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
