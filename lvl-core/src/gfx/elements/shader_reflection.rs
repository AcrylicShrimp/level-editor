use lvl_resource::ShaderSource;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ShaderReflection {
    pub vertex_entry_point: String,
    pub fragment_entry_point: String,
    pub locations: BTreeMap<String, u32>,
    pub builtin_uniform_bind_group: Option<u32>,
}

impl ShaderReflection {
    pub fn from_shader_source(source: &ShaderSource) -> Self {
        Self {
            vertex_entry_point: source.vs_main().to_owned(),
            fragment_entry_point: source.fs_main().to_owned(),
            locations: source.locations().clone(),
            builtin_uniform_bind_group: source.builtin_uniform_bind_group(),
        }
    }
}
