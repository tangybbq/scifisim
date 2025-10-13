//! Simulation of the user's craft.
//!
//! Most of the information behind the physics of the ship is in `solar.rs`,
//! including orbital movements. This module manages ship-specific aspects.

use bevy::prelude::*;
use na::{Unit, Vector3};
use serde::{Deserialize, Serialize};

use crate::solar::{
    AttitudeControl, AttitudeState, EarthMarker, MassiveBody, OrbitalBody, setup_solar,
};

#[derive(Component)]
pub struct PlayerShip;

/// A description of an initial orbit for the ship.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ShipOrbit {
    /// The normal vector of the orbital plane.
    pub plane_normal: Unit<Vector3<f64>>,
    /// The direction of periapsis within the orbital plane.
    pub periapsis_direction: Unit<Vector3<f64>>,
    /// The distance at periapsis, in km from the center of the planet.
    pub periapsis: f64,
    /// The distance at apoapsis, in km from the center of the planet.
    pub apoapsis: f64,
    /// The current true anomaly, in radians, with 0 being periapsis.
    pub true_anomaly: f64,
}

impl ShipOrbit {
    pub fn new(
        plane_normal: Unit<Vector3<f64>>,
        periapsis_direction: Unit<Vector3<f64>>,
        periapsis: f64,
        apoapsis: f64,
        true_anomaly: f64,
    ) -> Self {
        ShipOrbit {
            plane_normal,
            periapsis_direction,
            periapsis,
            apoapsis,
            true_anomaly,
        }
    }

    /// Construct a circular LEO orbit on the earth.
    pub fn new_leo() -> Self {
        ShipOrbit::new(
            Unit::new_normalize(Vector3::z()),
            Unit::new_normalize(Vector3::x()),
            6578.0, // 200 km altitude
            6678.0, // 201 km altitude
            0.0,
        )
    }
}

/// Plugin to setup a ship in orbit.
#[derive(Default)]
pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShipOrbit::new_leo());
        app.add_systems(Startup, setup_ship.after(setup_solar));
        app.add_systems(Update, rcs_keys_to_alpha);
    }
}

fn setup_ship(
    orbit: Res<ShipOrbit>,
    earth: Query<(&MassiveBody, &OrbitalBody), With<EarthMarker>>,
    mut commands: Commands,
) {
    // Ensure that the periapsis direction is perpendicular to the plane normal.
    assert!(
        orbit.plane_normal.dot(&orbit.periapsis_direction) < 1e-6,
        "Periapsis direction must be perpendicular to plane normal"
    );

    // Calculate the initial position and velocity.
    let a = (orbit.periapsis + orbit.apoapsis) / 2.0;
    let e = (orbit.apoapsis - orbit.periapsis) / (orbit.apoapsis + orbit.periapsis);
    let p = a * (1.0 - e * e);

    let p_hat = orbit.periapsis_direction.into_inner();
    let q_hat = orbit.plane_normal.cross(&p_hat);

    let (mb, ob) = earth.single().unwrap();
    let mu = mb.gm;
    let nu = orbit.true_anomaly;
    let r_mag = p / (1.0 + e * nu.cos());

    let r_rel = (p_hat * nu.cos() + q_hat * nu.sin()) * r_mag;
    let v_rel = (-p_hat * nu.sin() + q_hat * (e + nu.cos())) * (mu / p).sqrt();

    let r_world = ob.pos + r_rel;
    let v_world = ob.vel + v_rel;

    // Spawn the ship.
    commands.spawn((
        Name::new("PlayerShip"),
        OrbitalBody {
            pos: r_world,
            vel: v_world,
        },
        AttitudeState {
            // q_bw: na::UnitQuaternion::from_axis_angle(
            //     &Vector3::y_axis(),
            //     std::f64::consts::FRAC_PI_2,
            // ),
            q_bw: na::UnitQuaternion::identity(),
            // q_bw: na::UnitQuaternion::from_axis_angle(
            //     &Vector3::y_axis(),
            //     std::f64::consts::FRAC_PI_2,
            // ),
            // omega_b: Vector3::new(1.0, 2.0, 3.0).normalize() * 0.5,
            omega_b: Vector3::zeros(),
        },
        AttitudeControl {
            alpha_b: Vector3::zeros(),
        },
        PlayerShip,
    ));

    /*
    println!("Spawned ship at pos {:?} vel {:?}", r_rel, v_rel);
    println!(
        "  q_bw: {}",
        na::UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f64::consts::FRAC_PI_2)
    );
    println!("  omega_b: {}", Vector3::<f64>::zeros());
    */
}

const ACCEL_X: f64 = 0.25;
const ACCEL_Y: f64 = 0.25;
const ACCEL_Z: f64 = 0.25;

#[derive(Resource, Component, Debug, Default, Clone, Copy)]
pub enum RcsMode {
    #[default]
    Manual,
    Hold,
}

fn rcs_keys_to_alpha(
    kb: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<RcsMode>,
    mut query: Query<(&mut AttitudeControl, &AttitudeState), With<PlayerShip>>,
) {
    // TODO: This simple mode switch isn't what we really will want, but I'll
    // have to come up with what makes sense.  Basically, it shouldn't just go
    // between the modes as you wouldn't want it to start moving until you
    // confirm the mode. But this works for two modes.
    if kb.just_pressed(KeyCode::KeyR) {
        *mode = match *mode {
            RcsMode::Manual => RcsMode::Hold,
            RcsMode::Hold => RcsMode::Manual,
        };
    }

    match *mode {
        RcsMode::Hold => {
            let mut all_zero = true;

            for (mut control, state) in query.iter_mut() {
                // We want to stop the current rotation, so apply the RCS in a direction opposite to the desired state.
                // The min is to try and make this actually settle in, but it makes an assumption about the physics step.
                control.alpha_b.x =
                    -state.omega_b.x.signum() * (state.omega_b.x.abs() * 64.0).min(ACCEL_X);
                control.alpha_b.y =
                    -state.omega_b.y.signum() * (state.omega_b.y.abs() * 64.0).min(ACCEL_Y);
                control.alpha_b.z =
                    -state.omega_b.z.signum() * (state.omega_b.z.abs() * 64.0).min(ACCEL_Z);

                if control.alpha_b.norm() > 1e-6 {
                    all_zero = false;
                }
                if all_zero {
                    *mode = RcsMode::Manual;
                }
            }
        }
        RcsMode::Manual => {
            for (mut control, _state) in query.iter_mut() {
                let mut alpha_b = Vector3::zeros();
                if kb.pressed(KeyCode::KeyW) {
                    alpha_b.x += ACCEL_X;
                }
                if kb.pressed(KeyCode::KeyS) {
                    alpha_b.x -= ACCEL_X;
                }
                if kb.pressed(KeyCode::KeyA) {
                    alpha_b.y += ACCEL_Y;
                }
                if kb.pressed(KeyCode::KeyD) {
                    alpha_b.y -= ACCEL_Y;
                }
                if kb.pressed(KeyCode::KeyQ) {
                    alpha_b.z += ACCEL_Z;
                }
                if kb.pressed(KeyCode::KeyE) {
                    alpha_b.z -= ACCEL_Z;
                }
                control.alpha_b = alpha_b;
            }
        }
    }
}
