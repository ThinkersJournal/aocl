use crate::{rng_int_t, RNG_SRANDINITIALIZEUSER_UGEN, RNG_SRANDINITIALIZEUSER_UINI};

// Single-precision user-generator entry points: the guide states the prototypes (L112211, and the uppercase form L112198) but
// never the callback typedefs; the callback shapes are MEASURED (docs/cleanroom/types/TYPES-MEASUREMENT.md).
// WARNING: unsafe protocol, measured and not documented by the guide (see the doc comments below and docs/cleanroom/types/userinit/).
// Declared and link-checked only; not called by any test.
extern "C" {
    /// **Unsafe protocol, measured not documented.** AOCL 6.0.68 Windows: after `uini` returns the library reads `n = state[0]` and writes
    /// `state[n]`, `state[n+1]` (the `ugen` address split into two integers) and `state[state[0]+2]`; `n` is never checked against `lstate`, and
    /// `uini` that leaves `state[0] = 0` makes it write ~`hi32(ugen)+2` slots away (an out-of-bounds write). The guide does not state this.
    /// See `docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`. No safe wrapper exists, and no test calls this.
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
    /// **Unsafe protocol, measured not documented.** AOCL 6.0.68 Windows: after `uini` returns the library reads `n = state[0]` and writes
    /// `state[n]`, `state[n+1]` (the `ugen` address split into two integers) and `state[state[0]+2]`; `n` is never checked against `lstate`, and
    /// `uini` that leaves `state[0] = 0` makes it write ~`hi32(ugen)+2` slots away (an out-of-bounds write). The guide does not state this.
    /// See `docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`. No safe wrapper exists, and no test calls this.
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
