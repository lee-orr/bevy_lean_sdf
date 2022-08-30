#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use bevy::prelude::*;

pub mod utils;

pub mod sdf_trait;
pub mod sdf_primitives;
pub mod sdf_operations;

/// A plugin
pub struct HelloWorldPlugin;

impl Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(hello_world);
    }
}

fn hello_world() {
    println!("Hello, World!");
}
