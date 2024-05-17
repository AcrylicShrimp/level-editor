use crate::scene::Component;
use lvl_math::{Mat4, Vec2, Vec3};
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIAnchor {
    pub min: Vec2,
    pub max: Vec2,
}

impl UIAnchor {
    pub const FULL: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::ONE,
    };

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UIMargin {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl UIMargin {
    pub const ZERO: Self = Self {
        left: 0f32,
        right: 0f32,
        top: 0f32,
        bottom: 0f32,
    };

    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn from_size(pivot: Vec2, position: Vec2, size: Vec2) -> Self {
        let pivot_x = pivot.x * size.x;
        let pivot_y = pivot.y * size.y;

        Self {
            left: position.x - pivot_x,
            right: -(position.x - pivot_x + size.x),
            top: -(position.y - pivot_y + size.y),
            bottom: position.y - pivot_y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UIElement {
    is_dirty: bool,
    anchor: UIAnchor,
    margin: UIMargin,
    size: Vec2,
    position: Vec2,
    transform: Mat4,
    pub is_interactable: bool,
}

impl UIElement {
    pub fn new(anchor: UIAnchor, margin: UIMargin, is_interactable: bool) -> Self {
        Self {
            is_dirty: true,
            anchor,
            margin,
            size: Vec2::ZERO,
            position: Vec2::ZERO,
            transform: Mat4::identity(),
            is_interactable,
        }
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn anchor(&self) -> UIAnchor {
        self.anchor
    }

    pub fn margin(&self) -> UIMargin {
        self.margin
    }

    /// The size of the UI element. Note that this property will be re-calculated after all update/late update hooks have been called.
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// The position of the UI element. Note that this property will be re-calculated after all update/late update hooks have been called.
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// The transform of the UI element. Note that this property will be re-calculated after all update/late update hooks have been called.
    pub fn transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn set_anchor(&mut self, anchor: UIAnchor) {
        self.anchor = anchor;
        self.is_dirty = true;
    }

    pub fn set_margin(&mut self, margin: UIMargin) {
        self.margin = margin;
        self.is_dirty = true;
    }

    pub(crate) fn compute_properties(&mut self, parent_size: Vec2, transform: &Mat4) {
        let margin_left = parent_size.x * self.anchor.min.x;
        let margin_bottom = parent_size.y * self.anchor.min.y;
        let margin_right = parent_size.x * self.anchor.max.x;
        let margin_top = parent_size.y * self.anchor.max.y;

        let mut x = margin_left + self.margin.left;
        let mut y = margin_bottom + self.margin.top;

        let width = margin_right - x - self.margin.right;
        let height = margin_top - y - self.margin.bottom;

        // center the element
        x += width * 0.5;
        y -= height * 0.5;

        self.position = Vec2::new(x, y);
        self.size = Vec2::new(width, height);

        let (position, rotation, scale) = transform.split();

        self.transform = Mat4::srt(
            Vec3::from_vec2(self.position, 0.0) + position,
            rotation,
            Vec3::from_vec2(self.size, 1.0) * scale,
        );
        self.is_dirty = false;
    }
}

impl Component for UIElement {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
