use lvl_math::Mat4;
use std::mem::size_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, VertexAttribute, VertexFormat,
};
use zerocopy::AsBytes;

pub struct InstanceDataProvider<'device> {
    device: &'device Device,
}

impl<'device> InstanceDataProvider<'device> {
    pub fn new(device: &'device Device) -> Self {
        Self { device }
    }

    pub fn instance_data_size(&self) -> u32 {
        size_of::<[[f32; 4]; 4]>() as u32
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
        ]
    }

    pub fn create_instance_buffer(&self, matrix: &Mat4) -> Buffer {
        self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: matrix.as_bytes(),
            usage: BufferUsages::VERTEX,
        })
    }
}
