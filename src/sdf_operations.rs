//! Describes the available SDF operations
use bevy::{prelude::*, transform};

use crate::sdf_trait::SDF;

/// The operations manipulating a single SDF
#[derive(Debug, Clone, PartialEq)]
pub enum SdfOperation<S> where S: SDF {
    /// Transform by translation & rotation
    Transform(S, Vec3, Quat),
    /// Exact scaling - identical in all dimensions
    Scale(S, f32)
}

impl<S> SDF for SdfOperation<S> where S: SDF{
    fn value_at_point(&self, point: &Vec3) -> f32 {
        let point = point.clone();
        match self {
            SdfOperation::Transform(sdf, translation, rotation) => transform_sdf(point.clone(), sdf, translation, rotation),
            SdfOperation::Scale(sdf, scale) => scale_sdf(point, sdf, *scale),
        }
    }
}

/// The operations combining SDFs
#[derive(Debug, Clone, PartialEq)]
pub enum SdfOperators<L,R> where L: SDF, R: SDF {
    /// A hard union between two SDFs
    Union(L,R),
    /// A hard subtraction between two SDFs - subtracting R from L
    Subtraction(L, R),
    /// A hard intersection between two SDFs
    Intersection(L, R)
}

impl<L, R> SDF for SdfOperators<L,R> where L: SDF, R: SDF {
    fn value_at_point(&self, point: &Vec3) -> f32 {
        let point = point.clone();
        match self {
            SdfOperators::Union(left, right) => union(point, left, right),
            SdfOperators::Subtraction(left, right) => subtraction(point, left, right),
            SdfOperators::Intersection(left, right) => intersection(point, left, right),
        }
    }
}

fn transform_sdf<S>(point: Vec3, sdf: &S, translation: &Vec3, rotation: &Quat) -> f32 where S: SDF {
    let transform = Mat4::from_scale_rotation_translation(Vec3::ONE, rotation.clone(), translation.clone()).inverse();
    sdf.value_at_point(&(transform.transform_point3(point)))
}

fn scale_sdf<S>(point: Vec3, sdf: &S, scale: f32) -> f32 where S: SDF {
    sdf.value_at_point(&(point / scale)) * scale
}

fn union<L,R>(point: Vec3, left: &L, right:&R) -> f32 where L: SDF, R:SDF {
    left.value_at_point(&point).min(right.value_at_point(&point))
}

fn subtraction<L,R>(point: Vec3, left: &L, right:&R) -> f32 where L: SDF, R:SDF {
    (left.value_at_point(&point)).max( -1. * right.value_at_point(&point))
}

fn intersection<L,R>(point: Vec3, left: &L, right:&R) -> f32 where L: SDF, R:SDF {
    left.value_at_point(&point).max(right.value_at_point(&point))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use bevy::prelude::EulerRot;
    use assert_float_eq::*;

    use super::*;
    use crate::sdf_primitives::SdfPrimitive;

    #[test]
    fn translates_a_sdf() {
        let sdf = SdfOperation::Transform(SdfPrimitive::Sphere(1.), Vec3::X, Quat::IDENTITY);

        let interior = sdf.value_at_point(&Vec3::X);
        let surface = sdf.value_at_point(&Vec3::ZERO);
        let outside = sdf.value_at_point(&Vec3::new(-0.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_a_sdf() {
        let sdf = SdfOperation::Transform(SdfPrimitive::Box(Vec3::new(1., 2., 1.)), Vec3::ZERO, Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.));

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_and_transforms_sdf() {
        let sdf = SdfOperation::Transform(SdfPrimitive::Box(Vec3::new(1., 2., 1.)), Vec3::X, Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.));

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));
        
        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_a_sdf() {
        let sdf = SdfOperation::Scale(SdfPrimitive::Sphere(1.), 2.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::X * 2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -2.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn union_of_sdfs() {
        let sdf_a = SdfOperation::Transform(SdfPrimitive::Sphere(1.), Vec3::X, Quat::IDENTITY);
        let sdf_b = SdfOperation::Transform(SdfPrimitive::Sphere(1.), -1. * Vec3::X, Quat::IDENTITY);
        let sdf = SdfOperators::Union(sdf_a, sdf_b);

        let interior_a = sdf.value_at_point(&Vec3::X);
        let interior_b = sdf.value_at_point(&(Vec3::X * -1.));
        let surface_a = sdf.value_at_point(&(Vec3::X * 2.));
        let surface_b = sdf.value_at_point(&(Vec3::X * -2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(interior_a, -1.);
        assert_float_absolute_eq!(interior_b, -1.);
        assert_float_absolute_eq!(surface_a, 0.);
        assert_float_absolute_eq!(surface_b, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    
    #[test]
    fn subtraction_of_sdfs() {
        let sdf_a = SdfPrimitive::Sphere(2.);
        let sdf_b = SdfPrimitive::Sphere(1.);
        let sdf = SdfOperators::Subtraction(sdf_a, sdf_b);

        let center = sdf.value_at_point(&Vec3::ZERO);
        let inner_surface = sdf.value_at_point(&Vec3::X);
        let mid_way = sdf.value_at_point(&(Vec3::X * 1.5));
        let outer_surface = sdf.value_at_point(&(Vec3::X * -2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(center, 1.);
        assert_float_absolute_eq!(mid_way, -0.5);
        assert_float_absolute_eq!(inner_surface, 0.);
        assert_float_absolute_eq!(outer_surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn calculates_sphere_sdf() {
        let sdf_a = SdfPrimitive::Sphere(2.);
        let sdf_b = SdfPrimitive::Sphere(1.);
        let sdf = SdfOperators::Intersection(sdf_a, sdf_b);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&Vec3::Y);
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }
}