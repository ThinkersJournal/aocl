//! Links the installed AOCL-RNG and checks `rng_int_t` against the library: a width mismatch makes the init fail
//! (measured: docs/cleanroom/RNG-WIDTH-MEASUREMENT.md). Known answer from the probe: genid 1, seed [12345].
use aocl_rng_sys::*;

#[test]
fn width_and_known_answer() {
    assert_eq!(
        core::mem::size_of::<rng_int_t>(),
        if cfg!(feature = "ilp64") { 8 } else { 4 }
    );
    let mut seed: [rng_int_t; 1] = [12345];
    let mut lseed: rng_int_t = 1;
    let mut state: [rng_int_t; 633] = [0; 633];
    let mut lstate: rng_int_t = 633;
    let mut info: rng_int_t = -99;
    let mut x = [0f64; 3];
    unsafe {
        drandinitialize(
            1,
            0,
            seed.as_mut_ptr(),
            &mut lseed,
            state.as_mut_ptr(),
            &mut lstate,
            &mut info,
        );
        assert_eq!(info, 0, "init info");
        dranduniform(3, 0.0, 1.0, state.as_mut_ptr(), x.as_mut_ptr(), &mut info);
        assert_eq!(info, 0, "uniform info");
    }
    assert_eq!(x[0].to_bits(), 0x3fef20e1d46d4343);
}
