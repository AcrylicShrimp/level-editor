use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    num::NonZeroU64,
};

use log::warn;
use lvl_resource::{ShaderBindingElement, ShaderBindingElementKind};
use naga::{
    AddressSpace, ArraySize, Binding, ImageClass, ImageDimension, Module, ScalarKind, ShaderStage,
    Type, TypeInner, VectorSize,
};
use wgpu_types::{SamplerBindingType, TextureSampleType, TextureViewDimension};

pub fn inspect_bindings(
    module: &Module,
    builtin_uniform_bind_group: Option<u32>,
) -> Vec<ShaderBindingElement> {
    let mut bindings = Vec::with_capacity(module.global_variables.len());

    for (_, variable) in module.global_variables.iter() {
        let name = match &variable.name {
            Some(name) => name,
            None => {
                continue;
            }
        };

        let (group, binding) = match &variable.binding {
            Some(binding) => (binding.group, binding.binding),
            None => {
                continue;
            }
        };

        if Some(group) == builtin_uniform_bind_group {
            continue;
        }

        let kind = match variable.space {
            AddressSpace::Uniform | AddressSpace::Handle => {
                match shader_ty_to_binding_element_kind(module, &module.types[variable.ty]) {
                    Some(kind) => kind,
                    None => {
                        continue;
                    }
                }
            }
            AddressSpace::Function
            | AddressSpace::Private
            | AddressSpace::WorkGroup
            | AddressSpace::Storage { .. }
            | AddressSpace::PushConstant => {
                continue;
            }
        };

        bindings.push(ShaderBindingElement {
            name: name.clone(),
            group,
            binding,
            kind,
        });
    }

    bindings
}

fn shader_ty_to_binding_element_kind(
    module: &Module,
    ty: &Type,
) -> Option<ShaderBindingElementKind> {
    match &ty.inner {
        TypeInner::Scalar(_)
        | TypeInner::Vector { .. }
        | TypeInner::Matrix { .. }
        | TypeInner::Atomic(_)
        | TypeInner::Array { .. }
        | TypeInner::Struct { .. } => {
            resolve_shader_ty_size(module, ty).map(|size| ShaderBindingElementKind::Buffer { size })
        }
        TypeInner::Pointer { .. } | TypeInner::ValuePointer { .. } => None,
        TypeInner::Image {
            dim,
            arrayed,
            class,
        } => {
            if *arrayed {
                return None;
            }

            let view_dimension = match dim {
                ImageDimension::D1 => TextureViewDimension::D1,
                ImageDimension::D2 => TextureViewDimension::D2,
                ImageDimension::D3 => TextureViewDimension::D3,
                ImageDimension::Cube => TextureViewDimension::Cube,
            };

            let (sample_type, multisampled) = match class {
                ImageClass::Sampled { kind, multi } => {
                    let sample_type = match kind {
                        ScalarKind::Sint => TextureSampleType::Sint,
                        ScalarKind::Uint => TextureSampleType::Uint,
                        ScalarKind::Float => TextureSampleType::Float { filterable: true },
                        ScalarKind::Bool | ScalarKind::AbstractInt | ScalarKind::AbstractFloat => {
                            return None;
                        }
                    };
                    (sample_type, *multi)
                }
                ImageClass::Depth { multi } => (TextureSampleType::Depth, *multi),
                ImageClass::Storage { .. } => {
                    return None;
                }
            };

            Some(ShaderBindingElementKind::Texture {
                sample_type,
                view_dimension,
                multisampled,
            })
        }
        TypeInner::Sampler { comparison } => Some(ShaderBindingElementKind::Sampler {
            binding_type: if *comparison {
                SamplerBindingType::Comparison
            } else {
                SamplerBindingType::Filtering
            },
        }),
        TypeInner::AccelerationStructure => None,
        TypeInner::RayQuery => None,
        TypeInner::BindingArray { .. } => {
            // unsupported
            None
        }
    }
}

