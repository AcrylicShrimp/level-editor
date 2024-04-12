use super::{BoundingBox, Mesh, Plane};

#[derive(Debug, Clone)]
pub enum BspNode {
    Leaf(BspNodeLeaf),
    Internal(BspNodeInternal),
}

#[derive(Debug, Clone)]
pub struct BspNodeLeaf {
    pub meshes: Vec<Mesh>,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone)]
pub struct BspNodeInternal {
    pub plane: Plane,
    pub front: Option<Box<BspNode>>,
    pub back: Option<Box<BspNode>>,
}

impl BspNode {
    pub fn leaf(meshes: Vec<Mesh>) -> Self {
        let bounding_box = BoundingBox::merge(&meshes);

        Self::Leaf(BspNodeLeaf {
            meshes,
            bounding_box,
        })
    }
}
