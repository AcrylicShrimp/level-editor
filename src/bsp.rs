mod bounding_box;
mod mesh;
mod node;
mod plane;
mod triangle;
mod vec3;
mod vertex_list;

pub use bounding_box::*;
pub use mesh::*;
pub use node::*;
pub use plane::*;
pub use triangle::*;
pub use vec3::*;
pub use vertex_list::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BspLimit {
    pub max_depth: Option<usize>,
    pub min_size: BoundingBox,
}

pub fn build_bsp_tree(meshes: Vec<Mesh>, limit: BspLimit) -> BspNode {
    let mut root_node = BspNode::new_leaf(meshes);
    build::split(&mut root_node, 0, limit);
    root_node
}

mod build {
    use super::*;

    pub fn split(bsp_node: &mut BspNode, depth: usize, limit: BspLimit) {
        if !bsp_node.is_leaf() {
            return;
        }

        if let Some(max_depth) = limit.max_depth {
            if max_depth <= depth {
                return;
            }
        }

        if limit.min_size.contains_bounding_box(&bsp_node.bounding_box) {
            return;
        }

        todo!()
    }
}
