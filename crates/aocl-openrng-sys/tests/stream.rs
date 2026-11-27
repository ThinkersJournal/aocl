//! Known answers and determinism (docs/cleanroom/openrng/MEASUREMENT.md; compiled only with OPENRNG_LIB_DIR, cfg `openrng_linked`).
//! Seed 0's first four 32-bit words are the PUBLISHED Random123 Philox4x32-10 vector for counter 0 / key 0 (github.com/DEShawResearch/random123,
//! tests/kat_vectors at d8a0c25e: `philox4x32 10 00000000 00000000 00000000 00000000 00000000 00000000  6627e8d5 e169c58d bc57ac4c 9b00dbd8`);
//! that seed 0 means counter 0 / key 0 is an observed equality, not a documented mapping. Seed 42 has NO external reference: its words are the
//! ones recorded on both Windows builds (reproducibility across the two builds only).
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

fn bits32(seed: openrng_uint_t, n: usize) -> Vec<u32> {
    let mut s: VSLStreamStatePtr = core::ptr::null_mut();
    let mut w = vec![0u32; n];
    unsafe {
        assert_eq!(
            vslNewStream(&mut s, VSL_BRNG_PHILOX4X32X10 as openrng_int_t, seed),
            VSL_ERROR_OK
        );
        assert_eq!(
            viRngUniformBits32(0 as openrng_int_t, s, n as openrng_int_t, w.as_mut_ptr()),
            VSL_ERROR_OK
        );
        assert_eq!(vslDeleteStream(&mut s), VSL_ERROR_OK);
    }
    w
}

#[test]
fn philox_seed0_matches_the_published_random123_vector() {
    assert_eq!(
        &bits32(0, 4)[..],
        &[0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8][..]
    );
    // control: a different seed must not give that vector
    assert_ne!(
        &bits32(1, 4)[..],
        &[0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8][..]
    );
}

#[test]
fn philox_seed42_matches_the_recorded_words() {
    // recorded on both Windows builds (no external reference)
    assert_eq!(
        bits32(42, 8),
        vec![
            0x9ceaf053, 0x77f5493b, 0x12bf50ad, 0x5742b3d7, 0xfcdb2127, 0x53ba6cfd, 0x838f5a6e,
            0x744e06fb
        ]
    );
}
