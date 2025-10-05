//! The basic UI.
//!
//! At this level, we display some information about the scene.  This sets up
//! its own 2d camera to overlay this information on any other camera.

use bevy::{camera::visibility::RenderLayers, color::palettes::css::GOLD, prelude::*};
use std::io::Write;

// use bevy::pbr::wireframe::Wireframe;

use crate::solar::{OrbitalBody, SizedBody};

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
        BallMarker,
    ));

    // Throw in a sphere to see if I can render it.
    let ball_mesh = Sphere {
        radius: 100.0,
        ..default()
    }
    .mesh()
    .uv(24, 24);

    let ball_tex = asset_server.load("tex/ball.png");

    let ball_material = materials.add(StandardMaterial {
        // base_color: GREEN.into(),
        // base_color: Color::linear_rgb(1.0, 0.4, 0.2),
        base_color_texture: Some(ball_tex),
        perceptual_roughness: 0.5,
        unlit: true,
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
        Name::new("Ball"),
    ));

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
    ship: Query<&OrbitalBody, With<crate::ship::PlayerShip>>,
    earth: Query<(&OrbitalBody, &SizedBody), With<crate::solar::EarthMarker>>,
    mut ball: Query<&mut Transform, With<BallMarker>>,
) {
    let seconds = time.elapsed_secs_f64();
    let ship = ship.single().unwrap();
    let (earth, earth_size) = earth.single().unwrap();
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

        // Point the ball to the same as the ship.
        ball.rotation = Quat::from_rotation_arc(Vec3::Y, sim_to_bevy(&up));
    }
}

fn sim_to_bevy(v: &na::Vector3<f64>) -> Vec3 {
    Vec3::new(v.x as f32, v.z as f32, -v.y as f32)
}
