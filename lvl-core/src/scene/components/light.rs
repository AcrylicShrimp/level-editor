use lvl_math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub kind: LightKind,
    pub light_color: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightKind {
    Directional { direction: Vec3 },
    Point { position: Vec3 },
}
