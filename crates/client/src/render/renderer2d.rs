//! 2D sprite/quad rendering pipeline.
//!
//! Provides immediate-mode quad drawing for UI, login screen, text, etc.
//! Uses an orthographic projection matching screen pixel coordinates.

use wgpu::util::DeviceExt;

/// A single vertex for 2D rendering.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2D {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex2D>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
            wgpu::VertexAttribute { offset: 16, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
        ],
    };
}

/// Uniform buffer for orthographic projection.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    projection: [[f32; 4]; 4],
}

/// 2D sprite rendering pipeline.
pub struct Renderer2D {
    pipeline_textured: wgpu::RenderPipeline,
    pipeline_solid: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    vertices: Vec<Vertex2D>,
    indices: Vec<u16>,
    white_texture: wgpu::Texture,
    white_bind_group: wgpu::BindGroup,
    screen_width: f32,
    screen_height: f32,
}

impl Renderer2D {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader2d.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("2D Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let create_pipeline = |fs_entry: &str, label: &str| -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex2D::LAYOUT],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs_entry),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        let pipeline_textured = create_pipeline("fs_main", "2D Textured Pipeline");
        let pipeline_solid = create_pipeline("fs_solid", "2D Solid Pipeline");

        // Orthographic projection
        let projection = ortho_projection(width as f32, height as f32);
        let uniforms = Uniforms { projection };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("2D Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 1x1 white texture for solid color rendering
        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &white_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &[255u8, 255, 255, 255],
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );

        let white_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("2D Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("2D Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&white_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        let white_bind_group = uniform_bind_group.clone();

        Renderer2D {
            pipeline_textured,
            pipeline_solid,
            uniform_buffer,
            uniform_bind_group,
            bind_group_layout,
            vertices: Vec::with_capacity(4096),
            indices: Vec::with_capacity(8192),
            white_texture,
            white_bind_group: white_bind_group,
            screen_width: width as f32,
            screen_height: height as f32,
        }
    }

    /// Update projection when window resizes.
    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.screen_width = width as f32;
        self.screen_height = height as f32;
        let projection = ortho_projection(self.screen_width, self.screen_height);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[Uniforms { projection }]));
    }

    /// Begin a new frame — clears the vertex/index buffers.
    pub fn begin(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let base = self.vertices.len() as u16;
        self.vertices.extend_from_slice(&[
            Vertex2D { position: [x, y], uv: [0.0, 0.0], color },
            Vertex2D { position: [x + w, y], uv: [1.0, 0.0], color },
            Vertex2D { position: [x + w, y + h], uv: [1.0, 1.0], color },
            Vertex2D { position: [x, y + h], uv: [0.0, 1.0], color },
        ]);
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Draw a horizontal gradient rectangle.
    pub fn fill_gradient_h(&mut self, x: f32, y: f32, w: f32, h: f32, left: [f32; 4], right: [f32; 4]) {
        let base = self.vertices.len() as u16;
        self.vertices.extend_from_slice(&[
            Vertex2D { position: [x, y], uv: [0.0, 0.0], color: left },
            Vertex2D { position: [x + w, y], uv: [1.0, 0.0], color: right },
            Vertex2D { position: [x + w, y + h], uv: [1.0, 1.0], color: right },
            Vertex2D { position: [x, y + h], uv: [0.0, 1.0], color: left },
        ]);
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Draw a vertical gradient rectangle.
    pub fn fill_gradient_v(&mut self, x: f32, y: f32, w: f32, h: f32, top: [f32; 4], bottom: [f32; 4]) {
        let base = self.vertices.len() as u16;
        self.vertices.extend_from_slice(&[
            Vertex2D { position: [x, y], uv: [0.0, 0.0], color: top },
            Vertex2D { position: [x + w, y], uv: [1.0, 0.0], color: top },
            Vertex2D { position: [x + w, y + h], uv: [1.0, 1.0], color: bottom },
            Vertex2D { position: [x, y + h], uv: [0.0, 1.0], color: bottom },
        ]);
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// Draw outlined rectangle.
    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: [f32; 4]) {
        self.fill_rect(x, y, w, thickness, color);           // top
        self.fill_rect(x, y + h - thickness, w, thickness, color); // bottom
        self.fill_rect(x, y, thickness, h, color);             // left
        self.fill_rect(x + w - thickness, y, thickness, h, color); // right
    }

    /// Submit all queued quads to the render pass.
    pub fn flush(&self, device: &wgpu::Device, render_pass: &mut wgpu::RenderPass) {
        if self.indices.is_empty() {
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("2D Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("2D Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        render_pass.set_pipeline(&self.pipeline_solid);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }
}

/// Build an orthographic projection matrix for pixel-perfect 2D rendering.
fn ortho_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}
