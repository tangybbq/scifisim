/// Spice wrappers.
use std::{
    ffi::{CStr, CString},
    sync::{Arc, LazyLock, Mutex},
};

/// The single global SPICE instance.
static SPICE: LazyLock<Spice> = LazyLock::new(|| Spice::new());

/// An error from SPICE.
pub struct SpiceError(String);

impl std::fmt::Display for SpiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SPICE error: {}", self.0)
    }
}

impl std::fmt::Debug for SpiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SpiceError").field(&self.0).finish()
    }
}

impl std::error::Error for SpiceError {}

type Result<T> = std::result::Result<T, SpiceError>;

pub fn get_instance() -> Spice {
    SPICE.clone()
}

/// A wrapped SPICE interface.  Internally cares for its own locking.
#[derive(Clone)]
pub struct Spice(Arc<Mutex<()>>);

impl Spice {
    pub fn new() -> Self {
        // Load the SPICE kernels for use.
        spice::furnsh("assets/spice/de440s.bsp");
        spice::furnsh("assets/spice/jup365.bsp");
        spice::furnsh("assets/spice/mar099.bsp");
        spice::furnsh("assets/spice/nep095.bsp");
        spice::furnsh("assets/spice/plu060.bsp");
        spice::furnsh("assets/spice/sat441.bsp");
        spice::furnsh("assets/spice/ura184_part-1.bsp");
        spice::furnsh("assets/spice/ura184_part-2.bsp");
        spice::furnsh("assets/spice/ura184_part-3.bsp");
        spice::furnsh("assets/spice/naif0012.tls");
        spice::furnsh("assets/spice/pck00011.tpc");
        spice::furnsh("assets/spice/gm_de440.tpc");

        // Set the error handling to return errors, and to not print them out.
        unsafe {
            spice::c::erract_c(c"SET".as_ptr() as *mut _, 0, c"RETURN".as_ptr() as *mut _);
            spice::c::errprt_c(c"SET".as_ptr() as *mut _, 0, c"NONE".as_ptr() as *mut _);
        }

        Spice(Arc::new(Mutex::new(())))
    }

    /// Check if the last call returned an error, if so, clear it, and return the error.  Otherwise return Ok(()).
    ///
    /// This assumes the lock is already held.
    pub fn chkerr(&self) -> Result<()> {
        let is_err = unsafe { spice::c::failed_c() };
        if is_err != 0 {
            let mut msg = [0u8; 26];
            unsafe {
                spice::c::getmsg_c(c"SHORT".as_ptr() as *mut _, 26, msg.as_mut_ptr() as *mut _);
                spice::c::reset_c();
            }
            let msg = CStr::from_bytes_until_nul(&msg)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            Err(SpiceError(msg))
        } else {
            Ok(())
        }
    }

    pub fn str2et(&self, time: &str) -> Result<f64> {
        let _lock = self.0.lock().unwrap();
        let result = spice::str2et(time);
        self.chkerr()?;
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn spkezr(
        &self,
        target: &str,
        et: f64,
        ref_frame: &str,
        abcorr: &str,
        observer: &str,
    ) -> Result<([f64; 6], f64)> {
        let _lock = self.0.lock().unwrap();
        let result = spice::spkezr(target, et, ref_frame, abcorr, observer);
        self.chkerr()?;
        Ok(result)
    }

    pub fn bodvrd(&self, body: &str, item: &str, maxn: usize) -> Result<Vec<f64>> {
        let _lock = self.0.lock().unwrap();
        let result = spice::bodvrd(body, item, maxn);
        self.chkerr()?;
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn bodfnd(&self, body: i32, item: &str) -> bool {
        let _lock = self.0.lock().unwrap();
        spice::bodfnd(body, item)
    }

    #[allow(dead_code)]
    pub fn pxform(&self, from: &str, to: &str, et: f64) -> Result<[[f64; 3]; 3]> {
        let _lock = self.0.lock().unwrap();
        let result = spice::pxform(from, to, et);
        self.chkerr()?;
        Ok(result)
    }

    pub fn sxform(&self, from: &str, to: &str, et: f64) -> Result<[[f64; 6]; 6]> {
        let _lock = self.0.lock().unwrap();
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
        self.chkerr()?;
        Ok(result)
    }

    pub fn xf2rav(&self, xform: &[[f64; 6]; 6]) -> Result<([[f64; 3]; 3], [f64; 3])> {
        let _lock = self.0.lock().unwrap();
        let mut rot = [[0.0; 3]; 3];
        let mut av = [0.0; 3];
        unsafe {
            spice::c::xf2rav_c(xform.as_ptr() as *mut _, rot.as_mut_ptr(), av.as_mut_ptr());
        }
        self.chkerr()?;
        Ok((rot, av))
    }

    pub fn gnpool(&self, name: &str, start: usize, room: usize) -> Result<Vec<String>> {
        let _lock = self.0.lock().unwrap();
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
        self.chkerr()?;
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn gdpool(&self, name: &str, start: usize, room: usize) -> Result<Vec<f64>> {
        let _lock = self.0.lock().unwrap();
        let result = spice::gdpool(name, start, room);
        self.chkerr()?;
        Ok(result)
    }

    pub fn bodc2n(&self, code: i32) -> Option<String> {
        let _lock = self.0.lock().unwrap();
        let (name, found) = spice::bodc2n(code);
        if found { Some(name) } else { None }
    }
}
