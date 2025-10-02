//! Solar system modeling.

// The rust-spice crate has a locking mechanism to ensure single threaded
// access. However, it only implements a handeful of the SPICE functions, and
// when enabled, makes the raw versions of the functions inaccessible. In order
// for spice to actually be useful, we'll need to use our own lock, and just
// make sure we only use the API while holding the lock.

use bevy::ecs::component::Component;
use na::Matrix3x1;
use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

mod spice;

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct Body {
    pub id: i32,
    pub name: String,
    pub gm: f64, // Gravitational constant * mass, km^3/s^2
    pub pos: Vector3<f64>,
    pub vel: Vector3<f64>,
    pub radii: Vector3<f64>,
    pub north: Matrix3x1<f64>,
    pub omega: f64,
}

impl Body {
    pub fn new_from(id: i32, et: f64) -> Option<Self> {
        let sl = spice::get_instance();
        let name = sl.bodc2n(id)?;

        // Reject barycenters.
        if name.ends_with(" BARYCENTER") {
            return None;
        }

        let gm = sl.bodvrd(&name, "GM", 1).ok()?;
        // Reject "small" bodies.  This also avoids bodies that don't have a
        // radius.
        if gm[0] < 1.0 {
            return None;
        }
        let radii = sl.bodvrd(&name, "RADII", 3).ok()?;
        let radii = Vector3::new(radii[0], radii[1], radii[2]);

        let xform = sl.sxform(&format!("IAU_{}", name), "ECLIPJ2000", et).ok()?;
        let (rot, av) = sl.xf2rav(&xform).ok()?;
        let rot = Matrix3::from_row_slice(&[
            rot[0][0], rot[0][1], rot[0][2], // Row 0
            rot[1][0], rot[1][1], rot[1][2], // Row 1
            rot[2][0], rot[2][1], rot[2][2], // Row 2
        ]);
        let north = rot * Vector3::new(0.0, 0.0, 1.0);
        let av = Vector3::new(av[0], av[1], av[2]);
        let omega = av.norm();

        let (state, _) = sl.spkezr(&name, et, "ECLIPJ2000", "NONE", "SSB").ok()?;

        let pos = Vector3::new(state[0], state[1], state[2]);
        let vel = Vector3::new(state[3], state[4], state[5]);

        Some(Self {
            id,
            name,
            gm: gm[0],
            radii,
            pos,
            vel,
            north,
            omega,
        })
    }
}

pub fn init_spice() {
    let sl = spice::get_instance();
    // TODO: Better start date.
    let et = sl.str2et("2024-01-01T00:00:00").unwrap();
    let mut bodies = Vec::new();
    let mut start = 0;
    loop {
        let limit = 500;
        let names = sl.gnpool("BODY*_GM", start, limit).unwrap();
        println!("Names: count: {}", names.len());
        for name in &names {
            let code = name[4..name.len() - 3].parse::<i32>().unwrap();

            if let Some(body) = Body::new_from(code, et) {
                if body.gm > 1.0 && !body.name.ends_with(" BARYCENTER") {
                    bodies.push(body);
                }
                continue;
            }
        }
        if names.len() < limit {
            break;
        }

        start += names.len();
    }

    bodies.sort_by(|a, b| b.gm.partial_cmp(&a.gm).unwrap());
    for body in &bodies {
        println!("{:20} {:8}", body.name, body.id);
        println!("   gm: {}: radii: {:?}", body.gm, body.radii);
        println!("   omega: {} {:?}", body.omega, body.north);
        println!("   pos: {:?} km", body.pos);
        println!("   vel: {:?} km/s", body.vel);
    }

    println!("Interesting: {}", bodies.len());
}
