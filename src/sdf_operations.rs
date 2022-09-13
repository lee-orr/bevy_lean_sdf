//! Describes the available SDF operations
use bevy::prelude::*;

/// The operations combining SDFs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SDFOperators {
    /// A hard union between two SDFs
    Union,
    /// A hard subtraction between two SDFs - subtracting R from L
    Subtraction,
    /// A hard intersection between two SDFs
    Intersection,
}

impl SDFOperators {
    /// Process the value from two SDFs using the operator
    pub fn value(&self, left: &f32, right: &f32) -> f32 {
        match self {
            SDFOperators::Union => union(left, right),
            SDFOperators::Subtraction => subtraction(left, right),
            SDFOperators::Intersection => intersection(left, right),
        }
    }

    /// Process the bounds of two SDFs
    pub fn get_bounds(&self, left: &(Vec3, Vec3), right: &(Vec3, Vec3)) -> (Vec3, Vec3) {
        match self {
            SDFOperators::Union => (left.0.min(right.0), left.1.max(right.1)),
            SDFOperators::Subtraction => (left.0, left.1),
            SDFOperators::Intersection => (left.0.max(right.0), left.1.min(right.1)),
        }
    }
}

fn union(left: &f32, right: &f32) -> f32 {
    left.min(*right)
}

fn subtraction(left: &f32, right: &f32) -> f32 {
    left.max(-1. * right)
}

fn intersection(left: &f32, right: &f32) -> f32 {
    left.max(*right)
}

#[cfg(test)]
mod test {
    use assert_float_eq::*;

    use super::*;

    #[test]
    pub fn union_gets_minimum() {
        let result = SDFOperators::Union.value(&2., &1.);

        assert_float_absolute_eq!(result, 1.);
    }

    #[test]
    pub fn subtracting_two_positive_values_gets_max() {
        let result = SDFOperators::Subtraction.value(&2., &1.);

        assert_float_absolute_eq!(result, 2.);
    }

    #[test]
    pub fn subtracting_positive_from_negative_gets_min() {
        let result = SDFOperators::Subtraction.value(&-2., &1.);

        assert_float_absolute_eq!(result, -1.)
    }

    #[test]
    pub fn subtracting_negative_from_negative_gets_abs_of_subtracted() {
        let result = SDFOperators::Subtraction.value(&-2., &-1.);

        assert_float_absolute_eq!(result, 1.)
    }

    #[test]
    pub fn subtracting_negative_from_position_gets_max_abs() {
        let result = SDFOperators::Subtraction.value(&2., &-1.);

        assert_float_absolute_eq!(result, 2.)
    }

    #[test]
    pub fn intersecting_negative_and_negative_gets_min_abs() {
        let result = SDFOperators::Intersection.value(&-2., &-1.);
        assert_float_absolute_eq!(result, -1.);
    }

    #[test]
    pub fn intersecting_negative_and_positive_gets_positive() {
        let result = SDFOperators::Intersection.value(&-2., &1.);
        assert_float_absolute_eq!(result, 1.);
    }

    #[test]
    pub fn intersecting_positive_and_positive_gets_max() {
        let result = SDFOperators::Intersection.value(&2., &1.);
        assert_float_absolute_eq!(result, 2.);
    }

    #[test]
    pub fn union_bounds_encompass_both_bounds() {
        let bounds = SDFOperators::Union.get_bounds(
            &(Vec3::new(-1., -2., -0.5), Vec3::new(1., 0., 0.5)),
            &(Vec3::new(0., -1., -1.5), Vec3::new(1.5, 2., 0.5)),
        );

        assert_float_absolute_eq!(bounds.0.x, -1.);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -1.5);
        assert_float_absolute_eq!(bounds.1.x, 1.5);
        assert_float_absolute_eq!(bounds.1.y, 2.);
        assert_float_absolute_eq!(bounds.1.z, 0.5);
    }

    #[test]
    pub fn subtracted_bounds_are_subtractee_bounds() {
        let bounds = SDFOperators::Subtraction.get_bounds(
            &(Vec3::new(-1., -2., -0.5), Vec3::new(1., 0., 0.5)),
            &(Vec3::new(0., -1., -1.5), Vec3::new(1.5, 2., 0.5)),
        );

        assert_float_absolute_eq!(bounds.0.x, -1.);
        assert_float_absolute_eq!(bounds.0.y, -2.);
        assert_float_absolute_eq!(bounds.0.z, -0.5);
        assert_float_absolute_eq!(bounds.1.x, 1.);
        assert_float_absolute_eq!(bounds.1.y, 0.);
        assert_float_absolute_eq!(bounds.1.z, 0.5);
    }

    #[test]
    pub fn intersection_bounds_are_intersection_of_bounds() {
        let bounds = SDFOperators::Intersection.get_bounds(
            &(Vec3::new(-1., -2., -0.5), Vec3::new(1., 0., 0.5)),
            &(Vec3::new(0., -1., -1.5), Vec3::new(1.5, 2., 0.5)),
        );

        assert_float_absolute_eq!(bounds.0.x, 0.);
        assert_float_absolute_eq!(bounds.0.y, -1.);
        assert_float_absolute_eq!(bounds.0.z, -0.5);
        assert_float_absolute_eq!(bounds.1.x, 1.);
        assert_float_absolute_eq!(bounds.1.y, 0.);
        assert_float_absolute_eq!(bounds.1.z, 0.5);
    }
}
