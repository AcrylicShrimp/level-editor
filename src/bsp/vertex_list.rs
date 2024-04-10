use super::Vec3;

/// Indicates how normals and tangents are calculated for the mesh, when splitting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceShading {
    /// Normals and tangents are copied from existing vertex list. No interpolation.
    Flat,
    /// Normals and tangents are interpolated from adjacent vertices.
    Smooth,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VertexList {
    /// Surface shading.
    pub surface_shading: SurfaceShading,
    /// Positions.
    pub positions: Vec<Vec3>,
    /// Normals. Data only (not participating to BSP tree building). 3 elements each.
    pub normals: Option<Vec<f32>>,
    /// Tangent vectors. Data only (not participating to BSP tree building). 3 elements each.
    pub tangents: Option<Vec<f32>>,
    /// Texture coordinates. Data only (not participating to BSP tree building). 2 elements each.
    /// Can be multiple.
    pub texcoords: Vec<Vec<f32>>,
}

impl VertexList {
    pub fn new(surface_shading: SurfaceShading) -> Self {
        Self {
            surface_shading,
            positions: vec![],
            normals: None,
            texcoords: vec![],
            tangents: None,
        }
    }

    pub fn add_vertex(
        &mut self,
        position: Vec3,
        normal: Option<[f32; 3]>,
        tangent: Option<[f32; 3]>,
        texcoords: Vec<[f32; 2]>,
    ) -> usize {
        self.positions.push(position);
        self.normals.as_mut().map(|n| {
            if let Some(normal) = normal {
                n.push(normal[0]);
                n.push(normal[1]);
                n.push(normal[2]);
            }
        });
        self.tangents.as_mut().map(|t| {
            if let Some(tangent) = tangent {
                t.push(tangent[0]);
                t.push(tangent[1]);
                t.push(tangent[2]);
            }
        });

        for index in 0..self.texcoords.len() {
            self.texcoords[index].push(texcoords[index][0]);
            self.texcoords[index].push(texcoords[index][1]);
        }

        self.positions.len() - 1
    }

    pub fn transfer_vertex(index: usize, from: &Self, to: &mut Self) -> usize {
        let position = from.positions[index];
        let mut texcoords = from.texcoords.iter().map(|t| t[index]);
        let normal = from.normals.as_ref().map(|n| n[index]);
        let tangent = from.tangents.as_ref().map(|t| t[index]);

        to.positions.push(position);
        to.normals.as_mut().map(|n| {
            if let Some(normal) = normal {
                n.push(normal);
            }
        });
        to.tangents.as_mut().map(|t| {
            if let Some(tangent) = tangent {
                t.push(tangent);
            }
        });

        for t in &mut to.texcoords {
            if let Some(texcoord) = texcoords.next() {
                t.push(texcoord);
            }
        }

        to.positions.len() - 1
    }
}
