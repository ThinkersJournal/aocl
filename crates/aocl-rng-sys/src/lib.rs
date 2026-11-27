//! Raw FFI declarations for AOCL-RNG, written clean-room from AMD's published documentation.
//!
//! The declarations come from AMD's *AOCL API Guide 6.0* (document 68552), chapter 9.3, through a writer that was
//! given nothing but that text. No AOCL header is read, parsed or copied by this crate or its build. Per-function
//! provenance (guide line numbers) is `docs/cleanroom/rng_provenance.csv`; the integer-width measurement is
//! `docs/cleanroom/RNG-WIDTH-MEASUREMENT.md` in the repository.
//!
//! Declared: the lowercase, no-underscore C form (scalar inputs by value) and the UPPERCASE `S/DRAND*` forms
//! (`decls_upper.rs`, exported by both 6.0.68 DLLs; the guide states them mostly by pointer). The guide's
//! trailing-underscore forms are **not exported** by the 6.0.68 Windows DLLs (measured), so they are not declared;
//! `GET_RNGVERSION` is stated by the guide but not exported, so it is not declared either.
//!
//! `srandinitializeuser`/`SRANDINITIALIZEUSER` are declared with MEASURED callback typedefs but are unusable on 6.0.68 Windows as documented (see `decls_user_s.rs`).
//!
//! Not declared: the SecureRNG API unless the `securerng` feature is on, the SecureRNG API. With `securerng` the 15 guide-stated AOCL-SecureRNG
//! functions are declared and `amdsecrng` is linked (Windows only; measured: `docs/cleanroom/secrng/SECRNG-MEASUREMENT.md`).
//!
//! For a safe, idiomatic API see [`aocl-rng`](https://docs.rs/aocl-rng).

#![allow(
    non_camel_case_types,
    dead_code,
    unused_imports,
    improper_ctypes,
    reason = "declarations mirror the guide's C names and signatures"
)]

/// The AOCL-RNG integer type: 32-bit in the LP64 libraries, 64-bit in the ILP64 libraries.
///
/// Measured on the Windows 6.0.68 DLLs, not stated by the guide: see `docs/cleanroom/RNG-WIDTH-MEASUREMENT.md`.
#[cfg(not(feature = "ilp64"))]
pub type rng_int_t = core::ffi::c_int;
/// The AOCL-RNG integer type: 32-bit in the LP64 libraries, 64-bit in the ILP64 libraries.
#[cfg(feature = "ilp64")]
pub type rng_int_t = i64;

/// User initialiser callback for `drandinitializeuser` (shape stated at guide 6.0 L112219).
pub type RNG_DRANDINITIALIZEUSER_UINI = Option<
    unsafe extern "C" fn(
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
    ),
>;
/// User generator callback for `drandinitializeuser` (shape stated at guide 6.0 L112222).
pub type RNG_DRANDINITIALIZEUSER_UGEN = Option<
    unsafe extern "C" fn(*mut rng_int_t, *mut rng_int_t, *mut core::ffi::c_double, *mut rng_int_t),
>;

/// User initialiser callback for `srandinitializeuser`: the guide never states it; MEASURED to have the same 7-pointer shape as the D one (docs/cleanroom/types/TYPES-MEASUREMENT.md, Windows 6.0.68).
pub type RNG_SRANDINITIALIZEUSER_UINI = Option<
    unsafe extern "C" fn(
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
        *mut rng_int_t,
    ),
>;
/// User generator callback for `srandinitializeuser`: MEASURED: as the D one with a `float` output array.
pub type RNG_SRANDINITIALIZEUSER_UGEN = Option<
    unsafe extern "C" fn(*mut rng_int_t, *mut rng_int_t, *mut core::ffi::c_float, *mut rng_int_t),
>;

mod decls;
pub use decls::*;
#[cfg(feature = "securerng")]
mod decls_secure;
#[cfg(feature = "securerng")]
pub use decls_secure::*;
mod decls_user_s;
pub use decls_user_s::*;
mod decls_upper;
pub use decls_upper::*;
