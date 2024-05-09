use parking_lot::Mutex;
use std::{cell::RefCell, num::NonZeroU64, sync::Arc};
use wgpu::{Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device};

const SINGLE_BUFFER_SIZE: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(64 * 1024 * 1024) }; // 64MiB

pub struct PerFrameBufferPool {
    buffers: Mutex<Vec<SingleBuffer>>,
}

impl PerFrameBufferPool {
    pub fn new() -> Self {
        Self {
            buffers: Mutex::new(Vec::with_capacity(4)),
        }
    }

    pub fn allocate(&self, size: NonZeroU64, device: &Device) -> BufferSlicer {
        let mut buffers = self.buffers.lock();

        for buffer in buffers.iter() {
            if let Some(slice) = buffer.allocate(size) {
                return slice;
            }
        }

        let buffer_size = SINGLE_BUFFER_SIZE.get().max(size.get());
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: buffer_size,
            usage: BufferUsages::COPY_DST
                | BufferUsages::VERTEX
                | BufferUsages::INDEX
                | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let single_buffer = SingleBuffer::new(buffer_size, buffer);

        buffers.push(single_buffer);
        buffers.last().unwrap().allocate(size).unwrap()
    }

    pub(crate) fn reset(&self) {
        // TODO: consider drop some buffers if they are not in use
        for buffer in self.buffers.lock().iter_mut() {
            buffer.reset();
        }
    }
}

struct SingleBuffer {
    size: u64,
    offset: RefCell<u64>,
    buffer: Arc<Buffer>,
}

impl SingleBuffer {
    pub fn new(size: u64, buffer: Buffer) -> Self {
        Self {
            size,
            offset: RefCell::new(0),
            buffer: Arc::new(buffer),
        }
    }

    pub fn allocate(&self, size: NonZeroU64) -> Option<BufferSlicer> {
        let mut offset = self.offset.borrow_mut();

        if self.size < *offset + size.get() {
            return None;
        }

        let slicer = BufferSlicer::new(self.buffer.clone(), *offset, size);
        *offset += size.get();

        Some(slicer)
    }

    pub(crate) fn reset(&self) {
        *self.offset.borrow_mut() = 0;
    }
}

#[derive(Debug, Clone)]
pub struct BufferSlicer {
    buffer: Arc<Buffer>,
    offset: u64,
    size: NonZeroU64,
}

impl BufferSlicer {
    fn new(buffer: Arc<Buffer>, offset: u64, size: NonZeroU64) -> Self {
        Self {
            buffer,
            offset,
            size,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        self.buffer.as_ref()
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size.get()
    }

    pub fn slice(&self) -> BufferSlice {
        self.buffer
            .slice(self.offset..self.offset + self.size.get())
    }
}
