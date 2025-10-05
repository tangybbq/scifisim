//! The basic UI.
//!
//! At this level, we display some information about the scene.  This sets up
//! its own 2d camera to overlay this information on any other camera.

use bevy::{camera::visibility::RenderLayers, color::palettes::css::GOLD, prelude::*};
use std::io::Write;

// use bevy::pbr::wireframe::Wireframe;

use crate::solar::{AttitudeState, OrbitalBody, SizedBody};

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
            ..default()
        },
        BALL_LAYER,
        Name::new("Ball Camera"),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        perceptual_roughness: 0.5,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let ball = meshes.add(ball_mesh);
    commands.spawn((
        Mesh3d(ball),
        BALL_LAYER,
        MeshMaterial3d(ball_material),
        Transform::from_xyz(0.0, 0.0, -100.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        // Wireframe,
        BallMarker,
        Name::new("Ball"),
    ));

    let vignetter_image = asset_server.load("tex/vignette_512.png");
    commands
        .spawn(Node {
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
        })
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
        ));,

    // commands.spawn((
    //     DirectionalLight {
    //         shadows_enabled: true,
    //         illuminance: 10_000.0,
    //         ..default()
    //     },
    //     Transform::default().looking_to(Vec3::new(0.0, 0.0, 1.0), Vec3::X),
    //     BALL_LAYER,
    //     Name::new("Ball Light"),
    // ));

    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 300.0,
    //     ..default()
    // });
}

fn update_ui(
    mut text: Query<&mut Text, With<InfoText>>,
    time: Res<Time<Virtual>>,
    ship: Query<(&OrbitalBody, &AttitudeState), With<crate::ship::PlayerShip>>,
    earth: Query<(&OrbitalBody, &SizedBody, &AttitudeState), With<crate::solar::EarthMarker>>,
    mut ball: Query<&mut Transform, With<BallMarker>>,
) {
    let seconds = time.elapsed_secs_f64();
    let (ship, ship_attitude) = ship.single().unwrap();
    let (earth, earth_size, earth_attitude) = earth.single().unwrap();
    let mut ball = ball.single_mut().unwrap();

    if let Ok(mut text) = text.single_mut() {
        let mut message = Vec::new();
        writeln!(message, "Time: {:.3} s", seconds).unwrap();
        writeln!(
            message,
            "Ship pos: {:.3e}, {:.3e}, {:.3e}",
            ship.pos.x, ship.pos.y, ship.pos.z
        )
        .unwrap();

        let rel_pos = ship.pos - earth.pos;
        let distance = rel_pos.norm();
        let up = rel_pos.normalize();
        let altitude = distance - earth_size.radii[2];

        writeln!(message, "Ship altitude: {:.3e} km", altitude).unwrap();
        writeln!(message, "Up: {:.3}", up).unwrap();

        **text = String::from_utf8(message).unwrap();

        let q_ball = ship_attitude.q_bw.conjugate() * earth_attitude.q_bw;
        // Point the ball to the same as the ship.
        // println!("q_ball: {}", sim_quat_to_bevy(&q_ball));
        ball.rotation = sim_quat_to_bevy(&q_ball);
    }
}

// fn sim_to_bevy(v: &na::Vector3<f64>) -> Vec3 {
//     Vec3::new(v.x as f32, v.z as f32, -v.y as f32)
// }

fn sim_quat_to_bevy(q: &na::UnitQuaternion<f64>) -> Quat {
    let r =
        na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), -std::f64::consts::FRAC_PI_2);
    let q = r * q * r.conjugate();
    Quat::from_array([q.i as f32, q.j as f32, q.k as f32, q.w as f32])
}
