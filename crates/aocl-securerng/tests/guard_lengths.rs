//! Guard-page harness for the aocl-securerng array functions: an exact-length output at the end of a committed page before a PAGE_NOACCESS page
//! (the library writes `n` ELEMENTS for the u32/u64 arrays only if `n` counts elements: a fault here would mean it counts something else).
//! One process per test; a fault = HOLE. Hand-written, Windows only, Linux unmeasured. A CPU without RDRAND/RDSEED returns an error (accepted).
#![cfg(windows)]
use aocl_securerng::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;
fn at_guard<T: Copy + Default, R>(n: usize, f: impl FnOnce(&mut [T]) -> R) -> R {
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        let mut old = 0u32;
        assert_ne!(VirtualProtect((base as *mut u8).add(PAGE) as *mut c_void, PAGE, 0x01, &mut old), 0);
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for i in 0..n {
            start.add(i).write(T::default());
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}
macro_rules! case {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            let r = $body;
            println!("RESULT {} accepted ({:?})", stringify!($name), r.is_ok());
        }
    };
}
case!(rdrand_bytes_exact, at_guard::<u8, _>(37, |o| rdrand_bytes(o, 10)));
case!(rdseed_bytes_exact, at_guard::<u8, _>(37, |o| rdseed_bytes(o, 10)));
case!(rdrand_u32_array_exact, at_guard::<u32, _>(37, |o| rdrand_u32_array(o, 10)));
case!(rdrand_u64_array_exact, at_guard::<u64, _>(37, |o| rdrand_u64_array(o, 10)));
case!(rdrand_u64_array_odd, at_guard::<u64, _>(3, |o| rdrand_u64_array(o, 10)));
