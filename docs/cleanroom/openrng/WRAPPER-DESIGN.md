# aocl-openrng safe wrapper: DESIGN ONLY (no code until the PM reads aocl-openrng-sys; nothing here is built or measured)

- `Stream` owns a `VSLStreamStatePtr`; `Drop` calls `vslDeleteStream`; `!Send`/`!Sync` until the guide's thread-safety section (ch. 10.1.7, L116320) is read and measured; `Clone` = `vslCopyStream`; `copy_state_from` = `vslCopyStreamState`.
- Constructors typed by generator: `Stream::new(Brng, seed)` (`vslNewStream`), `Stream::from_params(Brng, &[u32])` (`vslNewStreamEx`); `Brng` is an enum over the 15 `VSL_BRNG_*` ids; `NONDETERM` and the ABSTRACT/user ids and `vslRegisterBrng`/`vsl?NewAbstractStream` stay unsafe-only in the `-sys` crate (callbacks; the guide says the function pointer types are unused).
- Distributions as methods taking `&mut Stream` and an output slice: `uniform(&mut self, out: &mut [f64], a, b)` -> `n = out.len()`, `Result<(), Error>` mapping `VSL_ERROR_*` / `VSL_RNG_ERROR_*` (constants in `-sys`); parameter validation in Rust only where the guide states a domain (to be read per distribution).
- Method enums per distribution (`VSL_RNG_METHOD_*`); `n` is `openrng_int_t`: checked conversion from `usize`, error on overflow (the width is feature-selected: `ilp64`).
- Leapfrog/skip-ahead: `vslLeapfrogStream`, `vslSkipAheadStream(Ex)` as `Result` methods; unsupported-by-generator errors surfaced.
- Save/load: `vslSaveStreamF/M`, `vslLoadStreamF/M` with `&CStr`/`&mut [u8]` after the size query `vslGetStreamSize`.
- `version()` via `openRngGetVersion` (header only, not in the guide).
- Tests: a known-answer per generator MEASURED on the built library, determinism, negative controls (wrong width, wrong seed), no test that calls an undocumented protocol.
