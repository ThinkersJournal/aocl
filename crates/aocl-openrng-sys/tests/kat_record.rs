//! RAW-OUTPUT RECORD for the known-answer step (docs/cleanroom/openrng/MEASUREMENT.md). No reference value is asserted here:
//! a reference exists only if one can be cited. This prints the first words of PHILOX4X32X10 for seed 42 and seed 0 so the two
//! builds can be compared byte for byte, and so a published Philox vector can be compared by hand if its provenance is shown.
//! Compiled only with OPENRNG_LIB_DIR (cfg `openrng_linked`); prints `KAT ...` lines.
#![cfg(openrng_linked)]
use aocl_openrng_sys::*;

fn words(seed: openrng_uint_t) -> ([u32; 8], [f64; 4]) {
    let mut s: VSLStreamStatePtr = core::ptr::null_mut();
    let mut bits = [0u32; 8];
    let mut d = [0f64; 4];
    unsafe {
        assert_eq!(
            vslNewStream(&mut s, VSL_BRNG_PHILOX4X32X10 as openrng_int_t, seed),
            VSL_ERROR_OK
        );
        assert_eq!(
            viRngUniformBits32(0 as openrng_int_t, s, 8, bits.as_mut_ptr()),
            VSL_ERROR_OK
        );
        assert_eq!(vslDeleteStream(&mut s), VSL_ERROR_OK);
        assert_eq!(
            vslNewStream(&mut s, VSL_BRNG_PHILOX4X32X10 as openrng_int_t, seed),
            VSL_ERROR_OK
        );
        assert_eq!(
            vdRngUniform(
                VSL_RNG_METHOD_UNIFORM_STD as openrng_int_t,
                s,
                4,
                d.as_mut_ptr(),
                0.0,
                1.0
            ),
            VSL_ERROR_OK
        );
        assert_eq!(vslDeleteStream(&mut s), VSL_ERROR_OK);
    }
    (bits, d)
}

#[test]
fn record_philox_words() {
    for seed in [42 as openrng_uint_t, 0] {
        let (bits, d) = words(seed);
        let hex: Vec<String> = bits.iter().map(|w| format!("{w:08x}")).collect();
        println!(
            "KAT PHILOX4X32X10 seed={seed:<3} viRngUniformBits32 x8 = {}",
            hex.join(" ")
        );
        let dh: Vec<String> = d.iter().map(|x| format!("{:016x}", x.to_bits())).collect();
        println!(
            "KAT PHILOX4X32X10 seed={seed:<3} vdRngUniform[0,1) x4 bits = {} values = {:?}",
            dh.join(" "),
            d
        );
        let (bits2, d2) = words(seed);
        println!(
            "KAT seed={seed} repeat identical: {}",
            bits == bits2 && d == d2
        );
    }
}
