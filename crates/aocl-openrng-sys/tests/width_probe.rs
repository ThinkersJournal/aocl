//! MEASUREMENT PROBE for the width of `openrng_int_t` (docs/cleanroom/openrng/MEASUREMENT.md), black box, by behaviour.
//! It declares its OWN extern block with every `openrng_int_t` argument as 64-bit and passes values whose low 32 bits are the
//! same but whose upper 32 bits differ. On Win64 a 32-bit parameter reads only the low half of the argument register, a 64-bit
//! one reads all of it. A library with a 32-bit `openrng_int_t` therefore accepts `n = 0xFFFFFFFF_00000004` as n = 4 and writes 4
//! doubles; a library with a 64-bit one sees a negative n and must refuse. (A positive huge n is deliberately NOT passed: a 64-bit
//! library would try to fill it.) Compiled only with OPENRNG_LIB_DIR (cfg `openrng_linked`). Prints `PROBE ...` lines; asserts
//! nothing about the width (the table in MEASUREMENT.md is built from the printed record).
#![cfg(openrng_linked)]
use core::ffi::c_void;

extern "C" {
    fn vslNewStream(stream: *mut *mut c_void, brng: i64, seed: u64) -> i32;
    fn vslDeleteStream(stream: *mut *mut c_void) -> i32;
    fn vdRngUniform(method: i64, stream: *mut c_void, n: i64, r: *mut f64, a: f64, b: f64) -> i32;
}

fn status_for(n: i64) -> (i32, usize) {
    let mut s: *mut c_void = core::ptr::null_mut();
    let mut r = [-1.0f64; 8];
    unsafe {
        let rc = vslNewStream(&mut s, aocl_openrng_sys::VSL_BRNG_PHILOX4X32X10 as i64, 42);
        assert_eq!(rc, 0, "vslNewStream");
        let st = vdRngUniform(0, s, n, r.as_mut_ptr(), 0.0, 1.0);
        let written = r.iter().filter(|x| **x != -1.0).count();
        vslDeleteStream(&mut s);
        (st, written)
    }
}

#[test]
fn record_width_probe() {
    println!(
        "PROBE crate openrng_int_t = {} bytes (the crate's own type for this build)",
        core::mem::size_of::<aocl_openrng_sys::openrng_int_t>()
    );
    for (label, n) in [
        ("n=4 (control: both widths accept)", 4i64),
        ("n=-4 (control: both widths refuse)", -4),
        (
            "n=0xFFFFFFFF_00000004 (low half 4, high half all ones)",
            ((-1i64) << 32) | 4,
        ),
    ] {
        let (st, w) = status_for(n);
        println!("PROBE {label:58} status={st:<6} doubles_written={w}");
    }
}
