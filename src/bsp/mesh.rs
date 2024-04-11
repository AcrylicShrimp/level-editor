use super::{BoundingBox, Plane, SurfaceShading, Triangle, TrianglePlaneSide, Vec3, VertexList};
use std::{collections::HashMap, num::NonZeroU32};

macro_rules! transfer_triangle {
    ($triangle:expr, $vertex_map:expr, $from:expr, $to:expr) => {{
        let indices = [
            *$vertex_map.entry($triangle.indices[0]).or_insert_with(|| {
                super::VertexList::transfer_vertex($triangle.indices[0], &$from, &mut $to)
            }),
            *$vertex_map.entry($triangle.indices[1]).or_insert_with(|| {
                super::VertexList::transfer_vertex($triangle.indices[1], &$from, &mut $to)
            }),
            *$vertex_map.entry($triangle.indices[2]).or_insert_with(|| {
                super::VertexList::transfer_vertex($triangle.indices[2], &$from, &mut $to)
            }),
        ];
        super::Triangle { indices }
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplittedMesh {
    pub front: Mesh,
    pub back: Mesh,
    pub on_plane: Mesh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub material_id: NonZeroU32,
    pub hierarch_id: NonZeroU32,
    pub vertex_list: VertexList,
    pub triangles: Vec<Triangle>,
    pub bounding_box: BoundingBox,
}

impl Mesh {
    pub fn new(
        material_id: NonZeroU32,
        hierarch_id: NonZeroU32,
        vertex_list: VertexList,
        triangles: Vec<Triangle>,
    ) -> Self {
        let bounding_box = BoundingBox::compute_from_vertex_list(&vertex_list);
        Self {
            material_id,
            hierarch_id,
            vertex_list,
            triangles,
            bounding_box,
        }
    }

    pub fn split_by_plane(self, plane: Plane) -> SplittedMesh {
        let mut front_vertex_list = VertexList::new(self.vertex_list.surface_shading);
        let mut back_vertex_list = VertexList::new(self.vertex_list.surface_shading);
        let mut on_plane_vertex_list = VertexList::new(self.vertex_list.surface_shading);

        let mut front_vertex_map = HashMap::new();
        let mut back_vertex_map = HashMap::new();
        let mut on_plane_vertex_map = HashMap::new();

        let mut front_triangles = Vec::new();
        let mut back_triangles = Vec::new();
        let mut on_plane_triangles = Vec::new();

        for triangle in self.triangles {
            match triangle.plane_side(&self.vertex_list, plane) {
                TrianglePlaneSide::Front => {
                    let triangle = transfer_triangle!(
                        triangle,
                        front_vertex_map,
                        self.vertex_list,
                        front_vertex_list
                    );
                    front_triangles.push(triangle);
                }
                TrianglePlaneSide::Back => {
                    let triangle = transfer_triangle!(
                        triangle,
                        back_vertex_map,
                        self.vertex_list,
                        back_vertex_list
                    );
                    back_triangles.push(triangle);
                }
                TrianglePlaneSide::OnPlane => {
                    let triangle = transfer_triangle!(
                        triangle,
                        on_plane_vertex_map,
                        self.vertex_list,
                        on_plane_vertex_list
                    );
                    on_plane_triangles.push(triangle);
                }
                TrianglePlaneSide::Front2Back1 { front, back } => {
                    let front_positions = [
                        self.vertex_list.positions[triangle.indices[front[0]]],
                        self.vertex_list.positions[triangle.indices[front[1]]],
                    ];
                    let back_positions = [self.vertex_list.positions[triangle.indices[back[0]]]];

                    let contact_points = [
                        plane.point_on(front_positions[0], back_positions[0] - front_positions[0]),
                        plane.point_on(front_positions[1], back_positions[0] - front_positions[1]),
                    ];
                    let total_lengths = [
                        (back_positions[0] - front_positions[0]).len(),
                        (back_positions[0] - front_positions[1]).len(),
                    ];

                    const EPSILON: f32 = 1e-3;

                    let ratios = [
                        if total_lengths[0] <= EPSILON {
                            0.0
                        } else {
                            (contact_points[0] - front_positions[0]).len() / total_lengths[0]
                        },
                        if total_lengths[1] <= EPSILON {
                            0.0
                        } else {
                            (contact_points[1] - front_positions[1]).len() / total_lengths[1]
                        },
                    ];

                    let front_new_positions = [contact_points[0], contact_points[1]];
                    let back_new_positions = front_new_positions.clone();

                    let front_new_normals = match &self.vertex_list.normals {
                        Some(normals) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    normals[triangle.indices[front[0]] * 3 + 0],
                                    normals[triangle.indices[front[0]] * 3 + 1],
                                    normals[triangle.indices[front[0]] * 3 + 2],
                                ],
                                [
                                    normals[triangle.indices[front[0]] * 3 + 0],
                                    normals[triangle.indices[front[0]] * 3 + 1],
                                    normals[triangle.indices[front[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => {
                                let front_normals = [
                                    Vec3::new(
                                        normals[triangle.indices[front[0]] * 3 + 0],
                                        normals[triangle.indices[front[0]] * 3 + 1],
                                        normals[triangle.indices[front[0]] * 3 + 2],
                                    ),
                                    Vec3::new(
                                        normals[triangle.indices[front[1]] * 3 + 0],
                                        normals[triangle.indices[front[1]] * 3 + 1],
                                        normals[triangle.indices[front[1]] * 3 + 2],
                                    ),
                                ];
                                let back_normal = Vec3::new(
                                    normals[triangle.indices[back[0]] * 3 + 0],
                                    normals[triangle.indices[back[0]] * 3 + 1],
                                    normals[triangle.indices[back[0]] * 3 + 2],
                                );

                                let front_new_normals = [
                                    front_normals[0] * ratios[0] + back_normal * (1.0 - ratios[0]),
                                    front_normals[1] * ratios[1] + back_normal * (1.0 - ratios[1]),
                                ];

                                Some([
                                    [
                                        front_new_normals[0].x,
                                        front_new_normals[0].y,
                                        front_new_normals[0].z,
                                    ],
                                    [
                                        front_new_normals[1].x,
                                        front_new_normals[1].y,
                                        front_new_normals[1].z,
                                    ],
                                ])
                            }
                        },
                        None => None,
                    };
                    let back_new_normals = match &self.vertex_list.normals {
                        Some(normals) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    normals[triangle.indices[back[0]] * 3 + 0],
                                    normals[triangle.indices[back[0]] * 3 + 1],
                                    normals[triangle.indices[back[0]] * 3 + 2],
                                ],
                                [
                                    normals[triangle.indices[back[0]] * 3 + 0],
                                    normals[triangle.indices[back[0]] * 3 + 1],
                                    normals[triangle.indices[back[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => front_new_normals.clone(),
                        },
                        None => None,
                    };

                    let front_new_tangents = match &self.vertex_list.tangents {
                        Some(tangents) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    tangents[triangle.indices[front[0]] * 3 + 0],
                                    tangents[triangle.indices[front[0]] * 3 + 1],
                                    tangents[triangle.indices[front[0]] * 3 + 2],
                                ],
                                [
                                    tangents[triangle.indices[front[0]] * 3 + 0],
                                    tangents[triangle.indices[front[0]] * 3 + 1],
                                    tangents[triangle.indices[front[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => {
                                let front_tangents = [
                                    Vec3::new(
                                        tangents[triangle.indices[front[0]] * 3 + 0],
                                        tangents[triangle.indices[front[0]] * 3 + 1],
                                        tangents[triangle.indices[front[0]] * 3 + 2],
                                    ),
                                    Vec3::new(
                                        tangents[triangle.indices[front[1]] * 3 + 0],
                                        tangents[triangle.indices[front[1]] * 3 + 1],
                                        tangents[triangle.indices[front[1]] * 3 + 2],
                                    ),
                                ];
                                let back_tangent = Vec3::new(
                                    tangents[triangle.indices[back[0]] * 3 + 0],
                                    tangents[triangle.indices[back[0]] * 3 + 1],
                                    tangents[triangle.indices[back[0]] * 3 + 2],
                                );

                                let front_new_tangents = [
                                    front_tangents[0] * ratios[0]
                                        + back_tangent * (1.0 - ratios[0]),
                                    front_tangents[1] * ratios[1]
                                        + back_tangent * (1.0 - ratios[1]),
                                ];

                                Some([
                                    [
                                        front_new_tangents[0].x,
                                        front_new_tangents[0].y,
                                        front_new_tangents[0].z,
                                    ],
                                    [
                                        front_new_tangents[1].x,
                                        front_new_tangents[1].y,
                                        front_new_tangents[1].z,
                                    ],
                                ])
                            }
                        },
                        None => None,
                    };
                    let back_new_tangents = match &self.vertex_list.tangents {
                        Some(tangents) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    tangents[triangle.indices[back[0]] * 3 + 0],
                                    tangents[triangle.indices[back[0]] * 3 + 1],
                                    tangents[triangle.indices[back[0]] * 3 + 2],
                                ],
                                [
                                    tangents[triangle.indices[back[0]] * 3 + 0],
                                    tangents[triangle.indices[back[0]] * 3 + 1],
                                    tangents[triangle.indices[back[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => front_new_tangents.clone(),
                        },
                        None => None,
                    };

                    let mut front_new_texcoords = [
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                    ];
                    let mut back_new_texcoords = [
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                    ];

                    for texcoords in &self.vertex_list.texcoords {
                        let front_texcoords = [
                            [
                                texcoords[triangle.indices[front[0]] * 2 + 0],
                                texcoords[triangle.indices[front[0]] * 2 + 1],
                            ],
                            [
                                texcoords[triangle.indices[front[1]] * 2 + 0],
                                texcoords[triangle.indices[front[1]] * 2 + 1],
                            ],
                        ];
                        let back_texcoords = [
                            texcoords[triangle.indices[back[0]] * 2 + 0],
                            texcoords[triangle.indices[back[0]] * 2 + 1],
                        ];

                        let new_texcoords = [
                            [
                                // u
                                front_texcoords[0][0] * ratios[0]
                                    + back_texcoords[0] * (1.0 - ratios[0]),
                                // v
                                front_texcoords[0][1] * ratios[0]
                                    + back_texcoords[1] * (1.0 - ratios[0]),
                            ],
                            [
                                // u
                                front_texcoords[1][0] * ratios[0]
                                    + back_texcoords[0] * (1.0 - ratios[0]),
                                // v
                                front_texcoords[1][1] * ratios[0]
                                    + back_texcoords[1] * (1.0 - ratios[0]),
                            ],
                        ];

                        front_new_texcoords[0].push(new_texcoords[0]);
                        front_new_texcoords[1].push(new_texcoords[1]);

                        back_new_texcoords[0].push(new_texcoords[0]);
                        back_new_texcoords[1].push(new_texcoords[1]);
                    }

                    let front_new_vertex_indices = [
                        front_vertex_list.add_vertex(
                            front_new_positions[0],
                            front_new_normals.map(|n| n[0]),
                            front_new_tangents.map(|t| t[0]),
                            std::mem::take(&mut front_new_texcoords[0]),
                        ),
                        front_vertex_list.add_vertex(
                            front_new_positions[1],
                            front_new_normals.map(|n| n[1]),
                            front_new_tangents.map(|t| t[1]),
                            std::mem::take(&mut front_new_texcoords[1]),
                        ),
                    ];
                    let back_new_vertex_indices = [
                        back_vertex_list.add_vertex(
                            back_new_positions[0],
                            back_new_normals.map(|n| n[0]),
                            back_new_tangents.map(|t| t[0]),
                            std::mem::take(&mut back_new_texcoords[0]),
                        ),
                        back_vertex_list.add_vertex(
                            back_new_positions[1],
                            back_new_normals.map(|n| n[1]),
                            back_new_tangents.map(|t| t[1]),
                            std::mem::take(&mut back_new_texcoords[1]),
                        ),
                    ];

                    front_triangles.push(Triangle {
                        indices: [front[0], front[1], front_new_vertex_indices[0]],
                    });
                    front_triangles.push(Triangle {
                        indices: [
                            front[0],
                            front_new_vertex_indices[0],
                            front_new_vertex_indices[1],
                        ],
                    });

                    back_triangles.push(Triangle {
                        indices: [
                            back_new_vertex_indices[0],
                            back[0],
                            back_new_vertex_indices[1],
                        ],
                    })
                }
                TrianglePlaneSide::Back2Front1 { front, back } => {
                    let back_positions = [
                        self.vertex_list.positions[triangle.indices[back[0]]],
                        self.vertex_list.positions[triangle.indices[back[1]]],
                    ];
                    let front_positions = [self.vertex_list.positions[triangle.indices[front[0]]]];

                    let contact_points = [
                        plane.point_on(back_positions[0], front_positions[0] - back_positions[0]),
                        plane.point_on(back_positions[1], front_positions[0] - back_positions[1]),
                    ];
                    let total_lengths = [
                        (front_positions[0] - back_positions[0]).len(),
                        (front_positions[0] - back_positions[1]).len(),
                    ];

                    const EPSILON: f32 = 1e-3;

                    let ratios = [
                        if total_lengths[0] <= EPSILON {
                            0.0
                        } else {
                            (contact_points[0] - back_positions[0]).len() / total_lengths[0]
                        },
                        if total_lengths[1] <= EPSILON {
                            0.0
                        } else {
                            (contact_points[1] - back_positions[1]).len() / total_lengths[1]
                        },
                    ];

                    let back_new_positions = [contact_points[0], contact_points[1]];
                    let front_new_positions = back_new_positions.clone();

                    let back_new_normals = match &self.vertex_list.normals {
                        Some(normals) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    normals[triangle.indices[back[0]] * 3 + 0],
                                    normals[triangle.indices[back[0]] * 3 + 1],
                                    normals[triangle.indices[back[0]] * 3 + 2],
                                ],
                                [
                                    normals[triangle.indices[back[0]] * 3 + 0],
                                    normals[triangle.indices[back[0]] * 3 + 1],
                                    normals[triangle.indices[back[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => {
                                let back_normals = [
                                    Vec3::new(
                                        normals[triangle.indices[back[0]] * 3 + 0],
                                        normals[triangle.indices[back[0]] * 3 + 1],
                                        normals[triangle.indices[back[0]] * 3 + 2],
                                    ),
                                    Vec3::new(
                                        normals[triangle.indices[back[1]] * 3 + 0],
                                        normals[triangle.indices[back[1]] * 3 + 1],
                                        normals[triangle.indices[back[1]] * 3 + 2],
                                    ),
                                ];
                                let front_normal = Vec3::new(
                                    normals[triangle.indices[front[0]] * 3 + 0],
                                    normals[triangle.indices[front[0]] * 3 + 1],
                                    normals[triangle.indices[front[0]] * 3 + 2],
                                );

                                let back_new_normals = [
                                    back_normals[0] * ratios[0] + front_normal * (1.0 - ratios[0]),
                                    back_normals[1] * ratios[1] + front_normal * (1.0 - ratios[1]),
                                ];

                                Some([
                                    [
                                        back_new_normals[0].x,
                                        back_new_normals[0].y,
                                        back_new_normals[0].z,
                                    ],
                                    [
                                        back_new_normals[1].x,
                                        back_new_normals[1].y,
                                        back_new_normals[1].z,
                                    ],
                                ])
                            }
                        },
                        None => None,
                    };
                    let front_new_normals = match &self.vertex_list.normals {
                        Some(normals) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    normals[triangle.indices[front[0]] * 3 + 0],
                                    normals[triangle.indices[front[0]] * 3 + 1],
                                    normals[triangle.indices[front[0]] * 3 + 2],
                                ],
                                [
                                    normals[triangle.indices[front[0]] * 3 + 0],
                                    normals[triangle.indices[front[0]] * 3 + 1],
                                    normals[triangle.indices[front[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => back_new_normals.clone(),
                        },
                        None => None,
                    };

                    let back_new_tangents = match &self.vertex_list.tangents {
                        Some(tangents) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    tangents[triangle.indices[back[0]] * 3 + 0],
                                    tangents[triangle.indices[back[0]] * 3 + 1],
                                    tangents[triangle.indices[back[0]] * 3 + 2],
                                ],
                                [
                                    tangents[triangle.indices[back[0]] * 3 + 0],
                                    tangents[triangle.indices[back[0]] * 3 + 1],
                                    tangents[triangle.indices[back[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => {
                                let back_tangents = [
                                    Vec3::new(
                                        tangents[triangle.indices[back[0]] * 3 + 0],
                                        tangents[triangle.indices[back[0]] * 3 + 1],
                                        tangents[triangle.indices[back[0]] * 3 + 2],
                                    ),
                                    Vec3::new(
                                        tangents[triangle.indices[back[1]] * 3 + 0],
                                        tangents[triangle.indices[back[1]] * 3 + 1],
                                        tangents[triangle.indices[back[1]] * 3 + 2],
                                    ),
                                ];
                                let front_tangent = Vec3::new(
                                    tangents[triangle.indices[front[0]] * 3 + 0],
                                    tangents[triangle.indices[front[0]] * 3 + 1],
                                    tangents[triangle.indices[front[0]] * 3 + 2],
                                );

                                let back_new_tangents = [
                                    back_tangents[0] * ratios[0]
                                        + front_tangent * (1.0 - ratios[0]),
                                    back_tangents[1] * ratios[1]
                                        + front_tangent * (1.0 - ratios[1]),
                                ];

                                Some([
                                    [
                                        back_new_tangents[0].x,
                                        back_new_tangents[0].y,
                                        back_new_tangents[0].z,
                                    ],
                                    [
                                        back_new_tangents[1].x,
                                        back_new_tangents[1].y,
                                        back_new_tangents[1].z,
                                    ],
                                ])
                            }
                        },
                        None => None,
                    };
                    let front_new_tangents = match &self.vertex_list.tangents {
                        Some(tangents) => match self.vertex_list.surface_shading {
                            SurfaceShading::Flat => Some([
                                [
                                    tangents[triangle.indices[front[0]] * 3 + 0],
                                    tangents[triangle.indices[front[0]] * 3 + 1],
                                    tangents[triangle.indices[front[0]] * 3 + 2],
                                ],
                                [
                                    tangents[triangle.indices[front[0]] * 3 + 0],
                                    tangents[triangle.indices[front[0]] * 3 + 1],
                                    tangents[triangle.indices[front[0]] * 3 + 2],
                                ],
                            ]),
                            SurfaceShading::Smooth => back_new_tangents.clone(),
                        },
                        None => None,
                    };

                    let mut back_new_texcoords = [
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                    ];
                    let mut front_new_texcoords = [
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                        Vec::with_capacity(self.vertex_list.texcoords.len()),
                    ];

                    for texcoords in &self.vertex_list.texcoords {
                        let back_texcoords = [
                            [
                                texcoords[triangle.indices[back[0]] * 2 + 0],
                                texcoords[triangle.indices[back[0]] * 2 + 1],
                            ],
                            [
                                texcoords[triangle.indices[back[1]] * 2 + 0],
                                texcoords[triangle.indices[back[1]] * 2 + 1],
                            ],
                        ];
                        let front_texcoords = [
                            texcoords[triangle.indices[front[0]] * 2 + 0],
                            texcoords[triangle.indices[front[0]] * 2 + 1],
                        ];

                        let new_texcoords = [
                            [
                                // u
                                back_texcoords[0][0] * ratios[0]
                                    + front_texcoords[0] * (1.0 - ratios[0]),
                                // v
                                back_texcoords[0][1] * ratios[0]
                                    + front_texcoords[1] * (1.0 - ratios[0]),
                            ],
                            [
                                // u
                                back_texcoords[1][0] * ratios[0]
                                    + front_texcoords[0] * (1.0 - ratios[0]),
                                // v
                                back_texcoords[1][1] * ratios[0]
                                    + front_texcoords[1] * (1.0 - ratios[0]),
                            ],
                        ];

                        back_new_texcoords[0].push(new_texcoords[0]);
                        back_new_texcoords[1].push(new_texcoords[1]);

                        front_new_texcoords[0].push(new_texcoords[0]);
                        front_new_texcoords[1].push(new_texcoords[1]);
                    }

                    let back_new_vertex_indices = [
                        back_vertex_list.add_vertex(
                            back_new_positions[0],
                            back_new_normals.map(|n| n[0]),
                            back_new_tangents.map(|t| t[0]),
                            std::mem::take(&mut back_new_texcoords[0]),
                        ),
                        back_vertex_list.add_vertex(
                            back_new_positions[1],
                            back_new_normals.map(|n| n[1]),
                            back_new_tangents.map(|t| t[1]),
                            std::mem::take(&mut back_new_texcoords[1]),
                        ),
                    ];
                    let front_new_vertex_indices = [
                        front_vertex_list.add_vertex(
                            front_new_positions[0],
                            front_new_normals.map(|n| n[0]),
                            front_new_tangents.map(|t| t[0]),
                            std::mem::take(&mut front_new_texcoords[0]),
                        ),
                        front_vertex_list.add_vertex(
                            front_new_positions[1],
                            front_new_normals.map(|n| n[1]),
                            front_new_tangents.map(|t| t[1]),
                            std::mem::take(&mut front_new_texcoords[1]),
                        ),
                    ];

                    back_triangles.push(Triangle {
                        indices: [back[0], back[1], back_new_vertex_indices[0]],
                    });
                    back_triangles.push(Triangle {
                        indices: [
                            back[0],
                            back_new_vertex_indices[0],
                            back_new_vertex_indices[1],
                        ],
                    });

                    front_triangles.push(Triangle {
                        indices: [
                            front_new_vertex_indices[0],
                            front[0],
                            front_new_vertex_indices[1],
                        ],
                    })
                }
            }
        }

        let front = Self::new(
            self.material_id,
            self.hierarch_id,
            front_vertex_list,
            front_triangles,
        );
        let back = Self::new(
            self.material_id,
            self.hierarch_id,
            back_vertex_list,
            back_triangles,
        );
        let on_plane = Self::new(
            self.material_id,
            self.hierarch_id,
            on_plane_vertex_list,
            on_plane_triangles,
        );

        SplittedMesh {
            front,
            back,
            on_plane,
        }
    }
}
