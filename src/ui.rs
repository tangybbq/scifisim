//! The basic UI.
//!
//! At this level, we display some information about the scene.  This sets up
//! its own 2d camera to overlay this information on any other camera.

use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    color::palettes::css::GOLD,
    pbr::wireframe::WireframeConfig,
    prelude::*,
    scene::SceneInstanceReady,
};
use std::io::Write;

// use bevy::pbr::wireframe::Wireframe;

use crate::{
    ship::RcsMode,
    solar::{AttitudeState, OrbitalBody, SizedBody},
};

pub const UI_LAYER: RenderLayers = RenderLayers::layer(8);
pub const BALL_LAYER: RenderLayers = RenderLayers::layer(7);

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct InfoText;

#[derive(Default)]
pub struct UIPlugin;

#[derive(Component)]
pub struct BallMarker;

#[derive(Component)]
pub struct MarkerMarker;

#[derive(Component)]
pub struct MainCameraMarker;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
        app.add_systems(Update, update_ui);
    }
}

fn setup_ui(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 2D camera for UI.
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 8,
            ..default()
        },
        UI_LAYER,
        Name::new("UI Camera"),
    ));

    // FPS text.
    commands
        .spawn((
            Text::new("FPS: "),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 2430.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
            UI_LAYER,
            Name::new("FPS Text"),
        ))
        .with_child((
            Text::new("50"),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 24.0,
                ..default()
            },
            TextColor(GOLD.into()),
            FpsText,
        ));

    // Informative text.
    commands.spawn((
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 24.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        UI_LAYER,
        Name::new("Info Text"),
        InfoText,
    ));

    // The ball gets its own 3d camera.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 7,
            clear_color: ClearColorConfig::None,
            viewport: Some(Viewport {
                physical_position: UVec2::new(10, 10),
                physical_size: UVec2::new(200, 200),
                ..default()
            }),
            ..default()
        },
        BALL_LAYER,
        Name::new("Ball Camera"),
        Transform::from_xyz(0.0, -250.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        Projection::Orthographic(OrthographicProjection::default_3d()),
    ));

    // Throw in a sphere to see if I can render it.
    let ball_mesh = Sphere {
        radius: 100.0,
        ..default()
    }
    .mesh()
    .uv(24, 24);

    let ball_tex = asset_server.load("tex/navball_surface_2048x1024.png");

    let ball_material = materials.add(StandardMaterial {
        // base_color: GREEN.into(),
        // base_color: Color::linear_rgb(1.0, 0.4, 0.2),
        base_color_texture: Some(ball_tex),
        // metallic: 1.0,
        perceptual_roughness: 0.85,
        reflectance: 0.02,
        // unlit: true
        // cull_mode: None,
        ..default()
    });

    let chartreuse_material = materials.add(StandardMaterial {
        base_color: Color::srgb_from_array([0.5, 1.0, 0.0]),
        perceptual_roughness: 0.85,
        reflectance: 0.02,
        unlit: false,
        // metallic: 1.0,
        cull_mode: None,
        ..default()
    });

    let ball = meshes.add(ball_mesh);
    commands.spawn((
        Mesh3d(ball),
        BALL_LAYER,
        Visibility::Hidden,
        MeshMaterial3d(ball_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        // Visibility::Hidden,
        // Wireframe,
        // BallMarker,
        Name::new("Ball"),
    ));

    // Instead of a ball, we can render some arrows to make the scene more obvious.
    commands
        .spawn((
            SceneRoot(asset_server.load("models/arrows.glb#Scene0")),
            BallMarker,
            BALL_LAYER,
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(100.0)),
        ))
        .observe(tag_scene_layers);

    commands.insert_resource(WireframeConfig {
        global: false,
        ..default()
    });

    commands.init_resource::<RcsMode>();

    let prograde_mesh: Handle<Mesh> =
        asset_server.load("models/marker-prograde.glb#Mesh0/Primitive0");

    commands.spawn((
        // SceneRoot(
        //     asset_server
        //         .load(GltfAssetLabel::Scene(0).from_asset("models/marker-prograde.glb#Scene0")),
        // ),
        Mesh3d(prograde_mesh),
        BALL_LAYER,
        // Wireframe,
        Transform::from_xyz(0.0, -100.0, 0.0).with_scale(Vec3::splat(100.0)), //     .with_rotation(Quat::from_euler(
        //         EulerRot::XYZ,
        //         // -std::f32::consts::FRAC_PI_2,
        //         0.0,
        //         0.0,
        //         0.0,
        MeshMaterial3d(chartreuse_material.clone()),
        MarkerMarker,
        GlobalTransform::default(),
    ));

    let vignetter_image = asset_server.load("tex/vignette_512.png");
    commands
        .spawn((
            Node {
                left: px(0.0),
                top: px(0.0),
                width: percent(100.0),  // Twice the radius.
                height: percent(100.0), // Twice the radius.
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                // position_type: PositionType::Absolute,
                // left: percent(50.0) - px(100.0),
                // top: percent(50.0) - px(100.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_child((
            Node {
                width: px(200.0),  // Twice the radius.
                height: px(200.0), // Twice the radius.
                ..Default::default()
            },
            ImageNode {
                image: vignetter_image,
                ..Default::default()
            },
        ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(0.0, 1.0, 0.0), Vec3::X),
        BALL_LAYER,
        Name::new("Ball Light"),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });

    // The main 3d scene.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Name::new("Main 3D Camera"),
        Transform::from_xyz(0.0, -2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_3,
            near: 1.0,
            far: 1_000_000.0,
            ..default()
        }),
        MainCameraMarker,
    ));

    // And some light for the ship
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(0.0, 2.0, 10.5).normalize(), Vec3::Z),
        Name::new("Main Light"),
    ));
}

