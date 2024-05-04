use crate::{
    gfx::{
        elements::{Material, MeshLayoutElementKind, StaticMesh},
        GfxContext,
    },
    scene::Component,
};
use lvl_resource::MeshIndexKind;
use std::{
    any::Any,
    cell::{RefCell, RefMut},
};
use wgpu::{
    ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Face, FragmentState,
    FrontFace, IndexFormat, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, StencilFaceState, StencilState, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

#[derive(Debug)]
pub struct StaticMeshRenderer {
    mesh: StaticMesh,
    material: Material,
    pipeline: RefCell<Option<RenderPipeline>>,
}

impl StaticMeshRenderer {
    pub fn new(mesh: StaticMesh, material: Material) -> Self {
        Self {
            mesh,
            material,
            pipeline: RefCell::new(None),
        }
    }

    pub fn mesh(&self) -> &StaticMesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn set_mesh(&mut self, mesh: StaticMesh) {
        self.mesh = mesh;
        *self.pipeline.borrow_mut() = None;
    }

    pub fn set_material(&mut self, material: Material) {
        self.material = material;
        *self.pipeline.borrow_mut() = None;
    }

    pub(crate) fn construct_render_pipeline(
        &self,
        gfx_ctx: &GfxContext,
    ) -> RefMut<Option<RenderPipeline>> {
        let mut pipeline = self.pipeline.borrow_mut();

        if pipeline.is_some() {
            return pipeline;
        }

        let mesh_layout = self.mesh.layout();
        let shader_locations = &self.material.shader().reflection().locations;
        let mut attributes = Vec::with_capacity(mesh_layout.elements().len());

        for element in mesh_layout.elements() {
            let shader_location = match shader_locations.get(&element.name) {
                Some(location) => *location,
                None => continue,
            };

            match element.kind {
                MeshLayoutElementKind::Position => {
                    attributes.push(VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: element.offset,
                        shader_location,
                    });
                }
                MeshLayoutElementKind::Normal => {
                    attributes.push(VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: element.offset,
                        shader_location,
                    });
                }
                MeshLayoutElementKind::TexCoord(_) => {
                    attributes.push(VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: element.offset,
                        shader_location,
                    });
                }
                MeshLayoutElementKind::Tangent => {
                    attributes.push(VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: element.offset,
                        shader_location,
                    });
                }
                MeshLayoutElementKind::Additional(_) => {
                    attributes.push(VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: element.offset,
                        shader_location,
                    });
                }
            }
        }

        let render_pipeline = gfx_ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(self.material.shader().pipeline_layout()),
                vertex: VertexState {
                    module: self.material.shader().module(),
                    entry_point: &self.material.shader().reflection().vertex_entry_point,
                    buffers: &[VertexBufferLayout {
                        array_stride: mesh_layout.stride(),
                        step_mode: VertexStepMode::Vertex,
                        attributes: &attributes,
                    }],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: Some(match self.mesh.index_kind() {
                        MeshIndexKind::U16 => IndexFormat::Uint16,
                        MeshIndexKind::U32 => IndexFormat::Uint32,
                    }),
                    front_face: FrontFace::Cw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
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
                    module: self.material.shader().module(),
                    entry_point: &self.material.shader().reflection().fragment_entry_point,
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: ColorWrites::all(),
                    })],
                }),
                multiview: None,
            });

        *pipeline = Some(render_pipeline);
        pipeline
    }
}

impl Component for StaticMeshRenderer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
