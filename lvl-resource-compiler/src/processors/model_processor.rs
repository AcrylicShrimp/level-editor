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
        let pmx = Pmx::parse(&content)?;

        let parts = pmx::split_parts(&pmx);
        let mut resources = Vec::with_capacity(1 + parts.len() * 2);

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
                    visible_parts: parts
                        .iter()
                        .map(|part| ModelVisiblePart {
                            mesh_name: part.mesh.name.clone(),
                            material_name: part.material.name.clone(),
                        })
                        .collect(),
                }],
            )),
        });

        for part in parts {
            resources.push(part.material);
            resources.push(part.mesh);
        }

        Ok(resources)
    }
}

mod pmx {
    use lvl_math::{Vec3, Vec4};
    use lvl_pmx::{Pmx, PmxMaterial, PmxSurface, PmxVertex};
    use lvl_resource::{
        MaterialProperty, MaterialPropertyValue, MaterialPropertyValueUniformKind, MaterialSource,
        MeshElement, MeshElementKind, MeshIndexKind, MeshSource, Resource, ResourceKind,
    };
    use std::{
        collections::{hash_map::Entry, HashMap},
        mem::size_of,
    };
    use zerocopy::AsBytes;

    pub struct SplittedPart {
        pub material: Resource,
        pub mesh: Resource,
    }

    pub fn split_parts(pmx: &Pmx) -> Vec<SplittedPart> {
        let mut parts = vec![];
        let mut previous_surface_count = 0;

        for material in &pmx.materials {
            let surface_count = (material.surface_count / 3) as usize;
            let material_source = make_shader_source(&pmx.header.model_name_local, material);
            let mesh_source = make_mesh(
                previous_surface_count,
                surface_count,
                &pmx.vertices,
                &pmx.surfaces,
            );

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
            parts.push(SplittedPart {
                material: material_resource,
                mesh: mesh_resource,
            });

            previous_surface_count += surface_count;
        }

        parts
    }

    fn make_shader_source(pmx_model_name: &str, pmx_material: &PmxMaterial) -> MaterialSource {
        let shader_name = format!("{}/shader:{}", pmx_model_name, pmx_material.name_local);
        let mut properties = vec![];

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
        pmx_surfaces: &[PmxSurface],
    ) -> MeshSource {
        let surface_count = pmx_surface_count as usize;
        let mut indices = Vec::with_capacity(surface_count * 3);
        let mut vertices = Vec::with_capacity(surface_count * 3);
        let mut index_map = HashMap::new();

        for surface in
            &pmx_surfaces[previous_surface_count..(previous_surface_count + surface_count)]
        {
            for vertex_index in surface.vertex_indices {
                let pmx_vertex_index = vertex_index.get();
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
                name: "uv0".to_owned(),
                kind: MeshElementKind::TexCoord(0),
                offset: size_of::<[f32; 3]>() as u64 + size_of::<[f32; 3]>() as u64,
            },
            MeshElement {
                name: "additional_0".to_owned(),
                kind: MeshElementKind::Additional(0),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64,
            },
            MeshElement {
                name: "additional_1".to_owned(),
                kind: MeshElementKind::Additional(1),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64
                    + size_of::<[f32; 4]>() as u64,
            },
            MeshElement {
                name: "additional_2".to_owned(),
                kind: MeshElementKind::Additional(2),
                offset: size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 3]>() as u64
                    + size_of::<[f32; 2]>() as u64
                    + size_of::<[f32; 4]>() as u64
                    + size_of::<[f32; 4]>() as u64,
            },
            MeshElement {
                name: "additional_3".to_owned(),
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

        MeshSource::new(vertices, indices, MeshIndexKind::U32, elements)
    }
}
