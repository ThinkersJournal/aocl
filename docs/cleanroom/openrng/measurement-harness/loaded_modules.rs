//! Reports which files the process actually loaded for the three DLLs of interest (GetModuleFileNameA), so the driver script can
//! hash THOSE files and compare them with C:/openrng-build/build-sha256.txt. It first touches the library (vslNewStream) so the
//! delay-free import is resolved either way. Prints `MODULE name = path` lines; asserts nothing.
use core::ffi::{c_char, c_void};

extern "system" {
    fn GetModuleHandleA(name: *const c_char) -> *mut c_void;
    fn GetModuleFileNameA(module: *mut c_void, buf: *mut c_char, size: u32) -> u32;
}

fn path_of(name: &str) -> String {
    let cname = std::ffi::CString::new(name).unwrap();
    unsafe {
        let h = GetModuleHandleA(cname.as_ptr());
        if h.is_null() {
            return "NOT LOADED".into();
        }
        let mut buf = vec![0 as c_char; 1024];
        let n = GetModuleFileNameA(h, buf.as_mut_ptr(), 1024);
        let bytes: Vec<u8> = buf[..n as usize].iter().map(|c| *c as u8).collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[test]
fn record_loaded_modules() {
    use aocl_openrng_sys::*;
    let mut s: VSLStreamStatePtr = core::ptr::null_mut();
    unsafe {
        assert_eq!(vslNewStream(&mut s, VSL_BRNG_PHILOX4X32X10 as openrng_int_t, 1), VSL_ERROR_OK);
        assert_eq!(vslDeleteStream(&mut s), VSL_ERROR_OK);
    }
    for m in ["openrng.dll", "libalm.dll"] {
        println!("MODULE {m} = {}", path_of(m));
    }
}
