//! The root SDF object
use crate::{sdf_operations::SDFOperators, sdf_primitives::SDFPrimitive};
use bevy::prelude::*;

/// A single SDF Element
#[derive(Debug, Clone)]
pub struct SDFElement {
    /// The SDF Primitive
    pub primitive: SDFPrimitive,
    /// The Translation of the primitive
    pub translation: Vec3,
    /// Rotation of the primitive
    pub rotation: Quat,
    /// Scale of the primitive
    pub scale: f32,
    /// Operation for joining the object with the previous object
    pub operation: SDFOperators,
}

impl Default for SDFElement {
    fn default() -> Self {
        Self {
            primitive: SDFPrimitive::Sphere(1.),
            translation: Default::default(),
            rotation: Default::default(),
            scale: 1.,
            operation: SDFOperators::Union,
        }
    }
}

impl SDFElement {
    /// Get the value of the SDF at a given point
    pub fn value_at_point(&self, point: &Vec3) -> f32 {
        let scale = self.scale;
        let rotation = self.rotation;
        let translation = self.translation;
        let transform =
            Mat4::from_scale_rotation_translation(Vec3::ONE * scale, rotation, translation)
                .inverse();
        self.primitive
            .value_at_point(&(transform.transform_point3(*point)))
            * scale
    }

    /// Get the value, taking into account previous values
    pub fn process_object_at_point(&self, point: &Vec3, previous: f32) -> f32 {
        let value = self.value_at_point(point);
        self.operation.value(&previous, &value)
    }
}

/// The root SDF object
#[derive(Debug, Clone)]
pub struct SDFObject {
    /// A list of all the elements in the SDF
    pub elements: Vec<SDFElement>,
}

impl SDFObject {
    /// Calculate the value of the SDF Object at a given point
    pub fn value_at_point(&self, point: &Vec3) -> f32 {
        self.elements.iter().fold(f32::INFINITY, |value, element| {
            element.process_object_at_point(point, value)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use assert_float_eq::*;
    use bevy::prelude::{EulerRot, Vec3};

    use super::*;
    use crate::sdf_primitives::SDFPrimitive;

    #[test]
    fn translates_a_sdf() {
        let sdf = SDFElement {
            translation: Vec3::X,
            ..default()
        };

        let interior = sdf.value_at_point(&Vec3::X);
        let surface = sdf.value_at_point(&Vec3::ZERO);
        let outside = sdf.value_at_point(&Vec3::new(-0.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_a_sdf() {
        let sdf = SDFElement {
            primitive: SDFPrimitive::Box(Vec3::new(1., 2., 1.)),
            rotation: Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.),
            ..default()
        };

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_and_transforms_sdf() {
        let sdf = SDFElement {
            primitive: SDFPrimitive::Box(Vec3::new(1., 2., 1.)),
            translation: Vec3::X,
            rotation: Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.),
            ..default()
        };

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_a_sdf() {
        let sdf = SDFElement {
            scale: 2.,
            ..default()
        };

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::X * 2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -2.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_transforms_and_rotates_a_sdf() {
        let sdf = SDFElement {
            primitive: SDFPrimitive::Box(Vec3::new(0.5, 1., 0.5)),
            translation: Vec3::X,
            rotation: Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.),
            scale: 2.,
            ..default()
        };

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn union_of_sdfs() {
        let sdf_a = SDFElement {
            translation: Vec3::X,
            ..default()
        };
        let sdf_b = SDFElement {
            translation: -1. * Vec3::X,
            ..default()
        };
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
        };

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
        let sdf_a = SDFElement {
            primitive: SDFPrimitive::Sphere(2.),
            ..default()
        };
        let sdf_b = SDFElement {
            primitive: SDFPrimitive::Sphere(1.),
            operation: SDFOperators::Subtraction,
            ..default()
        };
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
        };

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
    fn intersection_of_sdfs() {
        let sdf_a = SDFElement {
            primitive: SDFPrimitive::Sphere(2.),
            ..default()
        };
        let sdf_b = SDFElement {
            primitive: SDFPrimitive::Sphere(1.),
            operation: SDFOperators::Intersection,
            ..default()
        };
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
        };

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&Vec3::Y);
        let outside = sdf.value_at_point(&Vec3::new(1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }
}
