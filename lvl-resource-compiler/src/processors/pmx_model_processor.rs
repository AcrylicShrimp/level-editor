use super::{Processor, ShaderProcessor, TextureMetadata, TextureProcessor};
use anyhow::{anyhow, Error as AnyError};
use log::{error, warn};
use lvl_math::{Vec3, Vec4};
use lvl_pmx::{
    Pmx, PmxIndices, PmxMaterial, PmxMaterialEnvironmentBlendMode, PmxMaterialToonMode, PmxMorph,
    PmxMorphOffset, PmxMorphOffsetMaterialOffsetMode, PmxTexture, PmxVertex, PmxVertexDeformKind,
};
use lvl_resource::{
    MaterialProperty, MaterialPropertyUniformValue, MaterialPropertyValue, MaterialRenderState,
    MaterialRenderType, MaterialSource, PmxModelElement, PmxModelIndexKind, PmxModelMorph,
    PmxModelMorphGroupElement, PmxModelMorphKind, PmxModelMorphMaterialElement,
    PmxModelMorphMaterialOffsetMode, PmxModelSource, PmxModelVertexLayoutElement,
    PmxModelVertexLayoutElementKind, Resource, ResourceKind, TextureElement,
    TextureElementSamplingMode, TextureElementSize, TextureElementTextureFormat,
    TextureElementWrappingMode, TextureKind, TextureSource,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    mem::size_of,
    path::Path,
};
use wgpu_types::{AddressMode, FilterMode};
use zerocopy::{ByteOrder, LittleEndian};

#[derive(Deserialize, Debug, Clone)]
pub struct PmxModelMetadata {
    pub material_descriptions: BTreeMap<String, PmxModelMaterialDescription>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PmxModelMaterialDescription {
    pub render_type: MaterialRenderType,
}

pub struct PmxModelProcessor;

impl Processor for PmxModelProcessor {
    type Metadata = PmxModelMetadata;

    fn extension() -> &'static [&'static str] {
        &["pmx"]
    }

