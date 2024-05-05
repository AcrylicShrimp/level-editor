use anyhow::Error as AnyError;
use naga::{
    back::wgsl::WriterFlags,
    valid::{Capabilities, ValidationFlags},
    Binding, Module, ShaderStage, Type, TypeInner,
};

const BUILTIN_UNIFORMS: &str = include_str!("../../../builtins/builtin-uniforms.wgsl");
const BUILTIN_INSTANCE_INPUT: &str = include_str!("../../../builtins/builtin-instance-input.wgsl");

#[derive(Debug, Clone)]
pub struct ExpandedShaderContent {
    pub content: String,
    pub builtin_uniform_bind_group: Option<u32>,
    pub instance_input_typename: Option<String>,
}

pub fn expand_wgsl_shader_content(content: &str) -> Result<ExpandedShaderContent, AnyError> {
    const INSTANCE_INPUT_TYPENAME: &str = "InstanceInput";

    let expanded_source = format!(
        "{}\n{}\n{}",
        BUILTIN_UNIFORMS, BUILTIN_INSTANCE_INPUT, content
    );
    let mut module = naga::front::wgsl::parse_str(&expanded_source)?;

    increase_custom_binding_groups(&mut module);
    increase_custom_locations(&mut module, INSTANCE_INPUT_TYPENAME);

    let mut validator = naga::valid::Validator::new(ValidationFlags::all(), Capabilities::all());
    let module_info = validator.validate(&module)?;

    let transformed = naga::back::wgsl::write_string(&module, &module_info, WriterFlags::empty())?;

    Ok(ExpandedShaderContent {
        content: transformed,
        builtin_uniform_bind_group: Some(0),
        instance_input_typename: Some(INSTANCE_INPUT_TYPENAME.to_owned()),
    })
}

fn increase_custom_binding_groups(module: &mut Module) {
    const BINDING_GROUP_OFFSET: u32 = 1;

    for (_, variable) in module.global_variables.iter_mut() {
        if variable.name.as_deref() == Some("builtin_uniform") {
            continue;
        }

        let binding = match &mut variable.binding {
            Some(binding) => binding,
            None => {
                continue;
            }
        };

        binding.group += BINDING_GROUP_OFFSET;
    }
}

fn increase_custom_locations(module: &mut Module, instance_input_typename: &str) {
    const LOCATION_OFFSET: u32 = 4;

    let vertex_entry_point = module
        .entry_points
        .iter()
        .find(|entry_point| entry_point.stage == ShaderStage::Vertex);

    let vertex_entry_point = match vertex_entry_point {
        Some(vertex_entry_point) => vertex_entry_point,
        None => {
            return;
        }
    };

    for argument in &vertex_entry_point.function.arguments {
        let ty = &module.types[argument.ty];

        if ty.name.as_deref() == Some(instance_input_typename) {
            continue;
        }

        let (mut members, span) = match &ty.inner {
            TypeInner::Struct { members, span } => (members.clone(), *span),
            _ => {
                return;
            }
        };

        for member in &mut members {
            let binding = match &mut member.binding {
                Some(binding) => binding,
                None => {
                    continue;
                }
            };

            match binding {
                Binding::BuiltIn(_) => {
                    continue;
                }
                Binding::Location { location, .. } => {
                    *location += LOCATION_OFFSET;
                }
            }
        }

        module.types.replace(
            argument.ty,
            Type {
                name: ty.name.clone(),
                inner: TypeInner::Struct { members, span },
            },
        );
    }
}
