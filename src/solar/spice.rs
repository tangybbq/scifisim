/// Spice wrappers.
use std::{
    ffi::{CStr, CString},
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

    pub fn bodfnd(&self, body: i32, item: &str) -> bool {
        spice::bodfnd(body, item)
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

    pub fn gnpool(&self, name: &str, start: usize, room: usize) -> Vec<String> {
        let mut buf = vec![[0u8; 33]; room];
        let mut n = 0;
        let mut found = 0;

        unsafe {
            spice::c::gnpool_c(
                CString::new(name).unwrap().as_ptr() as *mut _,
                start as i32,
                room as i32,
                33,
                &mut n,
                buf.as_mut_ptr() as *mut _,
                &mut found,
            );
        }

        let mut result = Vec::new();
        for i in 0..n {
            let str = CStr::from_bytes_until_nul(&buf[i as usize]).unwrap();
            let str = str.to_str().unwrap().to_string();
            result.push(str);
        }
        result
    }

    #[allow(dead_code)]
    pub fn gdpool(&self, name: &str, start: usize, room: usize) -> Vec<f64> {
        spice::gdpool(name, start, room)
    }

    pub fn bodc2n(&self, code: i32) -> (String, bool) {
        spice::bodc2n(code)
    }
}
