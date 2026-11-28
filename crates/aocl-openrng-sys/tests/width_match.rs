//! Guards the one silent failure the width measurement found (docs/cleanroom/openrng/MEASUREMENT.md): the `ilp64` feature must match
//! the library build. With the feature ON this test passes a 64-bit `n` whose high half is nonzero (0xFFFFFFFF_00000004: negative as a 64-bit
//! value) to the library through the crate's own types: an ILP64 library refuses it (status -2), an LP64 library silently truncates it to 4
//! and ACCEPTS it, so a mismatched build fails here. With the feature OFF the crate's `openrng_int_t` is 32-bit and the test checks that
//! (a 64-bit value cannot even be expressed) and that a legal n works. Compiled only with OPENRNG_LIB_DIR (cfg `openrng_linked`).
#![cfg(openrng_linked)]
use aocl_openrng_sys::*;

fn uniform_status(n: openrng_int_t) -> core::ffi::c_int {
    let mut s: VSLStreamStatePtr = core::ptr::null_mut();
    let mut r = [0f64; 8];
    unsafe {
        assert_eq!(
            vslNewStream(&mut s, VSL_BRNG_PHILOX4X32X10 as openrng_int_t, 42),
            VSL_ERROR_OK
        );
        let st = vdRngUniform(
            VSL_RNG_METHOD_UNIFORM_STD as openrng_int_t,
            s,
            n,
            r.as_mut_ptr(),
            0.0,
            1.0,
        );
        assert_eq!(vslDeleteStream(&mut s), VSL_ERROR_OK);
        st
    }
}

#[cfg(feature = "ilp64")]
#[test]
fn ilp64_feature_matches_an_ilp64_library() {
    assert_eq!(core::mem::size_of::<openrng_int_t>(), 8);
    assert_eq!(uniform_status(4), VSL_ERROR_OK, "a legal n must work");
    let wide: openrng_int_t = ((-1i64) << 32) | 4;
    assert_eq!(
        uniform_status(wide),
        VSL_ERROR_BADARGS,
        "the `ilp64` feature is on but the library accepted a 64-bit n with a nonzero high half: it is the LP64 build (width mismatch)"
    );
}

#[cfg(not(feature = "ilp64"))]
#[test]
fn lp64_feature_off_is_32_bit_and_a_legal_n_works() {
    assert_eq!(core::mem::size_of::<openrng_int_t>(), 4);
    assert_eq!(uniform_status(4), VSL_ERROR_OK);
    assert_eq!(
        uniform_status(-4),
        VSL_ERROR_BADARGS,
        "control: a negative n is refused by both builds"
    );
}
