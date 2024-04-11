use super::{Mesh, Vec3, VertexList};

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn compute_from_vertex_list(vertex_list: &VertexList) -> Self {
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for vertex in &vertex_list.positions {
            min.x = vertex.x.min(min.x);
            min.y = vertex.y.min(min.y);
            min.z = vertex.z.min(min.z);

            max.x = vertex.x.max(max.x);
            max.y = vertex.y.max(max.y);
            max.z = vertex.z.max(max.z);
        }

        Self { min, max }
    }

    pub fn merge(meshes: &[Mesh]) -> Self {
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for mesh in meshes {
            let mesh_min = mesh.bounding_box.min;
            let mesh_max = mesh.bounding_box.max;

            min.x = mesh_min.x.min(min.x);
            min.y = mesh_min.y.min(min.y);
            min.z = mesh_min.z.min(min.z);

            max.x = mesh_max.x.max(max.x);
            max.y = mesh_max.y.max(max.y);
            max.z = mesh_max.z.max(max.z);
        }

        Self { min, max }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
            && self.min.z <= point.z
            && point.z <= self.max.z
    }

    pub fn contains_bounding_box(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.min.x
            && other.max.x <= self.max.x
            && self.min.y <= other.min.y
            && other.max.y <= self.max.y
            && self.min.z <= other.min.z
            && other.max.z <= self.max.z
    }
}
