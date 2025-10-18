//! Demonstration of the intermediate axis theorem.
//!
//! The intermediate axis theorem (or tennis-racket theorem) states that
//! rotation about the axis of an object that is the intermediate (meaning one
//! of the other axes is larger and the other smaller) results in an object that
//! has unexpected flips in its orientation.
//!
//! This demo shows a pair of cylinders setup to be similar to the handle in
//! this video: https://www.youtube.com/watch?v=1x5UiwEEvpQ that clearly
//! demonstrates the flipping effect. If the rotation physics are implemented
//! correctly, this demo should show a similar flipping effect.

extern crate nalgebra as na;

use bevy::{
    color::palettes::css::{GREEN, RED, YELLOW},
    post_process::motion_blur::MotionBlur,
    prelude::*,
};
use sim_physics::AttitudeState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Bump up the Fixed Update interval so that we can spin this faster to better observe the effect.
        // .insert_resource(Time::<Fixed>::from_hz(500.0))
        .add_systems(Startup, setup)
        .add_systems(Update, update_bevy_rot)
        .add_systems(FixedUpdate, update_rotational_physics)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Transform::default(),
            AttitudeState::new_with_omega_b(
                na::UnitQuaternion::identity(),
                na::Vector3::new(3000.0 / 373.0, 0.0, 3.0 / 78.0),
                na::Vector3::new(373.0, 415.0, 78.0),
                na::Vector3::zeros(),
            ),
        ))
        .with_child((
            Mesh3d(meshes.add(Cylinder {
                radius: 0.25,
                half_height: 1.0,
                ..default()
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GREEN.into(),
                ..default()
            })),
        ))
        .with_child((
            Mesh3d(meshes.add(Cylinder {
                radius: 0.15,
                half_height: 0.35,
                ..default()
            })),
            Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.0,
                0.0,
                std::f32::consts::FRAC_PI_2,
            ))
            .with_translation(Vec3::new(0.35, 0.0, 0.0)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: YELLOW.into(),
                ..default()
            })),
        ))
        .with_child((
            Mesh3d(meshes.add(Cylinder {
                radius: 0.15,
                half_height: 0.35,
                ..default()
            })),
            Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0.0,
                0.0,
                std::f32::consts::FRAC_PI_2,
            ))
            .with_translation(Vec3::new(-0.35, 0.0, 0.0)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: RED.into(),
                ..default()
            })),
        ));

    commands.spawn((
        PointLight {
            intensity: 5_500_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d { ..default() },
        MotionBlur {
            shutter_angle: 1.0,
            samples: 8,
        },
        Transform::from_xyz(-1.5, 2.5, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Update any object with an AttitudeState to update the Bevy Transform. Should be called in Update.
fn update_bevy_rot(mut query: Query<(&mut Transform, &AttitudeState)>) {
    for (mut transform, state) in query.iter_mut() {
        transform.rotation = sim_quat_to_bevy(&state.q_bw);
    }
}

/// Simulate the rotational physics.
///
/// For now, no torque is implemented.
fn update_rotational_physics(mut query: Query<&mut AttitudeState>, time: Res<Time>) {
    let dt = time.delta_secs_f64();

    for mut attitude in query.iter_mut() {
        // No torque for now.
        let torque_w_now = na::Vector3::zeros();
        attitude.step_rot_fixed_tau_b(dt, torque_w_now);
    }
}

/// Convert a nalgebra quaternion (f64) to a bevy quaternion (f32).  This
/// includes the basis change between the Z-up sim and the Y-up bevy.
pub fn sim_quat_to_bevy(q: &na::UnitQuaternion<f64>) -> Quat {
    let r =
        na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), -std::f64::consts::FRAC_PI_2);
    let q = r.conjugate() * q * r;
    Quat::from_array([q.i as f32, q.j as f32, q.k as f32, q.w as f32])
}
