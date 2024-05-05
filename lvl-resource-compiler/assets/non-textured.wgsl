
@group(0) @binding(0) var<uniform> diffuse_color: vec4<f32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv_0: vec2<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
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
  out.color = diffuse_color;
  out.uv = vertex.uv_0;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
  var out: FragmentOutput;
  out.color = vec4<f32>(in.uv.x, 0.0, in.uv.y, 1.0);
  return out;
}
