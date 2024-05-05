use super::Processor;
use anyhow::Error as AnyError;
use lvl_math::{Quat, Vec3};
use lvl_pmx::Pmx;
use lvl_resource::{
    ModelElement, ModelSource, ModelTransform, ModelVisiblePart, Resource, ResourceKind,
};
use std::path::Path;

pub struct ModelProcessor;

impl Processor for ModelProcessor {
    type Metadata = ();

    fn extension() -> &'static [&'static str] {
        &["pmx"]
    }

    fn process(file: &Path, _metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError> {
        let content = std::fs::read(file)?;
        let pmx: Pmx = Pmx::parse(&content)?;
        let splitted = pmx::split_pmx(file, &pmx);
        let mut resources = splitted.resources;

        resources.push(Resource {
            name: pmx.header.model_name_local.clone(),
            kind: ResourceKind::Model(ModelSource::new(
                0,
                vec![ModelElement {
                    index: 0,
                    name: pmx.header.model_name_local.clone(),
                    parent_index: None,
                    transform: ModelTransform {
                        position: Vec3::ZERO,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    visible_parts: splitted
                        .visible_parts
                        .into_iter()
                        .map(|part| ModelVisiblePart {
                            mesh_name: part.mesh_name.clone(),
                            material_name: part.material_name.clone(),
                        })
                        .collect(),
                }],
            )),
        });

        Ok(resources)
    }
}

mod pmx {
    use crate::processors::{ShaderProcessor, TextureMetadata, TextureProcessor};
    use anyhow::{anyhow, Error as AnyError};
    use log::{error, warn};
    use lvl_math::{Vec3, Vec4};
    use lvl_pmx::{Pmx, PmxIndices, PmxMaterial, PmxTexture, PmxVertex};
    use lvl_resource::{
        MaterialProperty, MaterialPropertyValue, MaterialPropertyValueUniformKind, MaterialSource,
        MeshElement, MeshElementKind, MeshIndexKind, MeshSource, Resource, ResourceKind,
        ShaderSource, TextureElementSamplingMode, TextureElementTextureFormat,
        TextureElementWrappingMode, TextureSource,
    };
    use std::{
        collections::{hash_map::Entry, BTreeSet, HashMap},
        mem::size_of,
        path::Path,
    };
    use wgpu_types::{AddressMode, FilterMode};
    use zerocopy::AsBytes;

    pub struct SplittedPmx {
        pub resources: Vec<Resource>,
        pub visible_parts: Vec<VisiblePart>,
    }

    pub struct VisiblePart {
        pub material_name: String,
        pub mesh_name: String,
    }

    pub fn split_pmx(pmx_path: &Path, pmx: &Pmx) -> SplittedPmx {
        let mut resources = Vec::with_capacity(pmx.materials.len() * 4);
        let mut visible_parts = Vec::with_capacity(pmx.materials.len() * 2);
        let mut texture_names = BTreeSet::new();
        let mut previous_surface_count = 0;

        let textured_shader_name = format!("{}/shader:{}", pmx.header.model_name_local, "textured");
        let textured_shader_source = make_textured_shader_source(&textured_shader_name);

        match textured_shader_source {
            Ok(source) => {
                resources.push(Resource {
                    name: textured_shader_name.clone(),
                    kind: ResourceKind::Shader(source),
                });
            }
            Err(err) => {
                error!(
                    "failed to process shader `{}`; it will be ignored: {}",
                    textured_shader_name, err
                );
            }
        }

        let non_textured_shader_name =
            format!("{}/shader:{}", pmx.header.model_name_local, "non-textured");
        let non_textured_shader_source = make_non_textured_shader_source(&non_textured_shader_name);

        match non_textured_shader_source {
            Ok(source) => {
                resources.push(Resource {
                    name: non_textured_shader_name.clone(),
                    kind: ResourceKind::Shader(source),
                });
            }
            Err(err) => {
                error!(
                    "failed to process shader `{}`; it will be ignored: {}",
                    non_textured_shader_name, err
                );
            }
        }

        for material in &pmx.materials {
            let texture_source_name = if 0 <= material.texture_index.get() {
                Some(format!(
                    "{}/texture:{}",
                    pmx.header.model_name_local,
                    material.texture_index.get()
                ))
            } else {
                None
            };
            let texture_source = match texture_source_name.as_ref() {
                Some(texture_source_name) => {
                    if texture_names.contains(texture_source_name) {
                        None
                    } else {
                        let pmx_texture = &pmx.textures[material.texture_index.get() as usize];
                        let texture_source = make_texture(pmx_path, pmx_texture);

                        match texture_source {
                            Ok(source) => {
                                texture_names.insert(texture_source_name.clone());
                                Some(source)
                            }
                            Err(err) => {
                                warn!(
                                    "failed to process texture `{}`; it will be ignored: {}",
                                    pmx_texture.path, err
                                );
                                None
                            }
                        }
                    }
                }
                None => None,
            };

            let material_source = make_material_source(
                if texture_source_name.is_some() {
                    textured_shader_name.clone()
                } else {
                    non_textured_shader_name.clone()
                },
                texture_source_name.clone(),
                material,
            );
            let mesh_source = make_mesh(
                previous_surface_count,
                material.surface_count as usize,
                &pmx.vertices,
                &pmx.indices,
            );

            let texture_resource = match texture_source {
                Some(source) => Some(Resource {
                    name: texture_source_name.unwrap(),
                    kind: ResourceKind::Texture(source),
                }),
                None => None,
            };
            let material_resource = Resource {
                name: format!(
                    "{}/material:{}",
                    pmx.header.model_name_local, material.name_local
                ),
                kind: ResourceKind::Material(material_source),
            };
            let mesh_resource = Resource {
                name: format!(
                    "{}/mesh:{}",
                    pmx.header.model_name_local, material.name_local
                ),
                kind: ResourceKind::Mesh(mesh_source),
            };

            visible_parts.push(VisiblePart {
                material_name: material_resource.name.clone(),
                mesh_name: mesh_resource.name.clone(),
            });

            if let Some(resource) = texture_resource {
                resources.push(resource);
            }

            resources.push(material_resource);
            resources.push(mesh_resource);

            previous_surface_count += material.surface_count as usize;
        }

        SplittedPmx {
            resources,
            visible_parts,
        }
    }

    fn make_textured_shader_source(shader_display_name: &str) -> Result<ShaderSource, AnyError> {
        let shader_content = include_str!("../../assets/textured.wgsl");
        ShaderProcessor::generate_shader_resource_from_wsgl_content(
            shader_display_name,
            shader_content.to_owned(),
        )
    }

    fn make_non_textured_shader_source(
        shader_display_name: &str,
    ) -> Result<ShaderSource, AnyError> {
        let shader_content = include_str!("../../assets/non-textured.wgsl");
        ShaderProcessor::generate_shader_resource_from_wsgl_content(
            shader_display_name,
            shader_content.to_owned(),
        )
    }

    fn make_material_source(
        shader_name: String,
        texture_name: Option<String>,
        pmx_material: &PmxMaterial,
    ) -> MaterialSource {
        let mut properties = vec![];

        if let Some(texture_name) = texture_name {
            properties.push(MaterialProperty {
                name: "texture".to_owned(),
                value: MaterialPropertyValue::Texture { texture_name },
            });
            properties.push(MaterialProperty {
                name: "texture_sampler".to_owned(),
                value: MaterialPropertyValue::Sampler {
                    address_mode_u: AddressMode::ClampToEdge,
                    address_mode_v: AddressMode::ClampToEdge,
                    address_mode_w: AddressMode::ClampToEdge,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,
                    mipmap_filter: FilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 32.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                },
            });
        }

        properties.push(MaterialProperty {
            name: "diffuse_color".to_owned(),
            value: MaterialPropertyValue::Uniform(MaterialPropertyValueUniformKind::Vec4(
                Vec4::new(
                    pmx_material.diffuse_color.x,
                    pmx_material.diffuse_color.y,
                    pmx_material.diffuse_color.z,
                    pmx_material.diffuse_color.w,
                ),
            )),
        });
        properties.push(MaterialProperty {
            name: "specular_color".to_owned(),
            value: MaterialPropertyValue::Uniform(MaterialPropertyValueUniformKind::Vec3(
                Vec3::new(
                    pmx_material.specular_color.x,
                    pmx_material.specular_color.y,
                    pmx_material.specular_color.z,
                ),
            )),
        });
        properties.push(MaterialProperty {
            name: "specular_strength".to_owned(),
            value: MaterialPropertyValue::Uniform(MaterialPropertyValueUniformKind::Float(
                pmx_material.specular_strength,
            )),
        });
        properties.push(MaterialProperty {
            name: "ambient_color".to_owned(),
            value: MaterialPropertyValue::Uniform(MaterialPropertyValueUniformKind::Vec3(
                Vec3::new(
                    pmx_material.ambient_color.x,
                    pmx_material.ambient_color.y,
                    pmx_material.ambient_color.z,
                ),
            )),
        });
        properties.push(MaterialProperty {
            name: "edge_color".to_owned(),
            value: MaterialPropertyValue::Uniform(MaterialPropertyValueUniformKind::Vec4(
                Vec4::new(
                    pmx_material.edge_color.x,
                    pmx_material.edge_color.y,
                    pmx_material.edge_color.z,
                    pmx_material.edge_color.w,
                ),
            )),
        });
        properties.push(MaterialProperty {
            name: "edge_size".to_owned(),
            value: MaterialPropertyValue::Uniform(MaterialPropertyValueUniformKind::Float(
                pmx_material.edge_size,
            )),
        });

        MaterialSource::new(shader_name, properties)
    }

    fn make_mesh(
        previous_surface_count: usize,
        pmx_surface_count: usize,
        pmx_vertices: &[PmxVertex],
        pmx_indices: &PmxIndices,
    ) -> MeshSource {
        let surface_count = pmx_surface_count as usize;
        let mut indices = Vec::with_capacity(surface_count * 3);
        let mut vertices = Vec::with_capacity(surface_count * 3);
        let mut index_map = HashMap::new();

        for index in &pmx_indices.vertex_indices
            [previous_surface_count..(previous_surface_count + surface_count)]
        {
            let pmx_vertex_index = index.get();
            let vertex_index = match index_map.entry(pmx_vertex_index) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => {
                    let index = vertices.len() as u32;
                    vertices.push(pmx_vertices[pmx_vertex_index as usize].clone());
                    entry.insert(index);
                    index
                }
            };

            indices.push(vertex_index);
        }

        let vertices = Vec::from_iter(
            vertices
                .into_iter()
                .map(|vertex| {
                    vec![
                        vertex.position.x,
                        vertex.position.y,
                        vertex.position.z,
                        vertex.normal.x,
                        vertex.normal.y,
                        vertex.normal.z,
                        vertex.uv.x,
                        vertex.uv.y,
                        vertex.additional_vec4s[0].x,
                        vertex.additional_vec4s[0].y,
                        vertex.additional_vec4s[0].z,
                        vertex.additional_vec4s[0].w,
                        vertex.additional_vec4s[1].x,
                        vertex.additional_vec4s[1].y,
                        vertex.additional_vec4s[1].z,
                        vertex.additional_vec4s[1].w,
                        vertex.additional_vec4s[2].x,
                        vertex.additional_vec4s[2].y,
                        vertex.additional_vec4s[2].z,
                        vertex.additional_vec4s[2].w,
                        vertex.additional_vec4s[3].x,
                        vertex.additional_vec4s[3].y,
                        vertex.additional_vec4s[3].z,
                        vertex.additional_vec4s[3].w,
                    ]
                })
                .flatten(),
        );
        let elements = vec![
            MeshElement {
                name: "position".to_owned(),
                kind: MeshElementKind::Position,
                offset: 0,
            },
            MeshElement {
                name: "normal".to_owned(),
                kind: MeshElementKind::Normal,
                offset: size_of::<[f32; 3]>() as u64,
            },
            MeshElement {
                name: "uv_0_".to_owned(),
                kind: MeshElementKind::TexCoord(0),
                offset: size_of::<[f32; 3]>() as u64 + size_of::<[f32; 3]>() as u64,
            },
            MeshElement {
                name: "additional_0_".to_owned(),
                kind: MeshElementKind::Additional(0),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64,
            },
            MeshElement {
                name: "additional_1_".to_owned(),
                kind: MeshElementKind::Additional(1),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64
                    + size_of::<[f32; 4]>() as u64,
            },
            MeshElement {
                name: "additional_2_".to_owned(),
                kind: MeshElementKind::Additional(2),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64
                    + size_of::<[f32; 4]>() as u64
                    + size_of::<[f32; 4]>() as u64,
            },
            MeshElement {
                name: "additional_3_".to_owned(),
                kind: MeshElementKind::Additional(3),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64
                    + size_of::<[f32; 4]>() as u64
                    + size_of::<[f32; 4]>() as u64
                    + size_of::<[f32; 4]>() as u64,
            },
        ];

        let vertices = vertices.as_bytes().to_vec();
        let indices = indices.as_bytes().to_vec();

        MeshSource::new(
            surface_count as u32,
            vertices,
            indices,
            MeshIndexKind::U32,
            elements,
        )
    }

    fn make_texture(pmx_path: &Path, pmx_texture: &PmxTexture) -> Result<TextureSource, AnyError> {
        let parent_path = match pmx_path.parent() {
            Some(parent_path) => parent_path,
            None => {
                return Err(anyhow!(
                    "failed to get parent path of PMX file path `{}`",
                    pmx_path.display()
                ));
            }
        };

        TextureProcessor::generate_texture_source(
            &parent_path.join(&pmx_texture.path),
            &TextureMetadata {
                texture_format: TextureElementTextureFormat::RGBA8Unorm,
                sampling_mode: Some(TextureElementSamplingMode::Bilinear),
                wrapping_mode_u: Some(TextureElementWrappingMode::Clamp),
                wrapping_mode_v: Some(TextureElementWrappingMode::Clamp),
            },
        )
    }
}
