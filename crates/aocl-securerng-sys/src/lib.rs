//! Raw FFI declarations for AOCL-SecureRNG (RDRAND / RDSEED), written clean-room from AMD's published documentation.
//!
//! The 15 declarations come from AMD's *AOCL API Guide 6.0* (document 68552), the SecureRNG section, through a writer session
//! given nothing but that text; no AOCL header is read, parsed or copied by this crate or its build. Provenance:
//! `docs/cleanroom/secrng/secrng_provenance.csv`. Behaviour (output widths, return codes, NULL and N = 0, retry semantics) was
//! MEASURED against the installed 6.0.68 DLLs, LP64 and ILP64 (identical): `docs/cleanroom/secrng/SECRNG-MEASUREMENT.md`, with
//! the vacuous controls listed. Measured codes: success 2, failure -2, NULL output pointer -3, `is_*_supported` 1 when supported;
//! the guide names `SECRNG_SUCCESS` etc. but gives no values, so no constants are exported here.
//!
//! Windows is the only platform measured (the Linux library name `secrng` is as before, unmeasured).
//!
//! For a safe, idiomatic API see [`aocl-securerng`](https://docs.rs/aocl-securerng).

#![allow(
    non_snake_case,
    dead_code,
    reason = "declarations mirror the guide's C names and signatures"
)]

mod decls;
pub use decls::*;
