use super::{Mesh, VertexList};
use lvl_math::{Plane, PlaneSide, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundingBoxPlaneSide {
    Front,
    Back,
    Spanning,
}

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

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn center_point(&self) -> Vec3 {
        (self.min + self.max) / 2.0
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

    pub fn plane_side(&self, plane: Plane) -> BoundingBoxPlaneSide {
        let mut front = 0;
        let mut back = 0;

        match plane.point_side(Vec3::new(self.min.x, self.min.y, self.min.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.max.x, self.min.y, self.min.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.min.x, self.max.y, self.min.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.min.x, self.min.y, self.max.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.max.x, self.max.y, self.min.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.max.x, self.min.y, self.max.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.min.x, self.max.y, self.max.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match plane.point_side(Vec3::new(self.max.x, self.max.y, self.max.z)) {
            PlaneSide::Front => front += 1,
            PlaneSide::Back => back += 1,
        }

        match (0 < front, 0 < back) {
            (true, true) => BoundingBoxPlaneSide::Spanning,
            (true, false) => BoundingBoxPlaneSide::Front,
            (false, true) => BoundingBoxPlaneSide::Back,
            (false, false) => unreachable!(),
        }
    }
}
