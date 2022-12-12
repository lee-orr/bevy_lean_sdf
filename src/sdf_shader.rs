//! Pipeline for instanced SDF shader
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin, render_asset::RenderAssetPlugin,
        render_resource::*, mesh::{MeshVertexBufferLayout, MeshVertexAttribute},
    }, pbr::{MaterialPipeline, MaterialPipelineKey},
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
            .add_plugin(MaterialPlugin::<SDFShader>::default());
    }
}

///The component representing the instances for an SDF shader
#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "89f25d03-40b8-49f5-9468-675cd8fbe9b9"]
pub struct SDFShader {
    /// Image
    #[texture(0, dimension = "3d")]
    #[sampler(1)]
    pub image: Handle<Image>,
}

/// UV 3D Attribute
pub const ATTRIBUTE_UV_3D: MeshVertexAttribute =
MeshVertexAttribute::new("UV_3D", 463763473457, VertexFormat::Float32x3);

impl Material for SDFShader {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn fragment_shader() -> ShaderRef {
        "array_texture.wgsl".into()
    }
}
