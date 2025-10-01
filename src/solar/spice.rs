/// Spice wrappers.
use std::{
    ffi::CString,
    sync::{LazyLock, Mutex},
};

pub static SPICE: LazyLock<Mutex<Spice>> = LazyLock::new(|| Mutex::new(Spice::new()));

pub struct Spice;

impl Spice {
    pub fn new() -> Self {
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
            spice::c::sxform_c(
                from.as_ptr() as *mut _,
                to.as_ptr() as *mut _,
                et,
                &mut result as *mut _,
            );
        }
        result
    }

    pub fn xf2rav(&self, xform: &[[f64; 6]; 6]) -> ([[f64; 3]; 3], [f64; 3]) {
        let mut rot = [[0.0; 3]; 3];
        let mut av = [0.0; 3];
        unsafe {
            spice::c::xf2rav_c(xform.as_ptr() as *mut _, rot.as_mut_ptr(), av.as_mut_ptr());
        }
        (rot, av)
    }

    // pub fn gnpool(&self, n: i32, a: &[f64]) -> f64 {
    //     spice::gnpool(n, a)
    // }
}
