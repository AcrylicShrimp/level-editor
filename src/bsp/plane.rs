use super::Vec3;
use zerocopy::AsBytes;

const EPSILON: f32 = 1e-3;

#[repr(C)]
#[derive(AsBytes, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneSide {
    Front,
    Back,
    OnPlane,
}

#[repr(C)]
#[derive(AsBytes, Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, point: Vec3) -> Self {
        let normal = normal.normalized();
        let distance = -Vec3::dot(normal, point);
        Self { normal, distance }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        Vec3::dot(self.normal, point) + self.distance
    }

    pub fn point_side(&self, point: Vec3) -> PlaneSide {
        let distance = self.distance_to_point(point);

        match distance {
            d if d < -EPSILON => PlaneSide::Back,
            d if EPSILON < d => PlaneSide::Front,
            _ => PlaneSide::OnPlane,
        }
    }

    pub fn point_on(&self, point: Vec3, direction: Vec3) -> Vec3 {
        point + direction * (self.distance_to_point(point) / Vec3::dot(self.normal, direction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_1() {
        let plane_normal = Vec3::new(0.0, 1.0, 0.0);
        let point_on_plane = Vec3::new(0.0, 0.0, 0.0);
        let plane = Plane::new(plane_normal, point_on_plane);
        assert_eq!(plane.point_side(point_on_plane), PlaneSide::OnPlane);
    }

    #[test]
    fn test_plane_2() {
        let plane_normal = Vec3::new(1.0, 1.0, 1.0).normalized();
        let point_on_plane = Vec3::new(4.0, -5.0, 2.0);
        let plane = Plane::new(plane_normal, point_on_plane);
        assert_eq!(plane.point_side(point_on_plane), PlaneSide::OnPlane);
    }

    #[test]
    fn test_plane_point_side_1() {
        let plane_normal = Vec3::new(0.0, 1.0, 0.0);
        let point_on_plane = Vec3::new(0.0, 0.0, 0.0);
        let plane = Plane::new(plane_normal, point_on_plane);

        let test_point = Vec3::new(3.0, 1.0, 2.0);
        assert_eq!(plane.point_side(test_point), PlaneSide::Front);
    }

    #[test]
    fn test_plane_point_side_2() {
        let plane_normal = Vec3::new(1.0, 1.0, 1.0).normalized();
        let point_on_plane = Vec3::new(4.0, -5.0, 2.0);
        let plane = Plane::new(plane_normal, point_on_plane);

        let test_point = Vec3::new(3.5, 5.5, -8.0);
        assert_eq!(plane.point_side(test_point), PlaneSide::OnPlane);
    }
}
