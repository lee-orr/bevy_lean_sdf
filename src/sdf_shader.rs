//! Pipeline for instanced SDF shader
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin, render_asset::RenderAssetPlugin,
        render_resource::*,
    },
};

use bytemuck::{Pod, Zeroable};

use crate::sdf_object::SDFObject;

/// The plugin enabling the SDF Instance Shader
pub struct SDFShaderPlugin;

impl Plugin for SDFShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<SDFObject>()
            .add_plugin(ExtractComponentPlugin::<Handle<SDFObject>>::default())
            .add_plugin(RenderAssetPlugin::<SDFObject>::default())
            .add_plugin(MaterialPlugin::<SDFRenderAsset>::default());
    }
}

///The component representing the instances for an SDF shader
#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "89f25d03-40b8-49f5-9468-675cd8fbe9b9"]
pub struct SDFRenderAsset {
    /// SDF Instance Data
    pub instance_data: Vec<SDFInstanceData>,
}

impl Material for SDFRenderAsset {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }
}

/// The data for a single instance in the SDF shader
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct SDFInstanceData {
    /// The instance's position
    pub position: Vec3,
}
