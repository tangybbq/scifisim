//! Demonstration of the intermediate axis theorem.

extern crate nalgebra as na;

use bevy::{
    color::palettes::css::{GREEN, RED, YELLOW},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Bump up the Fixed Update interval so that we can spin this faster to better observe the effect.
        .insert_resource(Time::<Fixed>::from_hz(500.0))
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
            AttitudeState::new(
                na::UnitQuaternion::identity(),
                na::Vector3::new(3000.0, 0.0, 3.0),
                na::Vector3::new(373.0, 415.0, 78.0),
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
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        attitude.step(torque_w_now, dt);
    }
}

#[derive(Debug, Clone, Component)]
pub struct AttitudeState {
    /// Orientation: body -> world
    pub q_bw: na::UnitQuaternion<f64>,
    /// Angular momentum (world frame), half stepped for leapfrog
    pub lw_half: na::Vector3<f64>,
    /// Principal inertia in body frame.
    pub i_body: na::Vector3<f64>,
}

impl AttitudeState {
    pub fn new(
        q_bw: na::UnitQuaternion<f64>,
        lw: na::Vector3<f64>,
        i_body: na::Vector3<f64>,
    ) -> Self {
        Self {
            q_bw,
            lw_half: lw,
            i_body,
        }
    }

    // Leapfrog step of size dt with torque in world frame.
    pub fn step(&mut self, torque_w_now: na::Vector3<f64>, dt: f64) {
        // --- KICK (half step): Update momentum from t-dt/2 to t ---
        let torque_b_now = self.q_bw.inverse() * torque_w_now;

        // Get L_b at half-step
        let lb_half = self.q_bw.inverse() * self.lw_half;

        // Compute omega at half-step
        let inv_i = na::Vector3::new(
            1.0 / self.i_body.x,
            1.0 / self.i_body.y,
            1.0 / self.i_body.z,
        );
        let omega_b_half = na::Vector3::new(
            inv_i.x * lb_half.x,
            inv_i.y * lb_half.y,
            inv_i.z * lb_half.z,
        );

        // Euler equation: dL/dt = L × ω + τ
        let dl_dt_b = lb_half.cross(&omega_b_half) + torque_b_now;

        // Half-kick: advance L by dt/2
        let lb_full = lb_half + dl_dt_b * (dt * 0.5);

        // --- DRIFT: Update orientation from t to t+dt using L at full step ---
        let omega_b_full = na::Vector3::new(
            inv_i.x * lb_full.x,
            inv_i.y * lb_full.y,
            inv_i.z * lb_full.z,
        );

        let dq = exp_quat(&(dt * omega_b_full));
        self.q_bw = self.q_bw * dq;

        // --- KICK (half step): Update momentum from t to t+dt/2 ---
        let torque_b_new = self.q_bw.inverse() * torque_w_now;

        // Recompute derivative at new orientation
        let dl_dt_b_new = lb_full.cross(&omega_b_full) + torque_b_new;

        // Half-kick: advance L by another dt/2
        let lb_next_half = lb_full + dl_dt_b_new * (dt * 0.5);

        // Store back in world frame
        self.lw_half = self.q_bw * lb_next_half;
    }
}

/// Exponential map for a pure-vector quaternuion input v (axis*angle).
/// Returns a unit quaternion representing rotation by |v| radians around axis v/|v|.
fn exp_quat(v: &na::Vector3<f64>) -> na::UnitQuaternion<f64> {
    let theta = v.norm();
    if theta < 1e-10 {
        let q = na::Quaternion::from_parts(1.0, 0.5 * v);
        na::UnitQuaternion::from_quaternion(q)
    } else {
        let half = 0.5 * theta;
        let s = half.sin() / theta;
        let q = na::Quaternion::from_parts(half.cos(), s * v);
        na::UnitQuaternion::from_quaternion(q)
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
