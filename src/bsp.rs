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
    pub min_triangle_count: Option<usize>,
    pub min_size: BoundingBox,
}

pub fn build_bsp_tree(meshes: Vec<Mesh>, limit: BspLimit) -> BspNode {
    build::split(BspNode::leaf(meshes), 0, limit)
}

mod build {
    use super::*;

    pub fn split(bsp_node: BspNode, depth: usize, limit: BspLimit) -> BspNode {
        let leaf = match bsp_node {
            BspNode::Leaf(leaf) => leaf,
            BspNode::Internal(internal) => return BspNode::Internal(internal),
        };

        if let Some(max_depth) = limit.max_depth {
            if max_depth <= depth {
                return BspNode::Leaf(leaf);
            }
        }

        if let Some(min_triangle_count) = limit.min_triangle_count {
            if leaf
                .meshes
                .iter()
                .map(|mesh| mesh.triangles.len())
                .sum::<usize>()
                < min_triangle_count
            {
                return BspNode::Leaf(leaf);
            }
        }

        if limit.min_size.contains_bounding_box(&leaf.bounding_box) {
            return BspNode::Leaf(leaf);
        }

        todo!()
    }
}
