//! Describes the available SDF primitives

use bevy::prelude::Vec3;

/// The basic primitives comprising an Signed Distance Field
#[derive(Debug, Clone, PartialEq)]
pub enum SDFPrimitive {
    /// Defines a sphere by it's radius
    Sphere(f32),
    /// Defines a box, provided it's half bounds
    Box(Vec3),
}

impl SDFPrimitive {
    /// Get the value of the SDF at a given point
    pub fn value_at_point(&self, point: &Vec3) -> f32 {
        let point = *point;
        match self {
            SDFPrimitive::Sphere(radius) => sphere_sdf(point, *radius),
            SDFPrimitive::Box(bounds) => box_sdf(point, *bounds),
        }
    }

    /// Get the bounds of the SDF
    pub fn get_bounds(&self) -> (Vec3, Vec3) {
        match self {
            SDFPrimitive::Sphere(radius) => (-1. * Vec3::ONE * *radius, Vec3::ONE * *radius),
            SDFPrimitive::Box(bounds) => (-1. * *bounds, *bounds),
        }
    }
}

fn sphere_sdf(point: Vec3, radius: f32) -> f32 {
    point.length() - radius
}

fn box_sdf(point: Vec3, bounds: Vec3) -> f32 {
    let q = point.abs() - bounds;
    q.max(Vec3::ZERO).length() + q.y.max(q.z).max(q.x).min(0.)
}

#[cfg(test)]
mod tests {
    use assert_float_eq::*;

    use super::*;

    #[test]
    fn calculates_sphere_sdf() {
        let sdf = SDFPrimitive::Sphere(1.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&Vec3::Y);
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn calculates_box_sdf() {
        let sdf = SDFPrimitive::Box(Vec3::new(1., 2., 1.));

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y * 2.));
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn calculate_sphere_bounds() {
        let sdf = SDFPrimitive::Sphere(2.);
        let bounds = sdf.get_bounds();
        assert_float_absolute_eq!(bounds.0.x, -2.);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -2.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 2.);
        assert_float_absolute_eq!(bounds.1.z, 2.);
    }

    #[test]
    fn calculate_box_bounds() {
        let sdf = SDFPrimitive::Box(Vec3::new(1.5, 1., 2.));
        let bounds = sdf.get_bounds();
        assert_float_absolute_eq!(bounds.0.x, -1.5);
        assert_float_absolute_eq!(bounds.0.y, -1.);
        assert_float_absolute_eq!(bounds.0.z, -2.);
        assert_float_absolute_eq!(bounds.1.x, 1.5);
        assert_float_absolute_eq!(bounds.1.y, 1.);
        assert_float_absolute_eq!(bounds.1.z, 2.);
    }
}
