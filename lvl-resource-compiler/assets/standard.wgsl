
@group(0) @binding(0) var<uniform> diffuse_color: vec4<f32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv0: vec2<f32>,
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
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = vec4<f32>(vertex.position, 1.0);
  out.color = diffuse_color;
  out.uv = vertex.uv0;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
  var out: FragmentOutput;
  out.color = in.color;
  return out;
}
