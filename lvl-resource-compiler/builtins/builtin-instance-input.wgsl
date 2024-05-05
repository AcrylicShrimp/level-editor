
struct InstanceInput {
  @location(0) model_matrix_col_0: vec4<f32>,
  @location(1) model_matrix_col_1: vec4<f32>,
  @location(2) model_matrix_col_2: vec4<f32>,
  @location(3) model_matrix_col_3: vec4<f32>,
};

fn builtin_transform_to_world_space(instance: InstanceInput, v: vec4<f32>) -> vec4<f32> {
  let model_matrix = mat4x4<f32>(
    instance.model_matrix_col_0,
    instance.model_matrix_col_1,
    instance.model_matrix_col_2,
    instance.model_matrix_col_3
  );

  return model_matrix * v;
}
