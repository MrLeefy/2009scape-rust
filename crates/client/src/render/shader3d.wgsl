// 3D world rendering shader.
// Handles terrain tiles, objects, and models with per-vertex coloring.

struct Camera {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) fog_factor: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.normal = in.normal;
    out.color = in.color;

    // Fog: distance-based, matching RS feel
    let dist = distance(camera.camera_pos, in.position);
    out.fog_factor = clamp(dist / 3200.0, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.4;
    let diffuse = max(dot(normalize(in.normal), light_dir), 0.0) * 0.6;
    let lighting = ambient + diffuse;

    var color = in.color.rgb * lighting;

    // Fog color (dark blue, RS-style)
    let fog_color = vec3<f32>(0.05, 0.08, 0.15);
    color = mix(color, fog_color, in.fog_factor);

    return vec4<f32>(color, in.color.a);
}