    fn process(file: &Path, metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError> {
        let content = std::fs::read(file)?;
        let pmx = Pmx::parse(&content)?;

        let shader_name = format!("{}/shader:{}", pmx.header.model_name_local, "standard");

        let pmx_material_namer = |pmx_material: &PmxMaterial| -> String {
            format!(
                "{}/material:{}",
                pmx.header.model_name_local, pmx_material.name_local
            )
        };
        let pmx_shader_namer = |_pmx_material: &PmxMaterial| -> String { shader_name.clone() };
        let pmx_texture_namer = |pmx_texture: &PmxTexture| -> String {
            format!(
                "{}/texture:{}",
                pmx.header.model_name_local, pmx_texture.path
            )
        };
        let pmx_internal_toon_texture_namer = |index: u8| -> String {
            format!(
                "{}/toon_texture:toon{:0>2}.bmp",
                pmx.header.model_name_local, index
            )
        };

        let morph_data = make_morph_data(
            &pmx.header.model_name_local,
            pmx.vertices.len() as u32,
            &pmx.morphs,
        );
        let vertex_morph_index_texture_name = format!(
            "{}/morph-texture:{}",
            pmx.header.model_name_local, "vertex-morph-index"
        );
        let uv_morph_index_texture_name = format!(
            "{}/morph-texture:{}",
            pmx.header.model_name_local, "uv-morph-index"
        );
        let vertex_displacement_texture_name = format!(
            "{}/morph-texture:{}",
            pmx.header.model_name_local, "vertex-displacement"
        );
        let uv_displacement_texture_name = format!(
            "{}/morph-texture:{}",
            pmx.header.model_name_local, "uv-displacement"
        );

        let (vertex_data, vertex_layout) =
            make_vertex_data(&pmx.vertices, morph_data.vertex_attributes);
        let (index_data, index_kind, elements) =
            make_index_data(pmx_material_namer, &pmx.materials, &pmx.indices);

        let pmx_model = PmxModelSource::new(
            vertex_data,
            vertex_layout,
            index_data,
            index_kind,
            elements,
            morph_data.morphs,
            vertex_morph_index_texture_name.clone(),
            uv_morph_index_texture_name.clone(),
            vertex_displacement_texture_name.clone(),
            uv_displacement_texture_name.clone(),
        );
        let pmx_model_resource = Resource {
            name: pmx.header.model_name_local.clone(),
            kind: ResourceKind::PmxModel(pmx_model),
        };

        let mut materials = Vec::with_capacity(pmx.materials.len());

        for pmx_material in &pmx.materials {
            let render_type = metadata
                .and_then(|metadata| metadata.material_descriptions.get(&pmx_material.name_local))
                .map(|description| description.render_type)
                .unwrap_or(MaterialRenderType::Opaque);

            let source = make_material_source(
                pmx_shader_namer,
                pmx_texture_namer,
                pmx_internal_toon_texture_namer,
                render_type,
                pmx_material,
                &pmx.textures,
                &vertex_morph_index_texture_name,
                &uv_morph_index_texture_name,
                &vertex_displacement_texture_name,
                &uv_displacement_texture_name,
            );
            let resource = Resource {
                name: pmx_material_namer(pmx_material),
                kind: ResourceKind::Material(source),
            };

            materials.push(resource);
        }

        let mut textures = Vec::with_capacity(pmx.textures.len() + 10);

        for pmx_texture in &pmx.textures {
            let source = match make_texture_source(file, pmx_texture) {
                Ok(source) => source,
                Err(err) => {
                    error!(
                        "failed to process texture `{}`; it will be ignored: {}",
                        pmx_texture.path, err
                    );
                    continue;
                }
            };
            let resource = Resource {
                name: pmx_texture_namer(pmx_texture),
                kind: ResourceKind::Texture(source),
            };

            textures.push(resource);
        }

        for index in 1..10 {
            let source = match make_internal_toon_texture_source(file, index) {
                Ok(source) => source,
                Err(err) => {
                    error!(
                        "failed to process internal toon texture index `{}`; it will be ignored: {}",
                        index, err
                    );
                    continue;
                }
            };
            let resource = Resource {
                name: pmx_internal_toon_texture_namer(index),
                kind: ResourceKind::Texture(source),
            };

            textures.push(resource);
        }

        let shader_content = include_str!("../../assets/standard.wgsl");
        let shader_source = ShaderProcessor::generate_shader_resource_from_wsgl_content(
            &shader_name,
            shader_content.to_owned(),
            &BTreeSet::from_iter(vec![
                "vertex_displacement_texture".to_owned(),
                "uv_displacement_texture".to_owned(),
            ]),
        );

        let mut resources = Vec::with_capacity(1 + pmx.materials.len() + pmx.textures.len());

        match shader_source {
            Ok(source) => {
                let resource = Resource {
                    name: shader_name,
                    kind: ResourceKind::Shader(source),
                };
                resources.push(resource);
            }
            Err(err) => {
                error!(
                    "failed to process shader `{}`; it will be ignored: {}",
                    shader_name, err
                );
            }
        }

        resources.push(pmx_model_resource);
        resources.push(Resource {
            name: vertex_morph_index_texture_name,
            kind: ResourceKind::Texture(morph_data.vertex_morph_index_texture_source),
        });
        resources.push(Resource {
            name: uv_morph_index_texture_name,
            kind: ResourceKind::Texture(morph_data.uv_morph_index_texture_source),
        });
        resources.push(Resource {
            name: vertex_displacement_texture_name,
            kind: ResourceKind::Texture(morph_data.vertex_displacement_texture_source),
        });
        resources.push(Resource {
            name: uv_displacement_texture_name,
            kind: ResourceKind::Texture(morph_data.uv_displacement_texture_source),
        });
        resources.extend(materials);
        resources.extend(textures);

        Ok(resources)
    }
}

struct MorphData {
    pub morphs: Vec<PmxModelMorph>,
    pub vertex_morph_index_texture_source: TextureSource,
    pub uv_morph_index_texture_source: TextureSource,
    pub vertex_displacement_texture_source: TextureSource,
    pub uv_displacement_texture_source: TextureSource,
    pub vertex_attributes: Vec<MorphVertexAttribute>,
}

#[derive(Default, Clone)]
struct MorphVertexAttribute {
    pub vertex_morph_index_start: u32,
    pub vertex_morph_count: u32,
    pub uv_morph_index_start: u32,
    pub uv_morph_count: u32,
}

fn make_morph_data(pmx_name: &str, vertex_count: u32, pmx_morphs: &[PmxMorph]) -> MorphData {
    let mut morphs = Vec::with_capacity(pmx_morphs.len());

    /// Encoded as texture format `RG32U`
    struct VertexMorphIndex {
        pub morph_index: u32,
        pub displacement_index: u32,
    }

    /// Encoded as texture format `RGBA32U` (alpha is not used)
    struct UvMorphIndex {
        /// - 0 for uv
        /// - 1 for additional 1
        /// - 2 for additional 2
        /// - 3 for additional 3
        /// - 4 for additional 4
        pub index: u32,
        pub morph_index: u32,
        pub displacement_index: u32,
    }

    let mut vertex_morph_index_map = HashMap::<u32, Vec<VertexMorphIndex>>::new();
    let mut uv_morph_index_map = HashMap::<u32, Vec<UvMorphIndex>>::new();

    let mut vertex_displacements = Vec::new();
    let mut uv_displacements = Vec::new();

    for (morph_index, morph) in pmx_morphs.iter().enumerate() {
        let morph_name = morph.name_local.clone();
        let morph_kind = match &morph.offset {
            PmxMorphOffset::Group(elements) => {
                let mut group_elements = Vec::with_capacity(elements.len());

                for element in elements {
                    let morph_index = element.index.get();

                    if morph_index < 0 || pmx_morphs.len() <= morph_index as usize {
                        continue;
                    }

                    group_elements.push(PmxModelMorphGroupElement {
                        morph_index: morph_index as u32,
                        coefficient: element.coefficient,
                    });
                }

                PmxModelMorphKind::Group(group_elements)
            }
            PmxMorphOffset::Vertex(vertices) => {
                for vertex in vertices {
                    let vertex_index = vertex.index.get();

                    vertex_morph_index_map
                        .entry(vertex_index)
                        .or_default()
                        .push(VertexMorphIndex {
                            morph_index: morph_index as u32,
                            displacement_index: vertex_displacements.len() as u32,
                        });

                    vertex_displacements.push(Vec3::new(
                        vertex.translation.x,
                        vertex.translation.y,
                        vertex.translation.z,
                    ));
                }

                PmxModelMorphKind::Vertex
            }
            PmxMorphOffset::Bone(_) => PmxModelMorphKind::Bone,
            PmxMorphOffset::Uv { offsets, uv_index } => {
                for uv in offsets {
                    let vertex_index = uv.index.get();

                    uv_morph_index_map
                        .entry(vertex_index)
                        .or_default()
                        .push(UvMorphIndex {
                            index: *uv_index as u32,
                            morph_index: morph_index as u32,
                            displacement_index: uv_displacements.len() as u32,
                        });

                    uv_displacements.push(Vec4::new(uv.vec4.x, uv.vec4.y, uv.vec4.z, uv.vec4.w));
                }

                PmxModelMorphKind::Uv
            }
            PmxMorphOffset::Material(elements) => {
                let mut material_elements = Vec::with_capacity(elements.len());

                for element in elements {
                    let material_index = element.index.get();
                    let material_index =
                        if material_index < 0 || pmx_morphs.len() <= material_index as usize {
                            None
                        } else {
                            Some(material_index as u32)
                        };

                    material_elements.push(PmxModelMorphMaterialElement {
                        material_index,
                        offset_mode: match element.offset_mode {
                            PmxMorphOffsetMaterialOffsetMode::Multiply => {
                                PmxModelMorphMaterialOffsetMode::Multiply
                            }
                            PmxMorphOffsetMaterialOffsetMode::Additive => {
                                PmxModelMorphMaterialOffsetMode::Additive
                            }
                        },
                        diffuse_color: Vec4::new(
                            element.diffuse_color.x,
                            element.diffuse_color.y,
                            element.diffuse_color.z,
                            element.diffuse_color.w,
                        ),
                        specular_color: Vec3::new(
                            element.specular_color.x,
                            element.specular_color.y,
                            element.specular_color.z,
                        ),
                        specular_strength: element.specular_strength,
                        ambient_color: Vec3::new(
                            element.ambient_color.x,
                            element.ambient_color.y,
                            element.ambient_color.z,
                        ),
                        edge_color: Vec4::new(
                            element.edge_color.x,
                            element.edge_color.y,
                            element.edge_color.z,
                            element.edge_color.w,
                        ),
                        edge_size: element.edge_size,
                        texture_tint_color: Vec4::new(
                            element.texture_tint_color.x,
                            element.texture_tint_color.y,
                            element.texture_tint_color.z,
                            element.texture_tint_color.w,
                        ),
                        environment_tint_color: Vec4::new(
                            element.environment_tint_color.x,
                            element.environment_tint_color.y,
                            element.environment_tint_color.z,
                            element.environment_tint_color.w,
                        ),
                        toon_tint_color: Vec4::new(
                            element.toon_tint_color.x,
                            element.toon_tint_color.y,
                            element.toon_tint_color.z,
                            element.toon_tint_color.w,
                        ),
                    })
                }

                PmxModelMorphKind::Material(material_elements)
            }
            PmxMorphOffset::Flip(elements) => {
                let mut group_elements = Vec::with_capacity(elements.len());

                for element in elements {
                    let morph_index = element.index.get();

                    if morph_index < 0 || pmx_morphs.len() <= morph_index as usize {
                        continue;
                    }

                    group_elements.push(PmxModelMorphGroupElement {
                        morph_index: morph_index as u32,
                        coefficient: -element.coefficient,
                    });
                }

                PmxModelMorphKind::Group(group_elements)
            }
            PmxMorphOffset::Impulse(_) => PmxModelMorphKind::Impulse,
        };

        morphs.push(PmxModelMorph {
            name: morph_name,
            kind: morph_kind,
        });
    }

    let mut vertex_attributes = vec![MorphVertexAttribute::default(); vertex_count as usize];
    let mut vertex_morph_indices = Vec::new();
    let mut uv_morph_indices = Vec::new();

    for (vertex_index, morph_indices) in vertex_morph_index_map {
        let attribute = &mut vertex_attributes[vertex_index as usize];

        attribute.vertex_morph_index_start = vertex_morph_indices.len() as u32;
        attribute.vertex_morph_count = morph_indices.len() as u32;

        vertex_morph_indices.extend(morph_indices);
    }

    for (vertex_index, morph_indices) in uv_morph_index_map {
        let attribute = &mut vertex_attributes[vertex_index as usize];

        attribute.uv_morph_index_start = uv_morph_indices.len() as u32;
        attribute.uv_morph_count = morph_indices.len() as u32;

        uv_morph_indices.extend(morph_indices);
    }

    let vertex_morph_index_texture_size =
        ((vertex_morph_indices.len() as f32).sqrt().ceil() as u32).max(1);
    let uv_morph_index_texture_size = ((uv_morph_indices.len() as f32).sqrt().ceil() as u32).max(1);
    let vertex_displacement_texture_size =
        ((vertex_displacements.len() as f32).sqrt().ceil() as u32).max(1);
    let uv_displacement_texture_size =
        ((uv_displacements.len() as f32).sqrt().ceil() as u32).max(1);

    if 2048 < vertex_morph_index_texture_size {
        warn!(
            "for the PMX model `{}`, vertex morph index texture size `{}` exceeds the maximum texture size of 2048; it may not be able to be used as a texture",
            pmx_name,
            vertex_morph_index_texture_size
        );
    }

    if 2048 < uv_morph_index_texture_size {
        warn!(
            "for the PMX model `{}`, uv morph index texture size `{}` exceeds the maximum texture size of 2048; it may not be able to be used as a texture",
            pmx_name,
            uv_morph_index_texture_size
        );
    }

    if 2048 < vertex_displacement_texture_size {
        warn!(
            "for the PMX model `{}`, vertex displacement texture size `{}` exceeds the maximum texture size of 2048; it may not be able to be used as a texture",
            pmx_name,
            vertex_displacement_texture_size
        );
    }

    if 2048 < uv_displacement_texture_size {
        warn!(
            "for the PMX model `{}`, uv displacement texture size `{}` exceeds the maximum texture size of 2048; it may not be able to be used as a texture",
            pmx_name,
            uv_displacement_texture_size
        );
    }

    let mut vertex_morph_index_texels = Vec::with_capacity(
        (vertex_morph_index_texture_size * vertex_morph_index_texture_size) as usize
            * size_of::<[u32; 2]>(),
    );
    let mut uv_morph_index_texels = Vec::with_capacity(
        (uv_morph_index_texture_size * uv_morph_index_texture_size) as usize
            * size_of::<[u32; 4]>(),
    );
    let mut vertex_displacement_texels = Vec::with_capacity(
        (vertex_displacement_texture_size * vertex_displacement_texture_size) as usize
            * size_of::<[f32; 4]>(),
    );
    let mut uv_displacement_texels = Vec::with_capacity(
        (uv_displacement_texture_size * uv_displacement_texture_size) as usize
            * size_of::<[f32; 4]>(),
    );

    for index in &vertex_morph_indices {
        let mut x = [0u8; size_of::<u32>()];
        let mut y = [0u8; size_of::<u32>()];

        LittleEndian::write_u32(&mut x, index.morph_index);
        LittleEndian::write_u32(&mut y, index.displacement_index);

        vertex_morph_index_texels.extend(x);
        vertex_morph_index_texels.extend(y);
    }

    for index in &uv_morph_indices {
        let mut x = [0u8; size_of::<u32>()];
        let mut y = [0u8; size_of::<u32>()];
        let mut z = [0u8; size_of::<u32>()];
        let mut w = [0u8; size_of::<u32>()];

        LittleEndian::write_u32(&mut x, index.index);
        LittleEndian::write_u32(&mut y, index.morph_index);
        LittleEndian::write_u32(&mut z, index.displacement_index);
        LittleEndian::write_u32(&mut w, 0);

        uv_morph_index_texels.extend(x);
        uv_morph_index_texels.extend(y);
        uv_morph_index_texels.extend(z);
        uv_morph_index_texels.extend(w);
    }

    for displacement in &vertex_displacements {
        let mut x = [0u8; size_of::<f32>()];
        let mut y = [0u8; size_of::<f32>()];
        let mut z = [0u8; size_of::<f32>()];
        let mut w = [0u8; size_of::<f32>()];

        LittleEndian::write_f32(&mut x, displacement.x);
        LittleEndian::write_f32(&mut y, displacement.y);
        LittleEndian::write_f32(&mut z, displacement.z);
        LittleEndian::write_f32(&mut w, 0f32);

        vertex_displacement_texels.extend(x);
        vertex_displacement_texels.extend(y);
        vertex_displacement_texels.extend(z);
        vertex_displacement_texels.extend(w);
    }

    for displacement in &uv_displacements {
        let mut x = [0u8; size_of::<f32>()];
        let mut y = [0u8; size_of::<f32>()];
        let mut z = [0u8; size_of::<f32>()];
        let mut w = [0u8; size_of::<f32>()];

        LittleEndian::write_f32(&mut x, displacement.x);
        LittleEndian::write_f32(&mut y, displacement.y);
        LittleEndian::write_f32(&mut z, displacement.z);
        LittleEndian::write_f32(&mut w, displacement.w);

        uv_displacement_texels.extend(x);
        uv_displacement_texels.extend(y);
        uv_displacement_texels.extend(z);
        uv_displacement_texels.extend(w);
    }

    vertex_morph_index_texels.extend(std::iter::repeat(0u8).take(
        ((vertex_morph_index_texture_size * vertex_morph_index_texture_size) as usize)
            * size_of::<[u32; 2]>()
            - vertex_morph_index_texels.len(),
    ));

    uv_morph_index_texels.extend(std::iter::repeat(0u8).take(
        ((uv_morph_index_texture_size * uv_morph_index_texture_size) as usize)
            * size_of::<[u32; 4]>()
            - uv_morph_index_texels.len(),
    ));

    vertex_displacement_texels.extend(std::iter::repeat(0u8).take(
        ((vertex_displacement_texture_size * vertex_displacement_texture_size) as usize)
            * size_of::<[u32; 4]>()
            - vertex_displacement_texels.len(),
    ));

    uv_displacement_texels.extend(std::iter::repeat(0u8).take(
        ((uv_displacement_texture_size * uv_displacement_texture_size) as usize)
            * size_of::<[u32; 4]>()
            - uv_displacement_texels.len(),
    ));

    let vertex_morph_index_texture = TextureSource::new(TextureKind::Single(TextureElement {
        data: vertex_morph_index_texels,
        size: TextureElementSize {
            width: vertex_morph_index_texture_size as u16,
            height: vertex_morph_index_texture_size as u16,
        },
        texture_format: TextureElementTextureFormat::RG32Uint,
        sampling_mode: TextureElementSamplingMode::Point,
        wrapping_mode_u: TextureElementWrappingMode::Clamp,
        wrapping_mode_v: TextureElementWrappingMode::Clamp,
    }));

    let uv_morph_index_texture = TextureSource::new(TextureKind::Single(TextureElement {
        data: uv_morph_index_texels,
        size: TextureElementSize {
            width: uv_morph_index_texture_size as u16,
            height: uv_morph_index_texture_size as u16,
        },
        texture_format: TextureElementTextureFormat::RGBA32Uint,
        sampling_mode: TextureElementSamplingMode::Point,
        wrapping_mode_u: TextureElementWrappingMode::Clamp,
        wrapping_mode_v: TextureElementWrappingMode::Clamp,
    }));

    let vertex_displacement_texture = TextureSource::new(TextureKind::Single(TextureElement {
        data: vertex_displacement_texels,
        size: TextureElementSize {
            width: vertex_displacement_texture_size as u16,
            height: vertex_displacement_texture_size as u16,
        },
        texture_format: TextureElementTextureFormat::RGBA32Float,
        sampling_mode: TextureElementSamplingMode::Point,
        wrapping_mode_u: TextureElementWrappingMode::Clamp,
        wrapping_mode_v: TextureElementWrappingMode::Clamp,
    }));

    let uv_displacement_texture = TextureSource::new(TextureKind::Single(TextureElement {
        data: uv_displacement_texels,
        size: TextureElementSize {
            width: uv_displacement_texture_size as u16,
            height: uv_displacement_texture_size as u16,
        },
        texture_format: TextureElementTextureFormat::RGBA32Float,
        sampling_mode: TextureElementSamplingMode::Point,
        wrapping_mode_u: TextureElementWrappingMode::Clamp,
        wrapping_mode_v: TextureElementWrappingMode::Clamp,
    }));

    MorphData {
        morphs,
        vertex_morph_index_texture_source: vertex_morph_index_texture,
        uv_morph_index_texture_source: uv_morph_index_texture,
        vertex_displacement_texture_source: vertex_displacement_texture,
        uv_displacement_texture_source: uv_displacement_texture,
        vertex_attributes,
    }
}

fn make_vertex_data(
    pmx_vertices: &[PmxVertex],
    morph_vertex_attributes: Vec<MorphVertexAttribute>,
) -> (Vec<u8>, Vec<PmxModelVertexLayoutElement>) {
    let layout_elements = vec![
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::Position,
            offset: size_of::<[[u8; 4]; 0]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::Normal,
            offset: size_of::<[[u8; 4]; 3]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::TexCoord,
            offset: size_of::<[[u8; 4]; 6]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::AdditionalVec4(0),
            offset: size_of::<[[u8; 4]; 8]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::AdditionalVec4(1),
            offset: size_of::<[[u8; 4]; 12]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::AdditionalVec4(2),
            offset: size_of::<[[u8; 4]; 16]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::AdditionalVec4(3),
            offset: size_of::<[[u8; 4]; 20]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::DeformKind,
            offset: size_of::<[[u8; 4]; 24]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::BoneIndex,
            offset: size_of::<[[u8; 4]; 25]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::BoneWeight,
            offset: size_of::<[[u8; 4]; 29]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::SdefC,
            offset: size_of::<[[u8; 4]; 33]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::SdefR0,
            offset: size_of::<[[u8; 4]; 36]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::SdefR1,
            offset: size_of::<[[u8; 4]; 39]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::EdgeSize,
            offset: size_of::<[[u8; 4]; 42]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::VertexMorphIndexStart,
            offset: size_of::<[[u8; 4]; 43]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::VertexMorphCount,
            offset: size_of::<[[u8; 4]; 44]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::UvMorphIndexStart,
            offset: size_of::<[[u8; 4]; 45]>() as u64,
        },
        PmxModelVertexLayoutElement {
            kind: PmxModelVertexLayoutElementKind::UvMorphCount,
            offset: size_of::<[[u8; 4]; 46]>() as u64,
        },
    ];

