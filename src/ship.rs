//! Simulation of the user's craft.
//!
//! Most of the information behind the physics of the ship is in `solar.rs`,
//! including orbital movements. This module manages ship-specific aspects.

use bevy::prelude::*;
use na::{Unit, Vector3};
use serde::{Deserialize, Serialize};

use crate::solar::{EarthMarker, MassiveBody, OrbitalBody, setup_solar};

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
            6578.0, // 200 km altitude
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
        PlayerShip,
    ));
}
