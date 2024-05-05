
struct Uniform {
  diffuse_color: vec4<f32>,
  specular_color: vec3<f32>,
  specular_strength: f32,
  ambient_color: vec3<f32>,
  light_color: vec3<f32>,
  light_direction: vec3<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniform;
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv_0: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) normal: vec3<f32>,
  @location(1) uv: vec2<f32>,
};

struct FragmentOutput {
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(instance: InstanceInput, vertex: VertexInput) -> VertexOutput {
  let world_pos = builtin_transform_to_world_space(instance, vec4<f32>(vertex.position, 1.0));
  let clip_pos = builtin_transform_to_clip_space(world_pos);

  var out: VertexOutput;
  out.position = clip_pos;
  out.normal = vertex.normal;
  out.uv = vertex.uv_0;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
  // if uniforms.diffuse_color.a <= 0.0 {
  //   discard;
  // }

  var out: FragmentOutput;

  let eye_dir = normalize(in.position.xyz);
  let light_dir = normalize(-uniforms.light_direction);
  let normal = normalize(in.normal);
  let ln = clamp(dot(normal, light_dir) + 0.5, 0.0, 1.0);
  var color = vec3<f32>(0.0);

  let alpha = uniforms.diffuse_color.a;
  let diffuse = uniforms.diffuse_color.rgb * uniforms.light_color;

  color += diffuse;
  color += uniforms.ambient_color;
  color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

  color *= textureSample(texture, texture_sampler, in.uv).rgb;

  var specular = vec3<f32>(0.0);

  if 0 < uniforms.specular_strength {
    let half = normalize(eye_dir + light_dir);
    let specular_light = uniforms.specular_color * uniforms.light_color;
    specular += pow(max(dot(normal, half), 0.0), uniforms.specular_strength) * specular_light;
  }
  
  color += specular;
  
  out.color = vec4<f32>(color, alpha);

  return out;
}