/// Put scenes from the UI into our layer.
fn tag_scene_layers(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    transforms: Query<&Transform, Without<RenderLayers>>,
    query: Query<(Entity, &RenderLayers)>,
) {
    let Ok((parent, render_layers)) = query.get(trigger.entity) else {
        return;
    };
    children.iter_descendants(parent).for_each(|entity| {
        if transforms.contains(entity) {
            commands.entity(entity).insert(render_layers.clone());
        }
    });
}

// fn tag_scene_layers(
//     mut commands: Commands,
//     scenes: Query<(Entity, &SceneRoot), Without<RenderLayers>>,
// ) {
//     for (ent, _scene) in scenes.iter() {
//         commands.entity(ent).insert(BALL_LAYER);
//     }
// }

fn update_ui(
    mut text: Query<&mut Text, With<InfoText>>,
    time: Res<Time<Virtual>>,
    ship: Query<(&OrbitalBody, &AttitudeState), With<crate::ship::PlayerShip>>,
    earth: Query<(&OrbitalBody, &SizedBody, &AttitudeState), With<crate::solar::EarthMarker>>,
    mut ball: Query<&mut Transform, With<BallMarker>>,
    mut marker: Query<&mut Transform, (With<MarkerMarker>, Without<BallMarker>)>,
    rcs: Res<RcsMode>,
) {
    let seconds = time.elapsed_secs_f64();
    let (ship, ship_attitude) = ship.single().unwrap();
    let (earth, earth_size, _earth_attitude) = earth.single().unwrap();
    let mut ball = ball.single_mut().unwrap();
    let mut marker = marker.single_mut().unwrap();

    if let Ok(mut text) = text.single_mut() {
        let mut message = Vec::new();
        writeln!(message, "Time: {:.3} s", seconds).unwrap();
        writeln!(
            message,
            "ship pos: {:.3e}, {:.3e}, {:.3e}",
            ship.pos.x - earth.pos.x,
            ship.pos.y - earth.pos.y,
            ship.pos.z - earth.pos.z
        )
        .unwrap();

        // Calculate the earth surface relative plane.
        let up_w = (ship.pos - earth.pos).normalize();
        let v_rel = ship.vel - earth.vel;
        // let v_tan = v_rel - v_rel.dot(&up_w) * up_w;
        // let v_tan = v_rel - v_rel.dot(&up_w) * up_w;
        let v_tan = v_rel - up_w * v_rel.dot(&up_w);
        if v_tan.norm_squared() < 1e-12 {
            // Don't bother with this until we get the moving version ok.
            todo!("Handle radial velocity case");
        }

        writeln!(
            message,
            "Velocity: {:.3} km/s, surface: {:.3} km/s",
            v_rel,
            v_tan.norm()
        )
        .unwrap();

        // Calculate our view frame.
        let view_y_f = v_tan.normalize();
        let view_z_f = up_w;
        let view_x_f = view_y_f.cross(&view_z_f).normalize();
        let view_rr = na::Matrix3::from_columns(&[view_x_f, view_y_f, view_z_f]);
        let nav_to_world =
            na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix(&view_rr));

        let body_to_world = ship_attitude.q_bw;

        let q_ball = body_to_world * nav_to_world.conjugate();
        ball.rotation = sim_quat_to_bevy(&q_ball);

        let distance = (ship.pos - earth.pos).norm();
        let altitude = distance - earth_size.radii[2];

        writeln!(message, "Ship altitude: {:.3} km", altitude).unwrap();
        //  writeln!(message, "Up: {:?}", up).unwrap();
        writeln!(message, " RCS: {:?}", rcs).unwrap();

        // Temp
        let q_fw = nav_to_world;

        // let v_f = q_fw.conjugate() * ship.vel.normalize();
        let v_f = q_fw.inverse_transform_vector(&ship.vel.normalize());
        let v_ball = q_ball.conjugate().transform_vector(&v_f).normalize();
        // let v_ball = (q_ball * v_f).normalize();
        writeln!(message, "v_ball: {:.3}", v_ball).unwrap();

        let q_marker = na::UnitQuaternion::rotation_between(&na::Vector3::z(), &v_ball)
            .unwrap_or(na::UnitQuaternion::identity());
        marker.rotation = sim_quat_to_bevy(&(q_ball * q_marker));
        // marker.rotation = sim_quat_to_bevy(&(q_marker * q_ball));
        // marker.rotation = sim_quat_to_bevy(&(q_marker * q_fw.conjugate()));
        // marker.rotation = sim_quat_to_bevy(&q_ball);
        // marker.rotation = sim_quat_to_bevy(&q_marker);
        // marker.rotation = sim_quat_to_bevy(&(q_ball * q_marker * q_ball.conjugate()));

        /*
        let want_world = q_ball * v_ball;
        let got_world = (q_ball * q_marker) * na::Vector3::z();

        let err = (got_world - want_world).norm();
        writeln!(message, "err: {:.3e}", err).unwrap();
        */

        **text = String::from_utf8(message).unwrap();
    }
}

