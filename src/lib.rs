#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;
use sdf_shader::SDFShaderPlugin;

pub mod utils;

pub mod sdf_object;
pub mod sdf_operations;
pub mod sdf_primitives;
pub mod sdf_shader;

/// A plugin
pub struct SDFPlugin;

impl Plugin for SDFPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SDFShaderPlugin);
    }
}
