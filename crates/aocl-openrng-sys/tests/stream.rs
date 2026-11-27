//! NEEDS THE BUILD (not run yet; compiled only with OPENRNG_LIB_DIR, cfg `openrng_linked`).
//! Checks the width (`openrng_int_t` against the library: a wrong width makes the call misbehave) and determinism. The known
//! answer (first values of PHILOX4X32X10, seed 42) is NOT written here: it must be MEASURED on the built library first.
#![cfg(openrng_linked)]
use aocl_openrng_sys::*;

fn first_values(seed: openrng_uint_t) -> [f64; 4] {
    let mut s: VSLStreamStatePtr = core::ptr::null_mut();
    let mut r = [0f64; 4];
    unsafe {
        assert_eq!(
            vslNewStream(&mut s, VSL_BRNG_PHILOX4X32X10 as openrng_int_t, seed),
            VSL_ERROR_OK
        );
        assert_eq!(
            vdRngUniform(
                VSL_RNG_METHOD_UNIFORM_STD as openrng_int_t,
                s,
                4,
                r.as_mut_ptr(),
                0.0,
                1.0
            ),
            VSL_ERROR_OK
        );
        assert_eq!(vslDeleteStream(&mut s), VSL_ERROR_OK);
    }
    r
}

#[test]
fn deterministic_and_in_range() {
    let a = first_values(42);
    let b = first_values(42);
    let c = first_values(43);
    assert_eq!(a, b);
    assert_ne!(a, c, "control: another seed must differ");
    assert!(a.iter().all(|x| (0.0..1.0).contains(x)));
}

#[test]
fn width_matches_feature() {
    assert_eq!(
        core::mem::size_of::<openrng_int_t>(),
        if cfg!(feature = "ilp64") { 8 } else { 4 }
    );
}
