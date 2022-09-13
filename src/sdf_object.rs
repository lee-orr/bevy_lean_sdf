//! The root SDF object
use crate::{sdf_operations::SDFOperators, sdf_primitives::SDFPrimitive};
use bevy::prelude::*;

/// A single SDF Element
#[derive(Debug, Clone)]
pub struct SDFElement {
    /// The SDF Primitive
    pub primitive: SDFPrimitive,
    /// The transform matrix
    transform: Mat4,
    /// The transform matrix
    inverse: Mat4,
    // the scale
    scale: f32,
    /// Operation for joining the object with the previous object
    pub operation: SDFOperators,
}

impl Default for SDFElement {
    fn default() -> Self {
        let transform = Mat4::from_scale(Vec3::ONE);
        Self {
            primitive: SDFPrimitive::Sphere(1.),
            inverse: transform.inverse(),
            transform,
            scale: 1.,
            operation: SDFOperators::Union,
        }
    }
}

impl SDFElement {
    /// Create a new element - short for the default
    pub fn new() -> Self {
        Self::default()
    }

    /// Make `SDFElement` with a primitive
    pub fn with_primitive(mut self, primitive: SDFPrimitive) -> Self {
        self.primitive = primitive;
        self
    }

    /// Make `SDFElement` with an operation
    pub fn with_operation(mut self, operation: SDFOperators) -> Self {
        self.operation = operation;
        self
    }