    let mut position = 0;
    let mut vertex_data = vec![0; size_of::<[[u8; 4]; 47]>() * pmx_vertices.len()];

    let mut write = |data: &[u8]| {
        vertex_data[position..position + data.len()].copy_from_slice(data);
        position += data.len();
    };

    trait WriteAsLeBytes {
        fn write_as_le_bytes(self, dst: &mut [u8]);
    }

    impl WriteAsLeBytes for i32 {
        fn write_as_le_bytes(self, dst: &mut [u8]) {
            LittleEndian::write_i32(dst, self);
        }
    }

    impl WriteAsLeBytes for u32 {
        fn write_as_le_bytes(self, dst: &mut [u8]) {
            LittleEndian::write_u32(dst, self);
        }
    }

    impl WriteAsLeBytes for f32 {
        fn write_as_le_bytes(self, dst: &mut [u8]) {
            LittleEndian::write_f32(dst, self);
        }
    }

    macro_rules! write {
        ($writer:expr, $value:expr) => {{
            let mut buf = [0u8; 4];
            WriteAsLeBytes::write_as_le_bytes($value, &mut buf);
            $writer(&mut buf);
        }};
    }

    for (index, pmx_vertex) in pmx_vertices.iter().enumerate() {
        // position
        write!(write, pmx_vertex.position.x);
        write!(write, pmx_vertex.position.y);
        write!(write, pmx_vertex.position.z);

        // normal
        write!(write, pmx_vertex.normal.x);
        write!(write, pmx_vertex.normal.y);
        write!(write, pmx_vertex.normal.z);

        // texcoord
        write!(write, pmx_vertex.uv.x);
        write!(write, pmx_vertex.uv.y);

        // additional vec4 0
        write!(write, pmx_vertex.additional_vec4s[0].x);
        write!(write, pmx_vertex.additional_vec4s[0].y);
        write!(write, pmx_vertex.additional_vec4s[0].z);
        write!(write, pmx_vertex.additional_vec4s[0].w);

        // additional vec4 1
        write!(write, pmx_vertex.additional_vec4s[1].x);
        write!(write, pmx_vertex.additional_vec4s[1].y);
        write!(write, pmx_vertex.additional_vec4s[1].z);
        write!(write, pmx_vertex.additional_vec4s[1].w);

        // additional vec4 2
        write!(write, pmx_vertex.additional_vec4s[2].x);
        write!(write, pmx_vertex.additional_vec4s[2].y);
        write!(write, pmx_vertex.additional_vec4s[2].z);
        write!(write, pmx_vertex.additional_vec4s[2].w);

        // additional vec4 3
        write!(write, pmx_vertex.additional_vec4s[3].x);
        write!(write, pmx_vertex.additional_vec4s[3].y);
        write!(write, pmx_vertex.additional_vec4s[3].z);
        write!(write, pmx_vertex.additional_vec4s[3].w);

        // deform info
        match &pmx_vertex.deform_kind {
            PmxVertexDeformKind::Bdef1 { bone_index } => {
                // deform kind
                write!(write, 0u32);

                // bone index
                write!(write, bone_index.get());
                write!(write, -1i32);
                write!(write, -1i32);
                write!(write, -1i32);

                // bone weight
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef c
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r0
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r1
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);
            }
            PmxVertexDeformKind::Bdef2 {
                bone_index_1,
                bone_index_2,
                bone_weight,
            } => {
                // deform kind
                write!(write, 1u32);

                // bone index
                write!(write, bone_index_1.get());
                write!(write, bone_index_2.get());
                write!(write, -1i32);
                write!(write, -1i32);

                // bone weight
                write!(write, *bone_weight);
                write!(write, 1f32 - bone_weight);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef c
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r0
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r1
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);
            }
            PmxVertexDeformKind::Bdef4 {
                bone_index_1,
                bone_index_2,
                bone_index_3,
                bone_index_4,
                bone_weight_1,
                bone_weight_2,
                bone_weight_3,
                bone_weight_4,
            } => {
                // deform kind
                write!(write, 2u32);

                // bone index
                write!(write, bone_index_1.get());
                write!(write, bone_index_2.get());
                write!(write, bone_index_3.get());
                write!(write, bone_index_4.get());

                // bone weight
                let total = bone_weight_1 + bone_weight_2 + bone_weight_3 + bone_weight_4;

                if total <= f32::EPSILON {
                    write!(write, 0f32);
                    write!(write, 0f32);
                    write!(write, 0f32);
                    write!(write, 0f32);
                } else {
                    write!(write, bone_weight_1 / total);
                    write!(write, bone_weight_2 / total);
                    write!(write, bone_weight_3 / total);
                    write!(write, bone_weight_4 / total);
                }

                // sdef c
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r0
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r1
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);
            }
            PmxVertexDeformKind::Sdef {
                bone_index_1,
                bone_index_2,
                bone_weight,
                c,
                r0,
                r1,
            } => {
                // deform kind
                write!(write, 3u32);

                // bone index
                write!(write, bone_index_1.get());
                write!(write, bone_index_2.get());
                write!(write, -1i32);
                write!(write, -1i32);

                // bone weight
                write!(write, *bone_weight);
                write!(write, 1f32 - bone_weight);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef c
                write!(write, c.x);
                write!(write, c.y);
                write!(write, c.z);

                // sdef r0
                write!(write, r0.x);
                write!(write, r0.y);
                write!(write, r0.z);

                // sdef r1
                write!(write, r1.x);
                write!(write, r1.y);
                write!(write, r1.z);
            }
            PmxVertexDeformKind::Qdef {
                bone_index_1,
                bone_index_2,
                bone_index_3,
                bone_index_4,
                bone_weight_1,
                bone_weight_2,
                bone_weight_3,
                bone_weight_4,
            } => {
                // deform kind
                write!(write, 2u32);

                // bone index
                write!(write, bone_index_1.get());
                write!(write, bone_index_2.get());
                write!(write, bone_index_3.get());
                write!(write, bone_index_4.get());

                // bone weight
                let total = bone_weight_1 + bone_weight_2 + bone_weight_3 + bone_weight_4;

                if total <= f32::EPSILON {
                    write!(write, 0f32);
                    write!(write, 0f32);
                    write!(write, 0f32);
                    write!(write, 0f32);
                } else {
                    write!(write, bone_weight_1 / total);
                    write!(write, bone_weight_2 / total);
                    write!(write, bone_weight_3 / total);
                    write!(write, bone_weight_4 / total);
                }

                // sdef c
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r0
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);

                // sdef r1
                write!(write, 0f32);
                write!(write, 0f32);
                write!(write, 0f32);
            }
        }

        // edge size
        write!(write, pmx_vertex.edge_size);

        let vertex_morph_attribute = &morph_vertex_attributes[index];

        // vertex morph index start
        write!(write, vertex_morph_attribute.vertex_morph_index_start);

        // vertex morph count
        write!(write, vertex_morph_attribute.vertex_morph_count);

        // uv morph index start
        write!(write, vertex_morph_attribute.uv_morph_index_start);

        // uv morph count
        write!(write, vertex_morph_attribute.uv_morph_count);
    }

    (vertex_data, layout_elements)
}

