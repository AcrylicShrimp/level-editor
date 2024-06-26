use super::{BufferSlicer, PerFrameBufferPool};
use lvl_math::Mat4;
use std::{mem::size_of, num::NonZeroU64};
use wgpu::{Device, Queue, VertexAttribute, VertexFormat};
use zerocopy::AsBytes;

pub struct InstanceDataProvider;

impl InstanceDataProvider {
    pub fn instance_data_size(&self) -> u64 {
        size_of::<[[f32; 4]; 8]>() as u64
    }

    pub fn instance_data_attributes(&self) -> &'static [VertexAttribute] {
        &[
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 0) as u64,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 1) as u64,
                shader_location: 1,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 2) as u64,
                shader_location: 2,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 3) as u64,
                shader_location: 3,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 4) as u64,
                shader_location: 4,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 5) as u64,
                shader_location: 5,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 6) as u64,
                shader_location: 6,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (size_of::<[f32; 4]>() * 7) as u64,
                shader_location: 7,
            },
        ]
    }

    pub fn create_instance_buffer(
        &self,
        matrix: &Mat4,
        buffer_pool: &PerFrameBufferPool,
        device: &Device,
        queue: &Queue,
    ) -> BufferSlicer {
        let size = NonZeroU64::new(self.instance_data_size() as u64).unwrap();
        let slicer = buffer_pool.allocate(size, device);

        if let Some(mut view) = queue.write_buffer_with(slicer.buffer(), slicer.offset(), size) {
            view[..size_of::<[f32; 4]>() * 4].copy_from_slice(matrix.as_bytes());
            view[size_of::<[f32; 4]>() * 4..].copy_from_slice(matrix.inversed().as_bytes());
        }

        slicer
    }
}