    /// Make `SDFElement` with a translation
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        let (scale, rotation, _) = self.transform.to_scale_rotation_translation();
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inverse = self.transform.inverse();
        self
    }

    /// Make `SDFElement` with a rotation
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        let (scale, _, translation) = self.transform.to_scale_rotation_translation();
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inverse = self.transform.inverse();
        self
    }

    /// Make `SDFElement` with a scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        let (_, rotation, translation) = self.transform.to_scale_rotation_translation();
        let scale = scale.abs();
        self.scale = scale;
        let scale = scale * Vec3::ONE;
        self.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        self.inverse = self.transform.inverse();
        self
    }

    /// Get the value of the SDF at a given point
    pub fn value_at_point(&self, point: &Vec3) -> f32 {
        let scale = self.scale;
        let transform = self.inverse;
        self.primitive
            .value_at_point(&(transform.transform_point3(*point)))
            * scale
    }

    /// Get the value, taking into account previous values
    pub fn process_object_at_point(&self, point: &Vec3, previous: f32) -> f32 {
        let value = self.value_at_point(point);
        self.operation.value(&previous, &value)
    }

    /// Get the bounds of the element, potentially given a previous element
    pub fn get_bounds(&self, previous: &Option<(Vec3, Vec3)>) -> (Vec3, Vec3) {
        let bounds = self.primitive.get_bounds();
        let bounds = (
            self.transform.transform_point3(bounds.0),
            self.transform.transform_point3(bounds.1),
        );
        let mut bounds = (bounds.0.min(bounds.1), bounds.0.max(bounds.1));

        if let Some(previous) = previous {
            bounds = self.operation.get_bounds(previous, &bounds);
        }
        bounds
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

    /// Calculate SDF Object bounds
    pub fn get_bounds(&self) -> (Vec3, Vec3) {
        self.elements
            .iter()
            .fold(None, |value, element| Some(element.get_bounds(&value)))
            .unwrap_or((Vec3::ZERO, Vec3::ZERO))
    }

    /// Get the locations of boxes designed to cover the surface at a given size
    pub fn generate_boxes(&self, resolution: i32) -> (f32, Vec<Vec3>) {
        let bounds = self.get_bounds();
        let size = (bounds.1 - bounds.0).max_element();
        let box_size = size / (resolution as f32);
        let half_box_size = box_size / 2.;
        let mut boxes: Vec<Vec3> = Vec::new();
        for x in (0..resolution).map(|x| {
            let x = x as f32;
            bounds.0.x + x * box_size + half_box_size
        }) {
            for y in (0..resolution).map(|y| {
                let y = y as f32;
                bounds.0.y + y * box_size + half_box_size
            }) {
                for z in (0..resolution).map(|z| {
                    let z = z as f32;
                    bounds.0.z + z * box_size + half_box_size
                }) {
                    let point = Vec3::new(x, y, z);
                    let sdf = self.value_at_point(&point);
                    println!("Testing Point {} = {}", &point, &sdf);
                    if sdf <= half_box_size + f32::EPSILON && sdf >= -half_box_size - f32::EPSILON {
                        boxes.push(point);
                    }
                }
            }
        }
        (box_size, boxes)
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
        let sdf = SDFElement::default().with_translation(Vec3::X);

        let interior = sdf.value_at_point(&Vec3::X);
        let surface = sdf.value_at_point(&Vec3::ZERO);
        let outside = sdf.value_at_point(&Vec3::new(-0.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_a_sdf() {
        let sdf = SDFElement::default()
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.))
            .with_primitive(SDFPrimitive::Box(Vec3::new(1., 2., 1.)));

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn rotates_and_transforms_sdf() {
        let sdf = SDFElement::default()
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.))
            .with_primitive(SDFPrimitive::Box(Vec3::new(1., 2., 1.)))
            .with_translation(Vec3::X);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_a_sdf() {
        let sdf = SDFElement::default().with_scale(2.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::X * 2.));
        let outside = sdf.value_at_point(&Vec3::new(-2.5, 0., 0.));

        assert_float_absolute_eq!(interior, -2.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn scales_transforms_and_rotates_a_sdf() {
        let sdf = SDFElement::default()
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90. * PI / 180.))
            .with_primitive(SDFPrimitive::Box(Vec3::new(0.5, 1., 0.5)))
            .with_translation(Vec3::X)
            .with_scale(2.);

        let interior = sdf.value_at_point(&Vec3::ZERO);
        let surface = sdf.value_at_point(&(Vec3::Y));
        let outside = sdf.value_at_point(&Vec3::new(-1.5, 0., 0.));

        assert_float_absolute_eq!(interior, -1.);
        assert_float_absolute_eq!(surface, 0.);
        assert_float_absolute_eq!(outside, 0.5);
    }

    #[test]
    fn union_of_sdfs() {
        let sdf_a = SDFElement::default().with_translation(Vec3::X);
        let sdf_b = SDFElement::default().with_translation(-1. * Vec3::X);
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

    #[test]
    fn translates_bounds() {
        let sdf = SDFElement::default().with_translation(Vec3::X);

        let bounds = sdf.get_bounds(&None);

        assert_float_absolute_eq!(bounds.0.x, 0.);
        assert_float_absolute_eq!(bounds.0.y, -1.);
        assert_float_absolute_eq!(bounds.0.z, -1.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 1.);
        assert_float_absolute_eq!(bounds.1.z, 1.);
    }

    #[test]
    fn rotates_bounds() {
        let sdf = SDFElement::default()
            .with_primitive(SDFPrimitive::Box(Vec3::new(1., 2., 0.5)))
            .with_rotation(Quat::from_euler(EulerRot::XYZ, 0., 90. * PI / 180., 0.));

        let bounds = sdf.get_bounds(&None);

        assert_float_absolute_eq!(bounds.0.x, -0.5);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -1.);
        assert_float_absolute_eq!(bounds.1.x, 0.5);
        assert_float_absolute_eq!(bounds.1.y, 2.);
        assert_float_absolute_eq!(bounds.1.z, 1.);
    }

    #[test]
    fn scales_bounds() {
        let sdf = SDFElement::default().with_scale(2.);

        let bounds = sdf.get_bounds(&None);

        assert_float_absolute_eq!(bounds.0.x, -2.);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -2.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 2.);
        assert_float_absolute_eq!(bounds.1.z, 2.);
    }

    #[test]
    fn bounds_of_multiple_elements() {
        let sdf_a = SDFElement::default().with_translation(Vec3::X);
        let sdf_b = SDFElement::default().with_translation(-1. * Vec3::X);
        let sdf = SDFObject {
            elements: vec![sdf_a, sdf_b],
        };

        let bounds = sdf.get_bounds();

        assert_float_absolute_eq!(bounds.0.x, -2.);
        assert_float_absolute_eq!(bounds.0.y, -1.);
        assert_float_absolute_eq!(bounds.0.z, -1.);
        assert_float_absolute_eq!(bounds.1.x, 2.);
        assert_float_absolute_eq!(bounds.1.y, 1.);
        assert_float_absolute_eq!(bounds.1.z, 1.);
    }

    #[test]
    fn generate_boxes_on_surface() {
        let sdf_a = SDFElement::default().with_primitive(SDFPrimitive::Box(Vec3::ONE));
        let sdf = SDFObject {
            elements: vec![sdf_a],
        };

        let result = sdf.generate_boxes(3);
        assert_float_absolute_eq!(result.0, 2. / 3.);
        assert_eq!(result.1.len(), 9 * 2 + 8);
    }
}
