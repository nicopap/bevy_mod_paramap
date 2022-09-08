use bevy::{
    asset::AssetServerSettings, prelude::*, render::camera::ScalingMode, window::close_on_esc,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_paramap::*;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "simple cube".into(),
        width: 756.0,
        height: 574.0,

        ..default()
    })
    // Tell the asset server to watch for asset changes on disk:
    .insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(WorldInspectorPlugin::new())
    .add_plugin(ParallaxMaterialPlugin)
    .add_startup_system(setup)
    .add_system(spin_cube)
    .add_system(close_on_esc);

    app.run();
}

#[derive(Component)]
struct Spin;

fn spin_cube(time: Res<Time>, mut query: Query<&mut Transform, With<Spin>>) {
    for mut transform in query.iter_mut() {
        transform.rotate_local_y(0.3 * time.delta_seconds());
        transform.rotate_local_x(0.3 * time.delta_seconds());
        transform.rotate_local_z(0.3 * time.delta_seconds());
    }
}

fn setup(
    mut cmd: Commands,
    mut mats: ResMut<Assets<ParallaxMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<AssetServer>,
) {
    // Orthographic camera
    cmd.spawn_bundle(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // light
    cmd.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        point_light: PointLight {
            intensity: 3000.0,
            ..default()
        },
        ..default()
    });

    // Cube
    let mut cube: Mesh = shape::Cube { size: 1.0 }.into();
    cube.generate_tangents().unwrap();
    cmd.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(cube),
        material: mats.add(ParallaxMaterial {
            base_color_texture: Some(assets.load("paramap_color.jpg")),
            normal_map_texture: Some(assets.load("paramap_normal.jpg")),
            height_map: assets.load("paramap_bump.jpg"),
            ..default()
        }),
        ..default()
    })
    .insert(Spin);
}
