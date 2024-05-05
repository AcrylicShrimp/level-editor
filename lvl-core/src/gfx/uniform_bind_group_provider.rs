use lvl_math::Mat4;
use std::{mem::size_of, num::NonZeroU64};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, Device, Queue, ShaderStages,
};
use zerocopy::AsBytes;

pub struct UniformBindGroupProvider {
    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl UniformBindGroupProvider {
    pub fn new(device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZeroU64::new(size_of::<[[f32; 4]; 4]>() as u64).unwrap(),
                    ),
                },
                count: None,
            }],
        });

        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<[[f32; 4]; 4]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: Some(NonZeroU64::new(size_of::<[[f32; 4]; 4]>() as u64).unwrap()),
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn update_camera_matrix(&self, matrix: &Mat4, queue: &Queue) {
        if let Some(mut view) = queue.write_buffer_with(
            &self.buffer,
            0,
            NonZeroU64::new(size_of::<[[f32; 4]; 4]>() as u64).unwrap(),
        ) {
            view.copy_from_slice(matrix.transposed().as_bytes());
        }
    }
}