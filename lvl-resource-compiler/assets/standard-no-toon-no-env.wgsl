
struct Uniform {
  diffuse_color: vec4<f32>,
  specular_color: vec3<f32>,
  specular_strength: f32,
  ambient_color: vec3<f32>,
  texture_tint_color: vec4<f32>,
  light_color: vec3<f32>,
  light_direction: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniform;
@group(0) @binding(1) var<storage, read> morph_coefficients: array<f32, 128>;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
@group(1) @binding(2) var vertex_morph_index_texture: texture_2d<u32>;
@group(1) @binding(3) var uv_morph_index_texture: texture_2d<u32>;
@group(1) @binding(4) var vertex_displacement_texture: texture_2d<f32>;
@group(1) @binding(5) var uv_displacement_texture: texture_2d<f32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) vertex_morph_index_start: u32,
  @location(4) vertex_morph_count: u32,
  @location(5) uv_morph_index_start: u32,
  @location(6) uv_morph_count: u32,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) world_position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
};

struct FragmentOutput {
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(instance: InstanceInput, vertex: VertexInput) -> VertexOutput {
  var position = vertex.position;
  var uv = vertex.uv;

  let vertex_morph_index_texture_size = textureDimensions(vertex_morph_index_texture).x;
  let vertex_displacement_texture_size = textureDimensions(vertex_displacement_texture).x;

  for (var i = 0u; i < vertex.vertex_morph_count; i += 1u) {
    let morph_index_uv_base = vertex.vertex_morph_index_start + i;
    let morph_index_uv = vec2<u32>(
      morph_index_uv_base % vertex_morph_index_texture_size,
      morph_index_uv_base / vertex_morph_index_texture_size,
    );

    let morph_index = textureLoad(vertex_morph_index_texture, morph_index_uv, 0);
    let coefficient = morph_coefficients[morph_index.x];

    let displacement_uv_base = morph_index.y;
    let displacement_uv = vec2<u32>(
      displacement_uv_base % vertex_displacement_texture_size,
      displacement_uv_base / vertex_displacement_texture_size,
    );

    let displacement = textureLoad(vertex_displacement_texture, displacement_uv, 0);
    position = position + coefficient * displacement.xyz;
  }

  let uv_morph_index_texture_size = textureDimensions(uv_morph_index_texture).x;
  let uv_displacement_texture_size = textureDimensions(uv_displacement_texture).x;

  for (var i = 0u; i < vertex.uv_morph_count; i += 1u) {
    let morph_index_uv_base = vertex.uv_morph_index_start + i;
    let morph_index_uv = vec2<u32>(
      morph_index_uv_base % uv_morph_index_texture_size,
      morph_index_uv_base / uv_morph_index_texture_size,
    );

    let morph_index = textureLoad(uv_morph_index_texture, morph_index_uv, 0);
    let coefficient = morph_coefficients[morph_index.y];

    let displacement_uv_base = morph_index.z;
    let displacement_uv = vec2<u32>(
      displacement_uv_base % uv_displacement_texture_size,
      displacement_uv_base / uv_displacement_texture_size,
    );

    let displacement = textureLoad(uv_displacement_texture, displacement_uv, 0);

    if (morph_index.x == 0) {
      uv = uv + coefficient * displacement.xy;
    }
  }

  let world_pos = builtin_transform_vertex_to_world_space(instance, vec4<f32>(position, 1.0));
  let clip_pos = builtin_transform_vertex_to_clip_space(world_pos);
  let normal = builtin_transform_normal_to_world_space(instance, vertex.normal);

  var out: VertexOutput;
  out.position = clip_pos;
  out.world_position = world_pos.xyz;
  out.normal = normal;
  out.uv = uv;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
  let eye_dir = normalize(builtin_uniform.camera_position - in.world_position);
  let light_dir = normalize(-uniforms.light_direction);
  let normal = normalize(in.normal);

  // half lambert
  var ln = dot(normal, light_dir);
  ln = clamp(ln + 0.5, 0.0, 1.0);

  // ambient term
  var color = uniforms.ambient_color;
  var alpha = uniforms.diffuse_color.a;

  // diffuse term
  let diffuse_color = uniforms.diffuse_color.rgb * uniforms.light_color;
  color += diffuse_color;
  color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

  // texture term
  let tex_color = textureSample(texture, texture_sampler, in.uv);
  color *= tex_color.rgb * uniforms.texture_tint_color.rgb;
  alpha *= tex_color.a;

  // specular term
  var specular_color = vec3<f32>(0.0);
  if (0.0 < uniforms.specular_strength) {
    let half = normalize(light_dir + eye_dir);
    let color = uniforms.specular_color * uniforms.light_color;
    specular_color += color * pow(max(dot(normal, half), 0.0), uniforms.specular_strength);
  }
  color += specular_color;

  // final
  var out: FragmentOutput;
  out.color = vec4<f32>(color, alpha);
  return out;
}
