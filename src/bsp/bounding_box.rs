use super::{Vec3, VertexList};

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
}
