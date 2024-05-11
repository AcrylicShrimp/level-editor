
struct BuiltinUniform {
  camera_matrix: mat4x4<f32>,
  camera_position: vec3<f32>,
  view_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> builtin_uniform: BuiltinUniform;

fn builtin_transform_vertex_to_clip_space(v: vec4<f32>) -> vec4<f32> {
  return builtin_uniform.camera_matrix * v;
}

fn builtin_transform_normal_to_view_space(v: vec3<f32>) -> vec3<f32> {
  let transposed = transpose(builtin_uniform.view_matrix);
  let view_matrix_for_normal = mat3x3<f32>(transposed[0].xyz, transposed[1].xyz, transposed[2].xyz);

  return normalize(view_matrix_for_normal * v);
}
