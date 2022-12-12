use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use template_lib::{
    sdf_object::{SDFElement, SDFObject},
    sdf_operations::SDFOperators,
    sdf_primitives::SDFPrimitive,
    sdf_shader::SDFShaderPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                present_mode: PresentMode::AutoNoVsync,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(SDFShaderPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sdfs: ResMut<Assets<SDFObject>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 0.1 }));
    let mat = materials.add(StandardMaterial::from(Color::GOLD));
    let image = images.add(Image::default());
    let sdf = (SDFObject {
        mesh_handle: Some(mesh.clone()),
        image_handle: Some(image.clone()),
        ..default()
    })
    .with_element(SDFElement::default().with_primitive(SDFPrimitive::Sphere(2.)))
    .with_element(
        SDFElement::default()
            .with_primitive(SDFPrimitive::Box(Vec3::ONE))
            .with_translation(Vec3::Z * 2.)
            .with_operation(SDFOperators::Subtraction),
    );

    let (sdf_mesh, sdf_image) = sdf.generate_mesh_and_texture(8, 1, 0.5);

    let _ = meshes.set(mesh.clone(), sdf_mesh);
    let _ = images.set(image, sdf_image);
    let sdf = sdfs.add(sdf);
    commands.spawn((
        mesh,
        mat,
        sdf,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
