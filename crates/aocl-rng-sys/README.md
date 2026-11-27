# aocl-rng-sys

Raw FFI declarations for AOCL-RNG (the lowercase C forms), written clean-room from AMD's *AOCL API Guide 6.0* (doc 68552), chapter 9.3: no AOCL header is read by this crate or its build; link-only, no bindgen. Provenance: `docs/cleanroom/rng_provenance.csv`. For the safe API see [`aocl-rng`](../aocl-rng/).

**Integer width.** The guide never states `rng_int_t`'s width. It is a MEASUREMENT (AOCL-RNG 6.0.0 build 20261027, AOCL 6.0.68, Windows x86_64 only): 4 bytes in the LP64 library (`c_int`), 8 in the ILP64 library (`i64`, feature `ilp64`). See `docs/cleanroom/RNG-WIDTH-MEASUREMENT.md`. Linux and the self-hosted rig are not measured.

**Not declared, and why**
- UPPERCASE forms (`DRANDINITIALIZE`, ...): exported and stated by the guide, but not needed yet; added on demand.
- Trailing-underscore forms (`drandinitialize_`, ...) and `GET_RNGVERSION`: never. The 6.0.68 DLLs do not export them (the yanked 0.1.0 declared 80 such symbols: a link-error defect).
- `srandinitializeuser`: the guide does not state its callback typedefs (`RNG_SRANDINITIALIZEUSER_UINI/UGEN`).
- `fc32_t`/`fc64_t` and `long double` items: the guide names the types without defining them.
- SecureRNG (`amdsecrng`): not probed; **no SecureRNG claim is made**.

Dual-licensed under MIT or Apache-2.0. Not published; the RNG release waits on CireSnave's answer (board 523).

## Coverage of the tests (what is exercised, what is only linked)

- Lowercase (79) and UPPERCASE (77) declarations: **every symbol link-checked** (`tests/link_all_lower.rs`, `tests/upper_kat.rs`); called with a known answer: `drandinitialize`/`dranduniform` and the uppercase `DRANDINITIALIZE`/`DRANDUNIFORM` (the rest are declared from the guide and linked only).
- Feature `securerng` (Windows link only; Linux library name not measured): **15 declarations, all 15 link-checked and all 15 probed** against the DLLs in LP64 and ILP64 (output widths by sentinel bytes, return codes, NULL and N=0 behaviour); see `docs/cleanroom/secrng/SECRNG-MEASUREMENT.md` for the controls that were vacuous.
