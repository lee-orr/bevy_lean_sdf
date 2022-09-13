use bevy::prelude::*;
use template_lib::{
    sdf_object::{SDFElement, SDFObject},
    sdf_operations::SDFOperators,
    sdf_primitives::SDFPrimitive,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    let sdf = SDFObject::default()
        .with_element(SDFElement::default().with_primitive(SDFPrimitive::Sphere(2.)))
        .with_element(
            SDFElement::default()
                .with_primitive(SDFPrimitive::Box(Vec3::ONE))
                .with_translation(Vec3::Z * 2.)
                .with_operation(SDFOperators::Subtraction),
        );
    let boxes = sdf.generate_lod_boxes(8, 6, 0.1);
    if let Some((size, boxes)) = boxes.last() {
        let scale = Vec3::ONE * *size;
        for b in boxes.iter().flatten() {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(b.x, b.y, b.z).with_scale(scale),
                ..default()
            });
        }
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
