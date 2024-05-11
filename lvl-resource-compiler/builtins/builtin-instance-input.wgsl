
struct InstanceInput {
  @location(0) model_matrix_col_0: vec4<f32>,
  @location(1) model_matrix_col_1: vec4<f32>,
  @location(2) model_matrix_col_2: vec4<f32>,
  @location(3) model_matrix_col_3: vec4<f32>,
  @location(4) inversed_model_matrix_col_0: vec4<f32>,
  @location(5) inversed_model_matrix_col_1: vec4<f32>,
  @location(6) inversed_model_matrix_col_2: vec4<f32>,
  @location(7) inversed_model_matrix_col_3: vec4<f32>,
};

fn builtin_transform_vertex_to_world_space(instance: InstanceInput, v: vec4<f32>) -> vec4<f32> {
  let model_matrix = mat4x4<f32>(
    instance.model_matrix_col_0,
    instance.model_matrix_col_1,
    instance.model_matrix_col_2,
    instance.model_matrix_col_3
  );

  return model_matrix * v;
}

fn builtin_transform_normal_to_world_space(instance: InstanceInput, v: vec3<f32>) -> vec3<f32> {
  let model_matrix = mat4x4<f32>(
    instance.inversed_model_matrix_col_0,
    instance.inversed_model_matrix_col_1,
    instance.inversed_model_matrix_col_2,
    instance.inversed_model_matrix_col_3
  );
  let transposed = transpose(model_matrix);
  let model_matrix_for_normal = mat3x3<f32>(transposed[0].xyz, transposed[1].xyz, transposed[2].xyz);

  return normalize(model_matrix_for_normal * v);
}
