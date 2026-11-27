# aocl-rng-sys

Raw FFI declarations for AOCL-RNG (the lowercase C forms), written clean-room from AMD's *AOCL API Guide 6.0* (doc 68552), chapter 9.3: no AOCL header is read by this crate or its build; link-only, no bindgen. Provenance: `docs/cleanroom/rng_provenance.csv`. For the safe API see [`aocl-rng`](../aocl-rng/).

**Integer width.** The guide never states `rng_int_t`'s width. It is a MEASUREMENT (AOCL-RNG 6.0.0 build 20261027, AOCL 6.0.68, Windows x86_64 only): 4 bytes in the LP64 library (`c_int`), 8 in the ILP64 library (`i64`, feature `ilp64`). See `docs/cleanroom/RNG-WIDTH-MEASUREMENT.md`. Linux and the self-hosted rig are not measured.

**Not declared, and why**
- UPPERCASE forms (`DRANDINITIALIZE`, ...): exported and stated by the guide, but not needed yet; added on demand.
- Trailing-underscore forms (`drandinitialize_`, ...) and `GET_RNGVERSION`: never. The 6.0.68 DLLs do not export them (the yanked 0.1.0 declared 80 such symbols: a link-error defect).
- `srandinitializeuser`/`SRANDINITIALIZEUSER` (and the D forms): declared with the S callback typedefs MEASURED (7-pointer UINI; UGEN `(n*, state*, float* r, info*)`), link-checked only, never called by a test. **Unsafe protocol, measured and not documented by the guide:** after UINI returns, the library reads `n = state[0]` and writes `state[n]`, `state[n+1]` and `state[state[0]+2]`; `n` is not checked against `lstate`, and a UINI that leaves `state[0] = 0` makes the library write ~`hi32(UGEN address)+2` slots away (an out-of-bounds write). With UINI setting `state[0]=n` and `lstate >= n+3` it works (measured, LP64 and ILP64). An earlier probe's "no working call sequence" was our call pattern, not a fixed AMD defect (`docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`). No safe wrapper exists.