fn make_index_data(
    mut pmx_material_namer: impl FnMut(&PmxMaterial) -> String,
    pmx_materials: &[PmxMaterial],
    pmx_indices: &PmxIndices,
) -> (Vec<u8>, PmxModelIndexKind, Vec<PmxModelElement>) {
    let mut position = 0;
    let mut index_data = vec![0; size_of::<u32>() * pmx_indices.vertex_indices.len()];

    for index in &pmx_indices.vertex_indices {
        index_data[position..position + 4].copy_from_slice(&index.get().to_le_bytes());
        position += size_of::<u32>();
    }

    let mut previous_index_count = 0;
    let mut elements = Vec::with_capacity(pmx_materials.len());

    for pmx_material in pmx_materials {
        elements.push(PmxModelElement {
            material_name: pmx_material_namer(pmx_material),
            index_range: (
                previous_index_count,
                previous_index_count + pmx_material.surface_count,
            ),
        });

        previous_index_count += pmx_material.surface_count;
    }

    (index_data, PmxModelIndexKind::U32, elements)
}

fn make_material_source(
    mut pmx_shader_namer: impl FnMut(&PmxMaterial) -> String,
    mut pmx_texture_namer: impl FnMut(&PmxTexture) -> String,
    mut pmx_internal_toon_texture_namer: impl FnMut(u8) -> String,
    render_type: MaterialRenderType,
    pmx_material: &PmxMaterial,
    pmx_textures: &[PmxTexture],
    vertex_morph_index_texture_name: &str,
    uv_morph_index_texture_name: &str,
    vertex_displacement_texture_name: &str,
    uv_displacement_texture_name: &str,
) -> MaterialSource {
    let mut properties = vec![];

    properties.push(MaterialProperty {
        name: "vertex_morph_index_texture".to_owned(),
        value: MaterialPropertyValue::Texture {
            texture_name: vertex_morph_index_texture_name.to_owned(),
        },
    });

    properties.push(MaterialProperty {
        name: "uv_morph_index_texture".to_owned(),
        value: MaterialPropertyValue::Texture {
            texture_name: uv_morph_index_texture_name.to_owned(),
        },
    });

    properties.push(MaterialProperty {
        name: "vertex_displacement_texture".to_owned(),
        value: MaterialPropertyValue::Texture {
            texture_name: vertex_displacement_texture_name.to_owned(),
        },
    });

    properties.push(MaterialProperty {
        name: "uv_displacement_texture".to_owned(),
        value: MaterialPropertyValue::Texture {
            texture_name: uv_displacement_texture_name.to_owned(),
        },
    });

    let pmx_texture_index = pmx_material.texture_index.get();

    if 0 <= pmx_texture_index && (pmx_texture_index as usize) < pmx_textures.len() {
        let pmx_texture = &pmx_textures[pmx_texture_index as usize];

        properties.push(MaterialProperty {
            name: "texture".to_owned(),
            value: MaterialPropertyValue::Texture {
                texture_name: pmx_texture_namer(pmx_texture),
            },
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

    let toon_texture_name = match pmx_material.toon_mode {
        PmxMaterialToonMode::Texture { index } => {
            let pmx_texture_index = index.get();

            if 0 <= pmx_texture_index && (pmx_texture_index as usize) < pmx_textures.len() {
                let pmx_texture = &pmx_textures[pmx_texture_index as usize];
                Some(pmx_texture_namer(pmx_texture))
            } else {
                None
            }
        }
        PmxMaterialToonMode::InternalTexture { index } => {
            Some(pmx_internal_toon_texture_namer(index))
        }
    };

    if let Some(toon_texture_name) = toon_texture_name {
        properties.push(MaterialProperty {
            name: "toon_texture".to_owned(),
            value: MaterialPropertyValue::Texture {
                texture_name: toon_texture_name,
            },
        });

        properties.push(MaterialProperty {
            name: "toon_texture_sampler".to_owned(),
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

    let env_texture_index = pmx_material.environment_texture_index.get();

    if 0 <= env_texture_index && (env_texture_index as usize) < pmx_textures.len() {
        let pmx_texture = &pmx_textures[env_texture_index as usize];

        properties.push(MaterialProperty {
            name: "env_texture".to_owned(),
            value: MaterialPropertyValue::Texture {
                texture_name: pmx_texture_namer(pmx_texture),
            },
        });

        properties.push(MaterialProperty {
            name: "env_texture_sampler".to_owned(),
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
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec4(Vec4::new(
            pmx_material.diffuse_color.x,
            pmx_material.diffuse_color.y,
            pmx_material.diffuse_color.z,
            pmx_material.diffuse_color.w,
        ))),
    });
    properties.push(MaterialProperty {
        name: "specular_color".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec3(Vec3::new(
            pmx_material.specular_color.x,
            pmx_material.specular_color.y,
            pmx_material.specular_color.z,
        ))),
    });
    properties.push(MaterialProperty {
        name: "specular_strength".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Float(
            pmx_material.specular_strength,
        )),
    });
    properties.push(MaterialProperty {
        name: "ambient_color".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec3(Vec3::new(
            pmx_material.ambient_color.x,
            pmx_material.ambient_color.y,
            pmx_material.ambient_color.z,
        ))),
    });
    properties.push(MaterialProperty {
        name: "edge_color".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec4(Vec4::new(
            pmx_material.edge_color.x,
            pmx_material.edge_color.y,
            pmx_material.edge_color.z,
            pmx_material.edge_color.w,
        ))),
    });
    properties.push(MaterialProperty {
        name: "edge_size".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Float(
            pmx_material.edge_size,
        )),
    });
    properties.push(MaterialProperty {
        name: "env_blend_mode".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::U32(
            match pmx_material.environment_blend_mode {
                PmxMaterialEnvironmentBlendMode::Disabled => 0,
                PmxMaterialEnvironmentBlendMode::Multiplicative => 1,
                PmxMaterialEnvironmentBlendMode::Additive => 2,
                PmxMaterialEnvironmentBlendMode::AdditionalVec4UV => 3,
            },
        )),
    });
    properties.push(MaterialProperty {
        name: "texture_tint_color".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec4(Vec4::ONE)),
    });
    properties.push(MaterialProperty {
        name: "environment_tint_color".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec4(Vec4::ONE)),
    });
    properties.push(MaterialProperty {
        name: "toon_tint_color".to_owned(),
        value: MaterialPropertyValue::Uniform(MaterialPropertyUniformValue::Vec4(Vec4::ONE)),
    });

    MaterialSource::new(
        pmx_shader_namer(pmx_material),
        MaterialRenderState {
            render_type,
            no_cull_back_face: pmx_material.flags.no_cull_back_face,
            cast_shadow_on_ground: pmx_material.flags.cast_shadow_on_ground,
            cast_shadow_on_object: pmx_material.flags.cast_shadow_on_object,
            receive_shadow: pmx_material.flags.receive_shadow,
            has_edge: pmx_material.flags.has_edge,
            vertex_color: pmx_material.flags.vertex_color,
            point_drawing: pmx_material.flags.point_drawing,
            line_drawing: pmx_material.flags.line_drawing,
            group_order: 0,
        },
        properties,
    )
}

fn make_texture_source(
    pmx_path: &Path,
    pmx_texture: &PmxTexture,
) -> Result<TextureSource, AnyError> {
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

fn make_internal_toon_texture_source(
    pmx_path: &Path,
    index: u8,
) -> Result<TextureSource, AnyError> {
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
        &parent_path.join(&format!("toon{:0>2}.bmp", index)),
        &TextureMetadata {
            texture_format: TextureElementTextureFormat::RGBA8Unorm,
            sampling_mode: Some(TextureElementSamplingMode::Bilinear),
            wrapping_mode_u: Some(TextureElementWrappingMode::Clamp),
            wrapping_mode_v: Some(TextureElementWrappingMode::Clamp),
        },
    )
}
