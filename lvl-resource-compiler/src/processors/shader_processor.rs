mod reflection;
mod template;

use self::{
    reflection::{inspect_bindings, inspect_locations},
    template::expand_wgsl_shader_content,
};
use super::Processor;
use anyhow::{anyhow, Context, Error as AnyError};
use lvl_resource::{Resource, ResourceKind, ShaderRenderType, ShaderSource};
use naga::{Module, ShaderStage};
use std::path::Path;

pub struct ShaderProcessor;

impl ShaderProcessor {
    pub fn generate_shader_resource_from_wsgl_content(
        display_name: &str,
        content: String,
    ) -> Result<ShaderSource, AnyError> {
        let expanded = expand_wgsl_shader_content(&content)?;
        let module = naga::front::wgsl::parse_str(&expanded.content).with_context(|| {
            format!(
                "failed to parse the file `{}` as a wgsl shader",
                display_name
            )
        })?;

        Self::generate_shader_resource_from_module(
            display_name,
            expanded.content,
            &module,
            expanded.builtin_uniform_bind_group,
            expanded.instance_input_typename.as_deref(),
        )
    }

    fn generate_shader_resource_from_module(
        display_name: &str,
        content: String,
        module: &Module,
        builtin_uniform_bind_group: Option<u32>,
        instance_input_typename: Option<&str>,
    ) -> Result<ShaderSource, AnyError> {
        let mut vertex_entry_point = None;
        let mut fragment_entry_point = None;

        for entry_point in &module.entry_points {
            match entry_point.stage {
                ShaderStage::Vertex => {
                    vertex_entry_point = Some(entry_point.name.clone());
                }
                ShaderStage::Fragment => {
                    fragment_entry_point = Some(entry_point.name.clone());
                }
                ShaderStage::Compute => {
                    continue;
                }
            }
        }

        let vertex_entry_point = match vertex_entry_point {
            Some(entry_point) => entry_point,
            None => {
                return Err(anyhow!(
                    "the shader `{}` does not contain a vertex entry point",
                    display_name
                ));
            }
        };
        let fragment_entry_point = match fragment_entry_point {
            Some(entry_point) => entry_point,
            None => {
                return Err(anyhow!(
                    "the shader `{}` does not contain a fragment entry point",
                    display_name
                ));
            }
        };

        let bindings = inspect_bindings(&module, builtin_uniform_bind_group);
        let locations = inspect_locations(display_name, &module, instance_input_typename);

        Ok(ShaderSource::new(
            ShaderRenderType::Opaque,
            content,
            vertex_entry_point,
            fragment_entry_point,
            builtin_uniform_bind_group,
            bindings,
            locations,
        ))
    }
}

impl Processor for ShaderProcessor {
    type Metadata = ();

    fn extension() -> &'static [&'static str] {
        &["wgsl"]
    }

    fn process(file: &Path, _metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError> {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        let content = std::fs::read_to_string(file)?;
        let source = Self::generate_shader_resource_from_wsgl_content(&name, content)
            .with_context(|| format!("failed to process the file `{}` as a wgsl shader", name))?;

        Ok(vec![Resource {
            name,
            kind: ResourceKind::Shader(source),
        }])
    }
}
