use crate::{rng_int_t, RNG_SRANDINITIALIZEUSER_UGEN, RNG_SRANDINITIALIZEUSER_UINI};

// Single-precision user-generator entry points: the guide states the prototypes (L112211, and the uppercase form L112198) but
// never the callback typedefs; the callback shapes are MEASURED (docs/cleanroom/types/TYPES-MEASUREMENT.md).
// WARNING: unsafe protocol, measured and not documented by the guide (see the doc comments below and docs/cleanroom/types/userinit/).
// Declared and link-checked only; not called by any test.
extern "C" {
    /// **Unsafe; undocumented protocol.** Requires the `uini` callback to set `state[0] = n` and `lstate >= n + 3`; the guide does not state
    /// this. Measured on AOCL 6.0.68 Windows (see `docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`). The library does not validate `n`
    /// against `lstate`, so a `uini` that does not follow it corrupts memory. No safe wrapper exists; the only call in the repository is `tests/user_init.rs`, in the measured-safe shape (UINI sets `state[0] = n`, `lstate >= n + 3`, guard region asserted).
    pub fn srandinitializeuser(
        uini: RNG_SRANDINITIALIZEUSER_UINI,
        ugen: RNG_SRANDINITIALIZEUSER_UGEN,
        genid: rng_int_t,
        subid: rng_int_t,
        seed: *mut rng_int_t,
        lseed: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    /// **Unsafe; undocumented protocol.** Requires the `uini` callback to set `state[0] = n` and `lstate >= n + 3`; the guide does not state
    /// this. Measured on AOCL 6.0.68 Windows (see `docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`). The library does not validate `n`
    /// against `lstate`, so a `uini` that does not follow it corrupts memory. No safe wrapper exists; the only call in the repository is `tests/user_init.rs`, in the measured-safe shape (UINI sets `state[0] = n`, `lstate >= n + 3`, guard region asserted).
    pub fn SRANDINITIALIZEUSER(
        uini: RNG_SRANDINITIALIZEUSER_UINI,
        ugen: RNG_SRANDINITIALIZEUSER_UGEN,
        genid: rng_int_t,
        subid: rng_int_t,
        seed: *mut rng_int_t,
        lseed: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
}
