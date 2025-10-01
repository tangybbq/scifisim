//! Solar system modeling.

// The rust-spice crate has a locking mechanism to ensure single threaded
// access. However, it only implements a handeful of the SPICE functions, and
// when enabled, makes the raw versions of the functions inaccessible. In order
// for spice to actually be useful, we'll need to use our own lock, and just
// make sure we only use the API while holding the lock.

use nalgebra::{Matrix3, Vector3};

mod spice;

pub fn init_spice() {
    let sl = spice::SPICE.lock().unwrap();

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
