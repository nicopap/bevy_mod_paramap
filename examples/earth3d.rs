use std::f32::consts::TAU;

use bevy::{
    asset::AssetPlugin,
    core_pipeline::bloom::BloomSettings,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::{camera::Projection, render_resource::TextureFormat},
    window::{close_on_esc, WindowPlugin},
};
#[cfg(feature = "inspector-def")]
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use bevy_mod_paramap::*;

const NORMAL_MAP: &str = "earth/normal_map.jpg";
const HEIGHT_MAP: &str = "earth/elevation_surface.jpg";
const ROUGH_MAP: &str = "earth/metallic_roughness.png";
const ALBEDO_MAP: &str = "earth/base_color.jpg";
const EMI_MAP: &str = "earth/emissive.jpg";

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Earth parallax mapping example".into(),
                    width: 756.0,
                    height: 574.0,
                    ..default()
                },
                ..default()
            })
            // Tell the asset server to watch for asset changes on disk:
            .set(AssetPlugin {
                watch_for_changes: !cfg!(target_arch = "wasm32"),
                ..default()
            }),
    )
    .add_plugin(ParallaxMaterialPlugin)
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
    .add_system(update_canvas_size)
    .add_system(close_on_esc);
    #[cfg(feature = "inspector-def")]
    app.add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Spin>();

    app.run();
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
fn update_canvas_size(mut windows: ResMut<Windows>) {
    let window_updated = windows.is_changed();
    #[cfg(not(target_arch = "wasm32"))]
    let update_window = || {};
    #[cfg(target_arch = "wasm32")]
    let mut update_window = || {
        let browser_window = web_sys::window()?;
        let window_width = browser_window.inner_width().ok()?.as_f64()?;
        let window_height = browser_window.inner_height().ok()?.as_f64()?;
        let window = windows.get_primary_mut()?;
        window.set_resolution(window_width as f32, window_height as f32);
        Some(())
    };
    if window_updated {
        update_window();
    }
}

#[derive(Component, PartialEq, Eq)]
struct Earth;

#[derive(Component, PartialEq)]
#[cfg_attr(feature = "inspector-def", derive(Inspectable))]
struct Spin(f32);

fn spin(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>) {
    for (mut transform, spin) in query.iter_mut() {
        transform.rotate_y(spin.0 * time.delta_seconds());
    }
}

/// Store handle of the earth normal to later modify its format
/// in [`update_normal`].
#[derive(Resource)]
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
    let normal_handle = assets.load(NORMAL_MAP);
    normal.0 = Some(normal_handle.clone());
    let mut sphere: Mesh = shape::UVSphere::default().into();
    sphere.generate_tangents().unwrap();
    commands
        .spawn(MaterialMeshBundle {
            transform: Transform::from_rotation(Quat::from_euler(XYZ, -TAU / 4.0, 0.0, TAU / 2.0)),
            mesh: meshes.add(sphere),
            material: materials.add(ParallaxMaterial {
                // reduce roughness set in the "earth/metallic_roughness.png" file
                perceptual_roughness: 0.75,
                // The base color. See README for source.
                base_color_texture: Some(assets.load(ALBEDO_MAP)),
                // Since emissive_texture value is multiplied by emissive, we use emissive
                // to reduce the intensity of the emissive_texture, so that the lights only
                // show up in earth's penumbra.
                emissive: Color::rgb_u8(30, 30, 30),
                // the nighttime visuals. See README for source.
                emissive_texture: Some(assets.load(EMI_MAP)),
                // The normal map generated from "earth/elevation_surface.png" using GIMP's
                // Filters -> Generic -> Normal Map filter.
                normal_map_texture: normal_handle,
                // See README for source.
                height_map: assets.load(HEIGHT_MAP),
                // Set the water to have a low roughness, while surface has high roughness.
                metallic_roughness_texture: Some(assets.load(ROUGH_MAP)),
                // How "deep" to displace stuff
                height_depth: 0.01,
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
        .insert((Earth, Spin(0.1), Name::new("Earth")));

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 500.0,
                ..default()
            },
            transform: Transform::from_xyz(2.0, 0.5, 2.0),
            ..default()
        })
        .with_children(|cmd| {
            let sphere = shape::Icosphere {
                radius: 0.05,
                subdivisions: 3,
            };
            cmd.spawn(PbrBundle {
                mesh: meshes.add(sphere.into()),
                ..default()
            });
        });

    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(3.9, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert((
            BloomSettings {
                intensity: 0.1,
                ..default()
            },
            PanOrbitCamera::default(),
        ));
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
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let right = MouseButton::Right;
    let left = MouseButton::Left;
    let middle = MouseButton::Middle;
    let rotation_speed = 0.001;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let wasm = cfg!(target_arch = "wasm32");
    let scroll = if wasm { 0.01 } else { 1.0 };
    let scroll = ev_scroll.iter().map(|e| e.y * scroll).sum::<f32>();

    let shift_held = keyboard.pressed(KeyCode::LShift) || keyboard.pressed(KeyCode::RShift);
    if (mouse.pressed(right) && !wasm) || (shift_held && mouse.pressed(left)) {
        rotation_move = ev_motion.iter().map(|ev| &ev.delta).sum();
    } else if mouse.pressed(middle) {
        pan = ev_motion.iter().map(|ev| &ev.delta).sum();
    }

    let right_press = !wasm && (mouse.just_released(right) || mouse.just_pressed(right));
    let left_press = shift_held && (mouse.just_released(left) || mouse.just_pressed(left));
    let orbit_button_changed = right_press || left_press;

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
            let sign = if pan_orbit.upside_down { -1.0 } else { 1.0 };
            let delta_x = sign * rotation_move.x * TAU * rotation_speed;
            let delta_y = rotation_move.y * TAU * rotation_speed / 2.0;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation * pitch;
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov)
                    * rotation_speed;
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
            // dont allow zoom to go bellow earth surface
            pan_orbit.radius = pan_orbit.radius.max(1.1).min(30.0);
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
