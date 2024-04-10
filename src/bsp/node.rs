use super::{Mesh, Plane};
use std::num::NonZeroUsize;

#[derive(Debug, Clone)]
pub struct BspNode {
    pub plane: Plane,
    pub meshes_on_plane: Vec<Mesh>,
    pub meshes_on_front: Vec<Mesh>,
    pub meches_on_back: Vec<Mesh>,
    pub left_node_index: Option<NonZeroUsize>,
    pub right_node_index: Option<NonZeroUsize>,
}
