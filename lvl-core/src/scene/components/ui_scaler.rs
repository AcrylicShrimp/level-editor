use crate::scene::Component;
use lvl_math::Vec2;
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UIScaleMode {
    Constant,
    Stretch,
    Fit,
    Fill,
    MatchWidth,
    MatchHeight,
}

#[derive(Debug, Clone)]
pub struct UIScaler {
    is_dirty: bool,
    mode: UIScaleMode,
    reference_size: Vec2,
}

impl UIScaler {
    pub fn new(mode: UIScaleMode, reference_size: Vec2) -> Self {
        Self {
            is_dirty: true,
            mode,
            reference_size,
        }
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn mode(&self) -> UIScaleMode {
        self.mode
    }

    pub fn reference_size(&self) -> Vec2 {
        self.reference_size
    }

    pub fn set_mode(&mut self, mode: UIScaleMode) {
        self.is_dirty = true;
        self.mode = mode;
    }

    pub fn set_reference_size(&mut self, reference_size: Vec2) {
        self.is_dirty = true;
        self.reference_size = reference_size;
    }

    pub(crate) fn compute_size(&mut self, parent_size: Vec2) -> Vec2 {
        let size = match self.mode {
            UIScaleMode::Constant => Vec2::new(self.reference_size.x, self.reference_size.y),
            UIScaleMode::Stretch => Vec2::new(parent_size.x, parent_size.y),
            UIScaleMode::Fit => {
                let scale_x = parent_size.x / self.reference_size.x;
                let scale_y = parent_size.y / self.reference_size.y;
                let scale = f32::min(scale_x, scale_y);
                Vec2::new(scale * self.reference_size.x, scale * self.reference_size.y)
            }
            UIScaleMode::Fill => {
                let scale_x = parent_size.x / self.reference_size.x;
                let scale_y = parent_size.y / self.reference_size.y;
                let scale = f32::max(scale_x, scale_y);
                Vec2::new(scale * self.reference_size.x, scale * self.reference_size.y)
            }
            UIScaleMode::MatchWidth => {
                let scale = parent_size.x / self.reference_size.x;
                Vec2::new(scale * self.reference_size.x, scale * self.reference_size.y)
            }
            UIScaleMode::MatchHeight => {
                let scale = parent_size.y / self.reference_size.y;
                Vec2::new(scale * self.reference_size.x, scale * self.reference_size.y)
            }
        };

        self.is_dirty = false;

        size
    }
}

impl Component for UIScaler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
