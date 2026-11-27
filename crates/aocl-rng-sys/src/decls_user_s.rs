use crate::{rng_int_t, RNG_SRANDINITIALIZEUSER_UGEN, RNG_SRANDINITIALIZEUSER_UINI};

// Single-precision user-generator entry points: the guide states the prototypes (L112211, and the uppercase form L112198) but
// never the callback typedefs; the callback shapes are MEASURED (docs/cleanroom/types/TYPES-MEASUREMENT.md).
// WARNING (measured on 6.0.68 Windows): this entry point writes `state[hi32+2]` about 128 KB past the start of the state and a
// following `srand*` returns info -4 without calling the generator; it could not be driven through the public API as documented.
// Declared and link-checked only; not called by any test.
extern "C" {
    pub fn srandinitializeuser(uini: RNG_SRANDINITIALIZEUSER_UINI, ugen: RNG_SRANDINITIALIZEUSER_UGEN, genid: rng_int_t, subid: rng_int_t, seed: *mut rng_int_t, lseed: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    pub fn SRANDINITIALIZEUSER(uini: RNG_SRANDINITIALIZEUSER_UINI, ugen: RNG_SRANDINITIALIZEUSER_UGEN, genid: rng_int_t, subid: rng_int_t, seed: *mut rng_int_t, lseed: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
}
