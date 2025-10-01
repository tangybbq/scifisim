//! Solar system modeling.

// The rust-spice crate has a locking mechanism to ensure single threaded
// access. However, it only implements a handeful of the SPICE functions, and
// when enabled, makes the raw versions of the functions inaccessible. In order
// for spice to actually be useful, we'll need to use our own lock, and just
// make sure we only use the API while holding the lock.

use std::sync::MutexGuard;

use nalgebra::{Matrix3, Vector3};

mod spice;

pub struct Body {
    pub id: i32,
    pub name: String,
    pub gm: f64, // Gravitational constant * mass, km^3/s^2
    pub radii: Vector3<f64>,
}

impl Body {
    pub fn new_from(id: i32) -> Option<Self> {
        let sl = spice::SPICE.lock().unwrap();
        let (name, has_name) = sl.bodc2n(id);
        if !has_name {
            return None;
        }
        // Reject barycenters.
        if name.ends_with(" BARYCENTER") {
            return None;
        }
        let gm = sl.bodvrd(&name, "GM", 1);
        // Reject "small" bodies.  This also avoids bodies that don't have a
        // radius.
        if gm[0] < 1.0 {
            return None;
        }
        // println!("Query: {name}: gm: {}", gm[0]);
        if !sl.bodfnd(id, "RADII") {
            return None;
        }
        let radii = sl.bodvrd(&name, "RADII", 3);
        Some(Self {
            id,
            name,
            gm: gm[0],
            radii: Vector3::new(radii[0], radii[1], radii[2]),
        })
    }
}

pub fn init_spice() {
    if false {
        let sl = spice::SPICE.lock().unwrap();
        was_init_spice(&sl);
    }
    let mut bodies = Vec::new();
    let mut start = 0;
    loop {
        let limit = 500;
        let names = spice::SPICE
            .lock()
            .unwrap()
            .gnpool("BODY*_GM", start, limit);
        println!("Names: count: {}", names.len());
        for name in &names {
            let code = name[4..name.len() - 3].parse::<i32>().unwrap();

            if let Some(body) = Body::new_from(code) {
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
        println!(
            "  {:20} {:20}: gm: {}: radii: {:?}",
            body.name, body.id, body.gm, body.radii
        );
    }

    println!("Interesting: {}", bodies.len());
}

pub fn was_init_spice(sl: &MutexGuard<'_, spice::Spice>) {
    let et = sl.str2et("2024-01-01T00:00:00");
    let (state, _) = sl.spkezr("EARTH", et, "ECLIPJ2000", "NONE", "SSB");
    println!("pos: {state:?}");
    let info = sl.bodvrd("EARTH", "RADII", 3);
    println!("RADII: {info:?}");
    let info = sl.bodvrd("EARTH", "GM", 1);
    println!("gm: {info:?}");

    // Information about north and rotation.
    let xform = sl.sxform("IAU_EARTH", "ECLIPJ2000", et);
    let (rot, av) = sl.xf2rav(&xform);
    let rot = Matrix3::from_row_slice(&[
        rot[0][0], rot[0][1], rot[0][2], // Row 0
        rot[1][0], rot[1][1], rot[1][2], // Row 1
        rot[2][0], rot[2][1], rot[2][2], // Row 2
    ]);
    let north = rot * Vector3::new(0.0, 0.0, 1.0);
    println!("north: {north:?}");

    // println!("rot: {rot:?}");
    let av = Vector3::new(av[0], av[1], av[2]);
    println!("av: {:?}", av.normalize());
    println!("avnormal: {}", av.norm());

    // Determine where the north pole is pointing.
    let rot = sl.pxform("IAU_EARTH", "ECLIPJ2000", et);
    let rot = Matrix3::from_row_slice(&[
        rot[0][0], rot[0][1], rot[0][2], // Row 0
        rot[1][0], rot[1][1], rot[1][2], // Row 1
        rot[2][0], rot[2][1], rot[2][2], // Row 2
    ]);
    let north = rot * Vector3::new(0.0, 0.0, 1.0);
    println!("north2: {north:?}");

    let (position, light_time) = sl.spkezr("SUN", et, "ECLIPJ2000", "NONE", "SSB");
    println!("position: {position:?}, light_time: {light_time:?}");
}