#[allow(dead_code)]
fn sim_to_bevy(v: &na::Vector3<f64>) -> Vec3 {
    Vec3::new(v.x as f32, v.z as f32, -v.y as f32)
}

pub fn sim_quat_to_bevy(q: &na::UnitQuaternion<f64>) -> Quat {
    let r =
        na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), -std::f64::consts::FRAC_PI_2);
    let q = r.conjugate() * q * r;
    Quat::from_array([q.i as f32, q.j as f32, q.k as f32, q.w as f32])
}

/// Return `q_fw` (Frame -> World) with:
///
/// `z_f`` aligned to `z_w`, X_f the projection of `ref_w` into the tangent plane,
/// and Y_f = Z_f x X_f. Stable, right-handed.
#[allow(dead_code)]
fn frame_from_z_and_ref(
    z_w: na::Vector3<f64>,
    ref_w: &na::Vector3<f64>,
    // optional: previous X axis in world to keep continuity
    x_prev_w: Option<&na::Vector3<f64>>,
    // Optional: user flip switch.
    user_flip: bool,
) -> (na::UnitQuaternion<f64>, na::Vector3<f64>) {
    let z_unit = na::Unit::new_normalize(z_w);
    // Project the reference into the tangent plane
    let mut x = ref_w - z_unit.into_inner() * ref_w.dot(&z_unit);
    let near_zero = x.norm_squared() < 1e-12;

    if near_zero {
        x = any_orthonormal_vector(&z_unit);
    } else {
        x = x.normalize();
    }

    if let Some(xp) = x_prev_w {
        // Ensure continuity by choosing the direction of X to be as close as possible to previous.
        if x.dot(xp) < 0.0 {
            // Flip to stay continuous.
            x = -x;
        }
    }

    // Optional manual flip switch.
    if user_flip {
        x = -x;
    }

    let y = z_unit.cross(&x).normalize();
    let r = na::Matrix3::from_columns(&[x, y, z_unit.into_inner()]);
    (
        na::UnitQuaternion::from_rotation_matrix(&na::Rotation3::from_matrix_unchecked(r)),
        x,
    )
}

#[allow(dead_code)]
fn any_orthonormal_vector(v: &na::Vector3<f64>) -> na::Vector3<f64> {
    let mut axis = na::Vector3::x_axis();
    if v.x.abs() > v.y.abs() {
        if v.y.abs() > v.z.abs() {
            axis = na::Vector3::z_axis();
        } else {
            axis = na::Vector3::y_axis();
        }
    } else if v.x.abs() > v.z.abs() {
        axis = na::Vector3::z_axis();
    }
    v.cross(&axis.into_inner()).normalize()
}
