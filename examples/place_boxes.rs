use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{view::NoFrustumCulling, renderer::RenderQueue},
};
use template_lib::{
    sdf_instanced_shader::{SDFInstanceAsset, SDFInstanceData, SDFInstancedShaderPlugin},
    sdf_object::{SDFElement, SDFObject},
    sdf_operations::SDFOperators,
    sdf_primitives::SDFPrimitive,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SDFInstancedShaderPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    queue: Res<RenderQueue>
) {
    let sdf = SDFObject::default()
        .with_element(SDFElement::default().with_primitive(SDFPrimitive::Sphere(2.)))
        .with_element(
            SDFElement::default()
                .with_primitive(SDFPrimitive::Box(Vec3::ONE))
                .with_translation(Vec3::Z * 2.)
                .with_operation(SDFOperators::Subtraction),
        );
    let boxes = sdf.generate_lod_boxes(8, 4, 0.5);
    if let Some((size, boxes)) = boxes.last() {
        let mesh = meshes.add(Mesh::from(shape::Cube { size: *size }));
        let half_size = Vec3::ONE * *size / 2.;
        commands.spawn_bundle((
            mesh.clone(),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            ComputedVisibility::default(),
            NoFrustumCulling,
            SDFInstanceAsset(
                boxes
                    .iter()
                    .flatten()
                .filter_map(|b| {
                    let texture = sdf.generate_texture(8, &(*b - half_size, *b + half_size));
                            Some(SDFInstanceData { position: *b })
                    })
                    .collect(),
            ),
        ));
        //        for b in boxes.iter().flatten() {
        //            commands.spawn_bundle(PbrBundle {
        //                mesh: mesh.clone(),
        //                material: material.clone(),
        //                transform: Transform::from_xyz(b.x, b.y, b.z),
        //                ..default()
        //            });
        //        }
    }

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
