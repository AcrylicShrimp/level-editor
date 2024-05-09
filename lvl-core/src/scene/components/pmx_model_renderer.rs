use crate::{
    gfx::{
        elements::{PmxModel, PmxModelElement, PmxModelVertexLayout},
        GfxContext,
    },
    scene::Component,
};
use lvl_resource::{MaterialRenderType, PmxModelVertexLayoutElementKind};
use std::{
    any::Any,
    cell::{RefCell, RefMut},
    sync::Arc,
};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Device, Face,
    FragmentState, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, StencilFaceState, StencilState, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

#[derive(Debug)]
pub struct PmxModelRenderer {
    model: PmxModel,
    // TODO: make a way to store pipeline for each render pass
    render_pipelines: RefCell<Vec<Arc<RenderPipeline>>>,
}

impl PmxModelRenderer {
    pub fn new(model: PmxModel) -> Self {
        Self {
            render_pipelines: RefCell::new(Vec::with_capacity(model.elements().len())),
            model,
        }
    }

    pub fn model(&self) -> &PmxModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut PmxModel {
        &mut self.model
    }

    pub(crate) fn construct_render_pipelines(
        &self,
        instance_data_size: u64,
        instance_data_attributes: &[VertexAttribute],
        gfx_ctx: &GfxContext,
    ) -> RefMut<Vec<Arc<RenderPipeline>>> {
        let mut render_pipelines = self.render_pipelines.borrow_mut();

        if !render_pipelines.is_empty() {
            return render_pipelines;
        }

        for element in self.model.elements() {
            let render_pipeline = self.create_render_pipeline(
                instance_data_size,
                instance_data_attributes,
                &self.model.vertex_layout(),
                element,
                &gfx_ctx.device,
            );

            render_pipelines.push(Arc::new(render_pipeline));
        }

        render_pipelines
    }

    fn create_render_pipeline(
        &self,
        instance_data_size: u64,
        instance_data_attributes: &[VertexAttribute],
        layout: &PmxModelVertexLayout,
        element: &PmxModelElement,
        device: &Device,
    ) -> RenderPipeline {
        let material = &element.material;
        let shader = material.shader();
        let shader_locations = &shader.reflection().locations;
        let vertex_layout = self.model.vertex_layout();
        let mut attributes = Vec::with_capacity(vertex_layout.elements.len());

        for element in &vertex_layout.elements {
            let name = shader_input_name_from_vertex_layout_kind(element.kind);
            let format = vertex_format_from_vertex_layout_kind(element.kind);

            let shader_location = match shader_locations.get(&name) {
                Some(location) => *location,
                None => {
                    continue;
                }
            };

            attributes.push(VertexAttribute {
                format,
                offset: element.offset,
                shader_location,
            });
        }

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("pmx-model-element-render-pipeline"),
            layout: Some(shader.pipeline_layout()),
            vertex: VertexState {
                module: shader.module(),
                entry_point: &shader.reflection().vertex_entry_point,
                buffers: &[
                    // TODO: let engine decide actual vertex buffers
                    // that is required because there are some pre-defined vertex buffers (e.g. instance transforms, etc.)
                    VertexBufferLayout {
                        array_stride: instance_data_size,
                        step_mode: VertexStepMode::Instance,
                        attributes: instance_data_attributes,
                    },
                    VertexBufferLayout {
                        array_stride: layout.stride,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &attributes,
                    },
                ],
            },
            primitive: PrimitiveState {
                topology: if material.render_state().point_drawing {
                    PrimitiveTopology::PointList
                } else if material.render_state().line_drawing {
                    PrimitiveTopology::LineList
                } else {
                    PrimitiveTopology::TriangleList
                },
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: if material.render_state().no_cull_back_face {
                    None
                } else {
                    Some(Face::Back)
                },
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                // TODO: let engine decide actual depth stencil state
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: shader.module(),
                entry_point: &shader.reflection().fragment_entry_point,
                targets: &[Some(ColorTargetState {
                    // TODO: let engine decide actual color target state
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: match material.render_state().render_type {
                        MaterialRenderType::Opaque => None,
                        MaterialRenderType::Transparent => Some(BlendState::ALPHA_BLENDING),
                    },
                    write_mask: ColorWrites::all(),
                })],
            }),
            multiview: None,
        })
    }
}

