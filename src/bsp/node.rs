use super::{BoundingBox, Mesh, Plane};

#[derive(Debug, Clone)]
pub struct BspNode {
    pub plane: Plane,
    pub meshes_on_plane: Vec<Mesh>,
    pub meshes_on_front: Vec<Mesh>,
    pub meches_on_back: Vec<Mesh>,
    pub front: Option<Box<BspNode>>,
    pub back: Option<Box<BspNode>>,
    pub bounding_box: BoundingBox,
}
