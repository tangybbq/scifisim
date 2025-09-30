//! Solar system modeling.

// The rust-spice crate has a locking mechanism to ensure single threaded
// access. However, it only implements a handeful of the SPICE functions, and
// when enabled, makes the raw versions of the functions inaccessible. In order
// for spice to actually be useful, we'll need to use our own lock, and just
// make sure we only use the API while holding the lock.

use std::{
    ffi::{CString, c_char},
    sync::{LazyLock, Mutex},
};

use nalgebra::{Matrix3, Vector3};

static SPICE: LazyLock<Mutex<Spice>> = LazyLock::new(|| Mutex::new(Spice::new()));

struct Spice;

impl Spice {
    fn new() -> Self {
        // Load the SPICE kernels for use.
        spice::furnsh("assets/spice/de440s.bsp");
        spice::furnsh("assets/spice/naif0012.tls");
        spice::furnsh("assets/spice/pck00011.tpc");
        spice::furnsh("assets/spice/gm_de440.tpc");

        Spice
    }

    pub fn str2et(&self, time: &str) -> f64 {
        spice::str2et(time)
    }

    pub fn spkezr(
        &self,
        target: &str,
        et: f64,
        ref_frame: &str,
        abcorr: &str,
        observer: &str,
    ) -> ([f64; 6], f64) {
        spice::spkezr(target, et, ref_frame, abcorr, observer)
    }

    pub fn bodvrd(&self, body: &str, item: &str, maxn: usize) -> Vec<f64> {
        spice::bodvrd(body, item, maxn)
    }

    pub fn pxform(&self, from: &str, to: &str, et: f64) -> [[f64; 3]; 3] {
        spice::pxform(from, to, et)
    }

    pub fn sxform(&self, from: &str, to: &str, et: f64) -> [[f64; 6]; 6] {
        let mut result = [[0.0; 6]; 6];
        let from = CString::new(from).unwrap();
        let to = CString::new(to).unwrap();
        unsafe {
            sxform_c(from.as_ptr(), to.as_ptr(), et, &mut result);
        }
        result
    }

    pub fn xf2rav(&self, xform: &[[f64; 6]; 6]) -> ([[f64; 3]; 3], [f64; 3]) {
        let mut rot = [[0.0; 3]; 3];
        let mut av = [0.0; 3];
        unsafe {
            xf2rav_c(xform, &mut rot, &mut av);
        }
        (rot, av)
    }
}

unsafe extern "C" {
    unsafe fn sxform_c(from: *const c_char, to: *const c_char, et: f64, xform: *mut [[f64; 6]; 6]);
    unsafe fn xf2rav_c(xform: *const [[f64; 6]; 6], rot: *mut [[f64; 3]; 3], av: *mut [f64; 3]);
}

pub fn init_spice() {
    let sl = SPICE.lock().unwrap();

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
