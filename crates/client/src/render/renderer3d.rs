//! 3D world rendering pipeline — terrain, objects, and models.

use wgpu::util::DeviceExt;
use super::camera::Camera3D;

/// A single vertex for 3D rendering.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex3D {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex3D>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
            wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
            wgpu::VertexAttribute { offset: 24, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
        ],
    };
}

/// Camera uniform buffer data.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding: f32,
}

/// 3D world renderer.
pub struct Renderer3D {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    terrain_vertex_buffer: wgpu::Buffer,
    terrain_index_buffer: wgpu::Buffer,
    terrain_index_count: u32,
    depth_texture: wgpu::TextureView,
}

impl Renderer3D {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader3d.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("3D BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex3D::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let camera_uniform = CameraUniform {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0; 3],
            _padding: 0.0,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera BG"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Generate terrain
        let (vertices, indices) = generate_terrain(50, 50);
        let terrain_index_count = indices.len() as u32;

        let terrain_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain VB"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let terrain_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain IB"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let depth_texture = create_depth_texture(device, width, height);

        Renderer3D {
            pipeline,
            camera_buffer,
            camera_bind_group,
            terrain_vertex_buffer,
            terrain_index_buffer,
            terrain_index_count,
            depth_texture,
        }
    }

    /// Update camera uniform.
    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera3D, aspect: f32) {
        let view_proj = camera.view_proj(aspect);
        let pos = camera.position();
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: pos.into(),
            _padding: 0.0,
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Resize depth texture.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.depth_texture = create_depth_texture(device, width, height);
    }

    /// Render the 3D world.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.terrain_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.terrain_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.terrain_index_count, 0, 0..1);
    }

    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_texture
    }
}

/// Generate a terrain mesh grid with RS-style heightmap variation.
fn generate_terrain(width: usize, height: usize) -> (Vec<Vertex3D>, Vec<u32>) {
    let tile_size = 128.0; // RS uses 128 units per tile
    let mut vertices = Vec::with_capacity((width + 1) * (height + 1));
    let mut indices = Vec::with_capacity(width * height * 6);

    for z in 0..=height {
        for x in 0..=width {
            let wx = x as f32 * tile_size;
            let wz = z as f32 * tile_size;

            // Procedural heightmap (sine waves mixing to look terrain-like)
            let h1 = (wx * 0.01).sin() * 40.0;
            let h2 = (wz * 0.015).cos() * 30.0;
            let h3 = ((wx + wz) * 0.008).sin() * 60.0;
            let h4 = (wx * 0.03).cos() * (wz * 0.02).sin() * 20.0;
            let height_val = h1 + h2 + h3 + h4;

            // Color based on height (RS-style: green grass, brown dirt, gray rock)
            let color = if height_val < -30.0 {
                [0.2, 0.3, 0.5, 1.0]  // water (blue-ish)
            } else if height_val < 0.0 {
                [0.35, 0.5, 0.2, 1.0]  // dark grass
            } else if height_val < 30.0 {
                [0.45, 0.6, 0.25, 1.0] // grass
            } else if height_val < 60.0 {
                [0.5, 0.4, 0.25, 1.0]  // dirt/brown
            } else {
                [0.55, 0.52, 0.48, 1.0] // rock/gray
            };

            vertices.push(Vertex3D {
                position: [wx, height_val, wz],
                normal: [0.0, 1.0, 0.0], // will be recalculated
                color,
            });
        }
    }

    // Generate indices and calculate normals
    let stride = width + 1;
    for z in 0..height {
        for x in 0..width {
            let tl = (z * stride + x) as u32;
            let tr = tl + 1;
            let bl = ((z + 1) * stride + x) as u32;
            let br = bl + 1;

            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    // Recalculate normals (face normals averaged at vertices)
    let mut normal_accum = vec![[0.0f32; 3]; vertices.len()];
    for tri in indices.chunks(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let v0 = glam::Vec3::from(vertices[i0].position);
        let v1 = glam::Vec3::from(vertices[i1].position);
        let v2 = glam::Vec3::from(vertices[i2].position);
        let normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();
        for &i in &[i0, i1, i2] {
            normal_accum[i][0] += normal.x;
            normal_accum[i][1] += normal.y;
            normal_accum[i][2] += normal.z;
        }
    }
    for (i, n) in normal_accum.iter().enumerate() {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(0.001);
        vertices[i].normal = [n[0] / len, n[1] / len, n[2] / len];
    }

    (vertices, indices)
}

fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d { width: width.max(1), height: height.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
