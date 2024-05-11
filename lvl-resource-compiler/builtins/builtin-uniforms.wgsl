
struct BuiltinUniform {
  camera_matrix: mat4x4<f32>,
  camera_position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> builtin_uniform: BuiltinUniform;

fn builtin_transform_to_clip_space(v: vec4<f32>) -> vec4<f32> {
  return builtin_uniform.camera_matrix * v;
}
