//! Describes the SDF trait

use bevy::prelude::Vec3;

/// This trait describes any SDF
pub trait SDF {
    /// This function calculates the value of the SDF at a given point
    fn value_at_point(&self, point: &Vec3) -> f32;
}