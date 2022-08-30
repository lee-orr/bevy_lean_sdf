//! Describes the available SDF primitives

use bevy::prelude::Vec3;

use crate::sdf_trait::SDF;

/// The basic primitives comprising an Signed Distance Field
#[derive(Debug, Clone, PartialEq)]
pub enum SdfPrimitive {
    /// Defines a sphere by it's radius
    Sphere(f32),
    /// Defines a box, provided it's half bounds
    Box(Vec3),
}

impl SDF for SdfPrimitive {
    fn value_at_point(&self, point: &Vec3) -> f32 {
        let point = point.clone();
        match self {
            SdfPrimitive::Sphere(radius) => sphere_sdf(point, *radius),
            SdfPrimitive::Box(bounds) => box_sdf(point, *bounds),
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
        let sdf = SdfPrimitive::Sphere(1.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&Vec3::Y);
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn calculates_box_sdf() {
        let sdf = SdfPrimitive::Box(Vec3::new(1., 2., 1.));

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y * 2.));
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }
}