use winit::dpi::PhysicalSize;

#[derive(Debug)]
pub struct ScreenSize {
    is_dirty: bool,
    size: PhysicalSize<u32>,
}

impl ScreenSize {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        Self {
            is_dirty: true,
            size,
        }
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub(crate) fn set_size(&mut self, size: PhysicalSize<u32>) {
        self.is_dirty = true;
        self.size = size;
    }

    pub(crate) fn reset_dirty(&mut self) {
        self.is_dirty = false;
    }
}
