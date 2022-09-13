#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;

pub mod utils;

pub mod sdf_object;
pub mod sdf_operations;
pub mod sdf_primitives;

/// A plugin
pub struct SDFPlugin;

impl Plugin for SDFPlugin {
    fn build(&self, _app: &mut App) {}
}
