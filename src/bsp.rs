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

pub fn build_bsp_tree(meshes: Vec<Mesh>) -> BspNode {
    todo!()
}

mod build {
    use super::*;

    pub struct SplittedBspNode {}
}
