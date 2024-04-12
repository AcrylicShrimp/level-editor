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
    build::split(BspNode::leaf(meshes), 0, &limit)
}

mod build {
    use super::*;

    pub fn split(bsp_node: BspNode, depth: usize, limit: &BspLimit) -> BspNode {
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

        let dividing_plane = make_dividing_plane(&leaf);
        let mut front_meshes = Vec::new();
        let mut back_meshes = Vec::new();

        for mesh in leaf.meshes {
            let splitted = mesh.split_by_plane(dividing_plane);

            if !splitted.front.is_empty() {
                front_meshes.push(splitted.front);
            }

            if !splitted.back.is_empty() {
                back_meshes.push(splitted.back);
            }
        }

        let front = BspNode::leaf(front_meshes);
        let back = BspNode::leaf(back_meshes);

        let front = split(front, depth + 1, limit);
        let back = split(back, depth + 1, limit);

        BspNode::Internal(BspNodeInternal {
            plane: dividing_plane,
            front: Some(Box::new(front)),
            back: Some(Box::new(back)),
        })
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum DividingAxis {
        X,
        Y,
        Z,
    }

    impl DividingAxis {
        pub fn normal(self) -> Vec3 {
            match self {
                DividingAxis::X => Vec3::new(1.0, 0.0, 0.0),
                DividingAxis::Y => Vec3::new(0.0, 1.0, 0.0),
                DividingAxis::Z => Vec3::new(0.0, 0.0, 1.0),
            }
        }
    }

    fn make_dividing_plane(leaf: &BspNodeLeaf) -> Plane {
        let bounding_box_size = leaf.bounding_box.size();
        let axis = if bounding_box_size.x > bounding_box_size.y
            && bounding_box_size.x > bounding_box_size.z
        {
            DividingAxis::X
        } else if bounding_box_size.y > bounding_box_size.z {
            DividingAxis::Y
        } else {
            DividingAxis::Z
        };

        let normal = axis.normal();
        let point = leaf.bounding_box.center_point();
        Plane::new(normal, point)
    }
}