fn shader_input_name_from_vertex_layout_kind(kind: PmxModelVertexLayoutElementKind) -> String {
    match kind {
        PmxModelVertexLayoutElementKind::Position => "position".to_owned(),
        PmxModelVertexLayoutElementKind::Normal => "normal".to_owned(),
        PmxModelVertexLayoutElementKind::TexCoord => "uv".to_owned(),
        PmxModelVertexLayoutElementKind::Tangent => "tangent".to_owned(),
        PmxModelVertexLayoutElementKind::AdditionalVec4(index) => format!("additional_{}_", index),
        PmxModelVertexLayoutElementKind::DeformKind => "deform_kind".to_owned(),
        PmxModelVertexLayoutElementKind::BoneIndex => "bone_index".to_owned(),
        PmxModelVertexLayoutElementKind::BoneWeight => "bone_weight".to_owned(),
        PmxModelVertexLayoutElementKind::SdefC => "sdef_c".to_owned(),
        PmxModelVertexLayoutElementKind::SdefR0 => "sdef_r0".to_owned(),
        PmxModelVertexLayoutElementKind::SdefR1 => "sdef_r1".to_owned(),
        PmxModelVertexLayoutElementKind::EdgeSize => "edge_size".to_owned(),
        PmxModelVertexLayoutElementKind::VertexMorphIndexStart => {
            "vertex_morph_index_start".to_owned()
        }
        PmxModelVertexLayoutElementKind::UvMorphIndexStart => "uv_morph_index_start".to_owned(),
        PmxModelVertexLayoutElementKind::VertexMorphCount => "vertex_morph_count".to_owned(),
        PmxModelVertexLayoutElementKind::UvMorphCount => "uv_morph_count".to_owned(),
    }
}

fn vertex_format_from_vertex_layout_kind(kind: PmxModelVertexLayoutElementKind) -> VertexFormat {
    match kind {
        PmxModelVertexLayoutElementKind::Position => VertexFormat::Float32x3,
        PmxModelVertexLayoutElementKind::Normal => VertexFormat::Float32x3,
        PmxModelVertexLayoutElementKind::TexCoord => VertexFormat::Float32x2,
        PmxModelVertexLayoutElementKind::Tangent => VertexFormat::Float32x3,
        PmxModelVertexLayoutElementKind::AdditionalVec4(_) => VertexFormat::Float32x4,
        PmxModelVertexLayoutElementKind::DeformKind => VertexFormat::Uint32,
        PmxModelVertexLayoutElementKind::BoneIndex => VertexFormat::Uint32x4,
        PmxModelVertexLayoutElementKind::BoneWeight => VertexFormat::Float32x4,
        PmxModelVertexLayoutElementKind::SdefC => VertexFormat::Float32x3,
        PmxModelVertexLayoutElementKind::SdefR0 => VertexFormat::Float32x3,
        PmxModelVertexLayoutElementKind::SdefR1 => VertexFormat::Float32x3,
        PmxModelVertexLayoutElementKind::EdgeSize => VertexFormat::Float32,
        PmxModelVertexLayoutElementKind::VertexMorphIndexStart => VertexFormat::Uint32,
        PmxModelVertexLayoutElementKind::UvMorphIndexStart => VertexFormat::Uint32,
        PmxModelVertexLayoutElementKind::VertexMorphCount => VertexFormat::Uint32,
        PmxModelVertexLayoutElementKind::UvMorphCount => VertexFormat::Uint32,
    }
}

impl Component for PmxModelRenderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
