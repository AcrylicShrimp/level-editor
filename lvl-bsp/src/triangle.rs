use super::VertexList;
use lvl_math::{Plane, PlaneSide};

#[derive(Debug, Clone, PartialEq)]
pub enum TrianglePlaneSide {
    Front,
    Back,
    Front2Back1 { front: [usize; 2], back: [usize; 1] },
    Back2Front1 { front: [usize; 1], back: [usize; 2] },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    /// Indices of the vertices of the triangle. It follows the winding order of the mesh.
    pub indices: [usize; 3],
}

impl Triangle {
    pub fn plane_side(&self, vertex_list: &VertexList, plane: Plane) -> TrianglePlaneSide {
        let positions = [
            vertex_list.positions[self.indices[0]],
            vertex_list.positions[self.indices[1]],
            vertex_list.positions[self.indices[2]],
        ];
        let sides = [
            plane.point_side(positions[0]),
            plane.point_side(positions[1]),
            plane.point_side(positions[2]),
        ];

        match (sides[0], sides[1], sides[2]) {
            (PlaneSide::Front, PlaneSide::Front, PlaneSide::Front) => TrianglePlaneSide::Front,
            (PlaneSide::Front, PlaneSide::Front, PlaneSide::Back) => {
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[0], self.indices[1]],
                    back: [self.indices[2]],
                }
            }
            (PlaneSide::Front, PlaneSide::Back, PlaneSide::Front) => {
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[2], self.indices[0]],
                    back: [self.indices[1]],
                }
            }
            (PlaneSide::Front, PlaneSide::Back, PlaneSide::Back) => {
                TrianglePlaneSide::Back2Front1 {
                    front: [self.indices[0]],
                    back: [self.indices[1], self.indices[2]],
                }
            }
            (PlaneSide::Back, PlaneSide::Front, PlaneSide::Front) => {
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[1], self.indices[2]],
                    back: [self.indices[0]],
                }
            }
            (PlaneSide::Back, PlaneSide::Front, PlaneSide::Back) => {
                TrianglePlaneSide::Back2Front1 {
                    front: [self.indices[1]],
                    back: [self.indices[2], self.indices[0]],
                }
            }
            (PlaneSide::Back, PlaneSide::Back, PlaneSide::Front) => {
                TrianglePlaneSide::Back2Front1 {
                    front: [self.indices[2]],
                    back: [self.indices[0], self.indices[1]],
                }
            }
            (PlaneSide::Back, PlaneSide::Back, PlaneSide::Back) => TrianglePlaneSide::Back,
        }
    }
}
