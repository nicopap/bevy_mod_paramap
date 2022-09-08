use bevy::{asset::AssetServerSettings, prelude::*, window::close_on_esc};
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
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    })
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
    mut std_mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<AssetServer>,
) {
    // Camera
    cmd.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(1.5, 1.5, 1.5).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // light
    cmd.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(1.8, 0.7, -1.1),
        point_light: PointLight {
            intensity: 226.0,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    })
    .with_children(|cmd| {
        let sphere = shape::Icosphere {
            radius: 0.05,
            subdivisions: 3,
        };
        cmd.spawn_bundle(PbrBundle {
            mesh: meshes.add(sphere.into()),
            ..default()
        });
    });

    // Plane
    cmd.spawn_bundle(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 10.0 }.into()),
        material: std_mats.add(StandardMaterial {
            perceptual_roughness: 0.45,
            reflectance: 0.18,
            ..Color::rgb_u8(0, 80, 0).into()
        }),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    });
    // Cube
    let mut cube: Mesh = shape::Cube { size: 1.0 }.into();
    cube.generate_tangents().unwrap();
    cmd.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(cube),
        material: mats.add(ParallaxMaterial {
            perceptual_roughness: 0.1,
            base_color_texture: Some(assets.load("paramap_color.jpg")),
            normal_map_texture: assets.load("paramap_normal.jpg"),
            height_map: assets.load("paramap_bump.jpg"),
            height_depth: 0.2,
            relief_mapping: true,
            max_height_layers: 64.0,
            ..default()
        }),
        ..default()
    })
    .insert(Spin);
}
