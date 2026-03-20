// 3D world rendering shader with SD/HD mode support.
// SD mode: Short draw distance, flat lighting, no specular
// HD mode: Extended draw distance, specular highlights, ambient occlusion, enhanced fog

struct Camera {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
};

struct RenderMode {
    fog_distance: f32,     // SD=1600, HD=6400
    fog_density: f32,      // SD=1.0, HD=0.5
    ambient_strength: f32, // SD=0.5, HD=0.35
    specular_power: f32,   // SD=0, HD=32
    
    fog_color: vec3<f32>,
    hd_mode: f32,          // 0.0=SD, 1.0=HD
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
@group(0) @binding(1) var<uniform> render_mode: RenderMode;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.normal = in.normal;
    out.color = in.color;

    // Fog: distance-based — shorter in SD, longer in HD
    let dist = distance(camera.camera_pos, in.position);
    let fog_start = render_mode.fog_distance * 0.6;
    let fog_range = render_mode.fog_distance - fog_start;
    out.fog_factor = clamp((dist - fog_start) / fog_range, 0.0, 1.0) * render_mode.fog_density;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let normal = normalize(in.normal);
    
    // Diffuse lighting
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // Ambient
    let ambient = render_mode.ambient_strength;
    
    // Specular (HD only — specular_power is 0 in SD)
    var specular = 0.0;
    if (render_mode.specular_power > 0.0) {
        let view_dir = normalize(camera.camera_pos - in.world_pos);
        let reflect_dir = reflect(-light_dir, normal);
        let spec_angle = max(dot(view_dir, reflect_dir), 0.0);
        specular = pow(spec_angle, render_mode.specular_power) * 0.3;
    }
    
    let lighting = ambient + diffuse * (1.0 - ambient) + specular;
    var color = in.color.rgb * lighting;

    // HD: Subtle ambient occlusion based on height
    if (render_mode.hd_mode > 0.5) {
        let height_ao = clamp(in.world_pos.y / 200.0 + 0.3, 0.0, 1.0);
        color *= mix(0.7, 1.0, height_ao);
    }

    // Fog blend
    color = mix(color, render_mode.fog_color, in.fog_factor);

    // HD: Subtle tone mapping for richer colors
    if (render_mode.hd_mode > 0.5) {
        color = color / (color + vec3<f32>(1.0));  // Reinhard tone mapping
        color = pow(color, vec3<f32>(1.0 / 1.1));  // Slight gamma boost
    }

    return vec4<f32>(color, in.color.a);
}