fn resolve_shader_ty_size(module: &Module, ty: &Type) -> Option<NonZeroU64> {
    fn aligned_size(size: u64, alignment: u64) -> u64 {
        (size + alignment - 1) / alignment * alignment
    }

    fn parse_array_size(size: ArraySize) -> Option<u32> {
        let size = match size {
            ArraySize::Constant(constant) => constant,
            _ => return None,
        };
        Some(size.get())
    }

    match &ty.inner {
        TypeInner::Scalar(scalar) => {
            let size = aligned_size(scalar.width as u64, 16);
            NonZeroU64::new(size)
        }
        TypeInner::Vector { size, scalar } => {
            let vector_size = match size {
                VectorSize::Bi => 2,
                VectorSize::Tri => 3,
                VectorSize::Quad => 4,
            };
            let size = aligned_size(vector_size * scalar.width as u64, 16);
            NonZeroU64::new(size)
        }
        TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => {
            let vector_size = match columns {
                VectorSize::Bi => 2,
                VectorSize::Tri => 3,
                VectorSize::Quad => 4,
            };
            let row_count = match rows {
                VectorSize::Bi => 2,
                VectorSize::Tri => 3,
                VectorSize::Quad => 4,
            };
            let size = aligned_size(vector_size * scalar.width as u64, 16) * row_count;
            NonZeroU64::new(size)
        }
        TypeInner::Atomic(scalar) => {
            let size = aligned_size(scalar.width as u64, 16);
            NonZeroU64::new(size)
        }
        TypeInner::Pointer { .. } => None,
        TypeInner::ValuePointer { .. } => None,
        TypeInner::Array { size, stride, .. } => {
            let array_size = match parse_array_size(*size) {
                Some(size) => size,
                None => {
                    return None;
                }
            };
            let size = aligned_size(*stride as u64 * array_size as u64, 16);
            NonZeroU64::new(size)
        }
        TypeInner::Struct { span, .. } => {
            let size = aligned_size(*span as u64, 16);
            NonZeroU64::new(size)
        }
        TypeInner::Image { .. } => None,
        TypeInner::Sampler { .. } => None,
        TypeInner::AccelerationStructure => None,
        TypeInner::RayQuery => None,
        TypeInner::BindingArray { base, size } => {
            let base_size = match resolve_shader_ty_size(module, &module.types[*base]) {
                Some(base_size) => base_size,
                None => return None,
            };
            let size = match parse_array_size(*size) {
                Some(size) => size,
                None => {
                    return None;
                }
            };
            NonZeroU64::new(base_size.get() * size as u64)
        }
    }
}

pub fn inspect_locations(
    display_name: &str,
    module: &Module,
    instance_input_typename: Option<&str>,
) -> BTreeMap<String, u32> {
    let mut locations = BTreeSet::new();
    let mut location_map = BTreeMap::new();

    let vertex_entry_point = module
        .entry_points
        .iter()
        .find(|entry_point| entry_point.stage == ShaderStage::Vertex);

    let vertex_entry_point = match vertex_entry_point {
        Some(vertex_entry_point) => vertex_entry_point,
        None => {
            return location_map;
        }
    };

    let mut process_location = |name: &str, binding: Option<&Binding>| {
        let binding = match binding {
            Some(binding) => binding,
            None => {
                warn!(
                    "the shader `{}` has an unbound vertex input `{}`; it will be ignored.",
                    display_name, name
                );
                return;
            }
        };

        match binding {
            Binding::BuiltIn(_) => {
                return;
            }
            Binding::Location { location, .. } => {
                if locations.contains(location) {
                    warn!(
                        "the shader `{}` has duplicated location at `{}`; later ones will be ignored.",
                        display_name,
                        location
                    );
                    return;
                }

                match location_map.entry(name.to_owned()) {
                    Entry::Vacant(entry) => {
                        entry.insert(*location);
                        locations.insert(*location);
                    }
                    Entry::Occupied(_) => {
                        warn!(
                            "the shader `{}` has duplicated vertex input `{}`; later ones will be ignored.",
                            display_name,
                            name
                        );
                        return;
                    }
                }
            }
        }
    };

    for argument in &vertex_entry_point.function.arguments {
        let ty = &module.types[argument.ty];

        if ty.name.as_deref() == instance_input_typename {
            continue;
        }

        if let TypeInner::Struct { members, .. } = &ty.inner {
            for member in members {
                let name = match &member.name {
                    Some(name) => name,
                    None => {
                        warn!(
                            "the shader `{}` has an unnamed vertex input; it will be ignored.",
                            display_name
                        );
                        continue;
                    }
                };

                process_location(name, member.binding.as_ref());
            }
        } else {
            let name = match &argument.name {
                Some(name) => name,
                None => {
                    warn!(
                        "the shader `{}` has an unnamed vertex input; it will be ignored.",
                        display_name
                    );
                    continue;
                }
            };

            process_location(name, argument.binding.as_ref());
        }
    }

    location_map
}
