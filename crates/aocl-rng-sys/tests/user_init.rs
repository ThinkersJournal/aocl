//! Link-and-call test for the user-generator entry points in the ONE measured-safe shape (docs/cleanroom/types/userinit/
//! USERINIT-MEASUREMENT.md): the UINI callback sets `state[0] = n` (the guide does not say so; MEASURED), `lstate >= n + 3`, and the
//! state buffer is larger than `lstate` with a guard region whose bytes are asserted untouched afterwards. Own test binary: a call
//! that does NOT follow the protocol (UINI leaving `state[0] = 0`) writes `hi32(UGEN address) + 2` slots away and is NEVER made here
//! (the wrong-n control lives in the measurement document, not in CI). Every other call pattern of these entry points is `unsafe` and
//! undocumented by AMD: see the crate docs.
use aocl_rng_sys::*;

const N_STATE: usize = 5; // `n`: the UINI-declared user state length
const LSTATE: usize = 100; // >= n + 3
const GUARD: usize = 16;
const SENTINEL: rng_int_t = 0x5555;

unsafe extern "C" fn uini(
    _g: *mut rng_int_t,
    _s: *mut rng_int_t,
    _seed: *mut rng_int_t,
    _lseed: *mut rng_int_t,
    state: *mut rng_int_t,
    _lstate: *mut rng_int_t,
    info: *mut rng_int_t,
) {
    *state = N_STATE as rng_int_t; // the measured protocol: state[0] = n
    *info = 0;
}

unsafe extern "C" fn ugen_d(
    n: *mut rng_int_t,
    _state: *mut rng_int_t,
    r: *mut core::ffi::c_double,
    info: *mut rng_int_t,
) {
    for i in 0..*n as usize {
        *r.add(i) = 0.5 + 0.1875 * (i as f64 + 1.0);
    }
    *info = 0;
}

unsafe extern "C" fn ugen_s(
    n: *mut rng_int_t,
    _state: *mut rng_int_t,
    r: *mut core::ffi::c_float,
    info: *mut rng_int_t,
) {
    for i in 0..*n as usize {
        *r.add(i) = 0.5 + 0.1875 * (i as f32 + 1.0);
    }
    *info = 0;
}

fn state_with_guard() -> Vec<rng_int_t> {
    let mut v = vec![0 as rng_int_t; LSTATE + GUARD];
    for g in &mut v[LSTATE..] {
        *g = SENTINEL;
    }
    v
}

fn guard_intact(v: &[rng_int_t]) -> bool {
    v[LSTATE..].iter().all(|g| *g == SENTINEL)
}

#[test]
fn drandinitializeuser_then_dranduniform_returns_the_user_generator_values() {
    let mut state = state_with_guard();
    let mut seed: [rng_int_t; 1] = [1];
    let (mut lseed, mut lstate, mut info): (rng_int_t, rng_int_t, rng_int_t) =
        (1, LSTATE as rng_int_t, -99);
    let mut x = [0f64; 4];
    unsafe {
        drandinitializeuser(
            Some(uini),
            Some(ugen_d),
            5,
            7,
            seed.as_mut_ptr(),
            &mut lseed,
            state.as_mut_ptr(),
            &mut lstate,
            &mut info,
        );
        assert_eq!(info, 0, "init info");
        assert!(
            guard_intact(&state),
            "init wrote past lstate: {:?}",
            &state[LSTATE..]
        );
        dranduniform(4, 0.0, 1.0, state.as_mut_ptr(), x.as_mut_ptr(), &mut info);
        assert_eq!(info, 0, "uniform info");
    }
    assert!(guard_intact(&state), "generate wrote past lstate");
    // UGEN's values come through (a = 0, b = 1): 0.6875, 0.875, 1.0625, 1.25.
    assert_eq!(x, [0.6875, 0.875, 1.0625, 1.25]);
    // Control: the init really called UINI (state[0] is the n it set) and the values are not the fill pattern.
    assert_eq!(state[0], N_STATE as rng_int_t);
    assert_ne!(x, [0.0; 4]);
}

#[test]
fn srandinitializeuser_then_sranduniform_returns_the_user_generator_values() {
    let mut state = state_with_guard();
    let mut seed: [rng_int_t; 1] = [1];
    let (mut lseed, mut lstate, mut info): (rng_int_t, rng_int_t, rng_int_t) =
        (1, LSTATE as rng_int_t, -99);
    let mut x = [0f32; 4];
    unsafe {
        srandinitializeuser(
            Some(uini),
            Some(ugen_s),
            5,
            7,
            seed.as_mut_ptr(),
            &mut lseed,
            state.as_mut_ptr(),
            &mut lstate,
            &mut info,
        );
        assert_eq!(info, 0, "init info");
        assert!(guard_intact(&state), "init wrote past lstate");
        sranduniform(4, 0.0, 1.0, state.as_mut_ptr(), x.as_mut_ptr(), &mut info);
        assert_eq!(info, 0, "uniform info");
    }
    assert!(guard_intact(&state), "generate wrote past lstate");
    assert_eq!(x, [0.6875, 0.875, 1.0625, 1.25]);
    assert_eq!(state[0], N_STATE as rng_int_t);
}
