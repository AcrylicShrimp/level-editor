use std::mem::size_of;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshLayoutElementKind {
    /// Vec3
    Position,
    /// Vec3
    Normal,
    /// Vec2
    TexCoord(u8),
    /// Vec3
    Tangent,
}

impl MeshLayoutElementKind {
    pub fn size(self) -> usize {
        match self {
            Self::Position => size_of::<[f32; 3]>(),
            Self::Normal => size_of::<[f32; 3]>(),
            Self::TexCoord(_) => size_of::<[f32; 2]>(),
            Self::Tangent => size_of::<[f32; 3]>(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshLayoutElement {
    pub name: String,
    pub kind: MeshLayoutElementKind,
    pub offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshLayout {
    elements: Vec<MeshLayoutElement>,
    stride: u64,
}

impl MeshLayout {
    pub fn new(elements: Vec<MeshLayoutElement>) -> Self {
        let stride = compute_stride_from_elements(&elements);
        Self { elements, stride }
    }

    pub fn with_stride(elements: Vec<MeshLayoutElement>, stride: u64) -> Self {
        Self { elements, stride }
    }

    pub fn elements(&self) -> &[MeshLayoutElement] {
        &self.elements
    }

    pub fn stride(&self) -> u64 {
        self.stride
    }
}

fn compute_stride_from_elements(elements: &[MeshLayoutElement]) -> u64 {
    elements
        .iter()
        .map(|element| element.kind.size() as u64 + element.offset)
        .max()
        .unwrap_or_default()
}
