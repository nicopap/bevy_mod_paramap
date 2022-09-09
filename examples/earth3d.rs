use std::f32::consts::TAU;

use bevy::{
    asset::AssetServerSettings,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::{camera::Projection, render_resource::TextureFormat},
    window::close_on_esc,
};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use bevy_mod_paramap::*;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Earth parallax mapping example".into(),
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
    .register_inspectable::<Spin>()
    .insert_resource(AmbientLight {
        color: Color::BLACK,
        brightness: 0.01,
    })
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Normal(None))
    .add_startup_system(setup)
    .add_system(pan_orbit_camera)
    .add_system(update_normal)
    .add_system(spin)
    .add_system(close_on_esc);

    app.run();
}

#[derive(Component, PartialEq, Eq)]
struct Earth;

#[derive(Component, PartialEq, Inspectable)]
struct Spin(f32);

fn spin(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>) {
    for (mut transform, spin) in query.iter_mut() {
        transform.rotate_y(spin.0 * time.delta_seconds());
    }
}

/// Store handle of the earth normal to later modify its format
/// in [`update_normal`].
struct Normal(Option<Handle<Image>>);

/// Work around the fact that the default bevy image loader sets the
/// normal's format to something incompatible with normal shaders.
/// The format must be one of the `TextureFormat` ending in `*Unorm`.
///
/// In this function, we wait until the image is loaded, immediately
/// change its format and never run the core logic afterward.
///
/// Without proper format, it looks like the light source moves as the
/// earth move, and there is major glitchy artifacts on the poles.
fn update_normal(
    mut already_ran: Local<bool>,
    mut images: ResMut<Assets<Image>>,
    normal: Res<Normal>,
) {
    if *already_ran {
        return;
    }
    if let Some(normal) = normal.0.as_ref() {
        if let Some(mut image) = images.get_mut(normal) {
            image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
            *already_ran = true;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParallaxMaterial>>,
    mut normal: ResMut<Normal>,
    assets: Res<AssetServer>,
) {
    use bevy::math::EulerRot::XYZ;
    let normal_handle = assets.load("earth/normal_map.jpg");
    normal.0 = Some(normal_handle.clone());
    let mut sphere: Mesh = shape::UVSphere::default().into();
    sphere.generate_tangents().unwrap();
    commands
        .spawn_bundle(MaterialMeshBundle {
            transform: Transform::from_rotation(Quat::from_euler(XYZ, -TAU / 4.0, 0.0, TAU / 2.0)),
            mesh: meshes.add(sphere),
            material: materials.add(ParallaxMaterial {
                // reduce roughness set in the "earth/metallic_roughness.png" file
                perceptual_roughness: 0.75,
                // The base color. See README for source.
                base_color_texture: Some(assets.load("earth/base_color.jpg")),
                // Since emissive_texture value is multiplied by emissive, we use emissive
                // to reduce the intensity of the emissive_texture, so that the lights only
                // show up in earth's penumbra.
                emissive: Color::rgb_u8(30, 30, 30),
                // the nighttime visuals. See README for source.
                emissive_texture: Some(assets.load("earth/emissive.jpg")),
                // The normal map generated from "earth/elevation_surface.png" using GIMP's
                // Filters -> Generic -> Normal Map filter.
                normal_map_texture: normal_handle,
                // See README for source.
                height_map: assets.load("earth/elevation_surface.png"),
                // Set the water to have a low roughness, while surface has high roughness.
                metallic_roughness_texture: Some(assets.load("earth/metallic_roughness.png")),
                // How "deep" to displace stuff
                height_depth: 0.015,
                // Use the quality algo, for show.
                algorithm: ParallaxAlgo::ReliefMapping,
                // This is an unreasonably high value, but since we expect to inspect up close
                // the surface of the texture, we need to set the max_height_layers pretty high.
                max_height_layers: 128.0,
                flip_normal_map_y: false,
                ..default()
            }),
            ..default()
        })
        .insert_bundle((Earth, Spin(0.15), Name::new("Earth")));

    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 500.0,
                ..default()
            },
            transform: Transform::from_xyz(1.5, 1.5, 1.5),
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

    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(3.9, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PanOrbitCamera::default());
}

///
/// Camera panning taken from <https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html>
///

#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}
