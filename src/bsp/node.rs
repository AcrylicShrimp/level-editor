use super::{BoundingBox, Mesh, Plane};

#[derive(Debug, Clone)]
pub struct BspNode {
    pub meshes: Vec<Mesh>,
    pub bounding_box: BoundingBox,
    pub plane: Option<Plane>,
    pub front: Option<Box<BspNode>>,
    pub back: Option<Box<BspNode>>,
}

impl BspNode {
    pub fn new_leaf(meshes: Vec<Mesh>) -> Self {
        let bounding_box = BoundingBox::merge(&meshes);
        Self {
            meshes,
            bounding_box,
            plane: None,
            front: None,
            back: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.plane.is_none()
    }
}
