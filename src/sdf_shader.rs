//! Pipeline for instanced SDF shader
use bevy::{
    asset::Asset,
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{
        MaterialPipeline, MaterialPipelineKey, MeshPipeline, MeshPipelineKey, MeshUniform,
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin,
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::{RenderAssetPlugin, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::ExtractedView,
        RenderApp, RenderStage,
    },
};
use bytemuck::{Pod, Zeroable};
use bitflags::bitflags;

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