use super::{Plane, PlaneSide, VertexList};

#[derive(Debug, Clone, PartialEq)]
pub enum TrianglePlaneSide {
    Front,
    Back,
    OnPlane,
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
            (PlaneSide::Front, PlaneSide::Front, PlaneSide::OnPlane) => TrianglePlaneSide::Front,
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
            (PlaneSide::Front, PlaneSide::Back, PlaneSide::OnPlane) => {
                // there are two ways to solve this, but treating `OnPlane` vertex to `Front` is fine here
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[2], self.indices[0]],
                    back: [self.indices[1]],
                }
            }
            (PlaneSide::Front, PlaneSide::OnPlane, PlaneSide::Front) => TrianglePlaneSide::Front,
            (PlaneSide::Front, PlaneSide::OnPlane, PlaneSide::Back) => {
                // there are two ways to solve this, but treating `OnPlane` vertex to `Front` is fine here
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[0], self.indices[1]],
                    back: [self.indices[2]],
                }
            }
            (PlaneSide::Front, PlaneSide::OnPlane, PlaneSide::OnPlane) => TrianglePlaneSide::Front,
            (PlaneSide::Back, PlaneSide::Front, PlaneSide::Front) => {
                TrianglePlaneSide::Back2Front1 {
                    front: [self.indices[0]],
                    back: [self.indices[1], self.indices[2]],
                }
            }
            (PlaneSide::Back, PlaneSide::Front, PlaneSide::Back) => {
                TrianglePlaneSide::Back2Front1 {
                    front: [self.indices[1]],
                    back: [self.indices[2], self.indices[0]],
                }
            }
            (PlaneSide::Back, PlaneSide::Front, PlaneSide::OnPlane) => {
                // there are two ways to solve this, but treating `OnPlane` vertex to `Front` is fine here
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[1], self.indices[2]],
                    back: [self.indices[0]],
                }
            }
            (PlaneSide::Back, PlaneSide::Back, PlaneSide::Front) => {
                TrianglePlaneSide::Back2Front1 {
                    front: [self.indices[2]],
                    back: [self.indices[0], self.indices[1]],
                }
            }
            (PlaneSide::Back, PlaneSide::Back, PlaneSide::Back) => TrianglePlaneSide::Back,
            (PlaneSide::Back, PlaneSide::Back, PlaneSide::OnPlane) => TrianglePlaneSide::Back,
            (PlaneSide::Back, PlaneSide::OnPlane, PlaneSide::Front) => {
                // there are two ways to solve this, but treating `OnPlane` vertex to `Front` is fine here
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[1], self.indices[2]],
                    back: [self.indices[0]],
                }
            }
            (PlaneSide::Back, PlaneSide::OnPlane, PlaneSide::Back) => TrianglePlaneSide::Back,
            (PlaneSide::Back, PlaneSide::OnPlane, PlaneSide::OnPlane) => TrianglePlaneSide::Back,
            (PlaneSide::OnPlane, PlaneSide::Front, PlaneSide::Front) => TrianglePlaneSide::Front,
            (PlaneSide::OnPlane, PlaneSide::Front, PlaneSide::Back) => {
                // there are two ways to solve this, but treating `OnPlane` vertex to `Front` is fine here
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[0], self.indices[1]],
                    back: [self.indices[2]],
                }
            }
            (PlaneSide::OnPlane, PlaneSide::Front, PlaneSide::OnPlane) => TrianglePlaneSide::Front,
            (PlaneSide::OnPlane, PlaneSide::Back, PlaneSide::Front) => {
                // there are two ways to solve this, but treating `OnPlane` vertex to `Front` is fine here
                TrianglePlaneSide::Front2Back1 {
                    front: [self.indices[2], self.indices[0]],
                    back: [self.indices[1]],
                }
            }
            (PlaneSide::OnPlane, PlaneSide::Back, PlaneSide::Back) => TrianglePlaneSide::Back,
            (PlaneSide::OnPlane, PlaneSide::Back, PlaneSide::OnPlane) => TrianglePlaneSide::Back,
            (PlaneSide::OnPlane, PlaneSide::OnPlane, PlaneSide::Front) => TrianglePlaneSide::Front,
            (PlaneSide::OnPlane, PlaneSide::OnPlane, PlaneSide::Back) => TrianglePlaneSide::Back,
            (PlaneSide::OnPlane, PlaneSide::OnPlane, PlaneSide::OnPlane) => {
                TrianglePlaneSide::OnPlane
            }
        }
    }
}
