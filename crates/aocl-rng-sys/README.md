# aocl-rng-sys

Raw FFI declarations for AOCL-RNG (the lowercase C forms and the UPPERCASE `S/DRAND*` forms), written clean-room from AMD's *AOCL API Guide 6.0* (doc 68552), chapter 9.3: no AOCL header is read by this crate or its build; link-only, no bindgen. Provenance: `docs/cleanroom/rng_provenance.csv` and `docs/cleanroom/rng_upper_provenance.csv`; see `NOTICE` (AMD's terms, and that this repository's authors read the headers; the declarations were written by separate sessions that read only the guide). For the safe API see [`aocl-rng`](../aocl-rng/).

**Integer width.** The guide never states `rng_int_t`'s width. It is a MEASUREMENT (AOCL-RNG 6.0.0 build 20261027, AOCL 6.0.68, Windows x86_64 only): 4 bytes in the LP64 library (`c_int`), 8 in the ILP64 library (`i64`, feature `ilp64`). See `docs/cleanroom/RNG-WIDTH-MEASUREMENT.md`. Linux and the self-hosted rig are not measured.

**Declared:** the lowercase C forms (scalar inputs by value) and the 77 UPPERCASE `S/DRAND*` forms the guide states and the libraries export (`src/decls_upper.rs`; all arguments by pointer as the guide states them, except the user-generator ones).

**Not declared, and why**
- `GET_RNGVERSION`: stated by the guide but not exported by the 6.0.68 libraries.
- Trailing-underscore forms (`drandinitialize_`, ...): never. The 6.0.68 DLLs do not export them (the yanked 0.1.0 declared 80 such symbols: a link-error defect).
- `srandinitializeuser`/`SRANDINITIALIZEUSER` (and the D forms): declared with the S callback typedefs MEASURED (7-pointer UINI; UGEN `(n*, state*, float* r, info*)`), link-checked, and called by ONE test only, `tests/user_init.rs`, in the measured-safe shape (UINI sets `state[0] = n`, `lstate >= n + 3`, a guard region asserted untouched; LP64 and ILP64, D and S); a call that does not follow the protocol is never made in CI. **Unsafe; undocumented protocol:** the UINI callback must set `state[0] = n` and `lstate >= n + 3`; the guide does not state this. Measured on AOCL 6.0.68 Windows (LP64 and ILP64; `docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`): after UINI returns the library reads `n = state[0]` and writes `state[n]`, `state[n+1]` and `state[state[0]+2]`, and does not check `n` against `lstate`. An earlier probe's "no working call sequence" was our call pattern, not a library defect. No safe wrapper exists.
