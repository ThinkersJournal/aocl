//! Raw FFI declarations for AOCL-LibM, written clean-room from AMD's published documentation.
//!
//! The declarations in this crate come from AMD's *AOCL API Guide 6.0* (document 68552), chapter 8 only, through a
//! writer session that was given nothing but that text. No AOCL header is read, parsed or copied by this crate or its
//! build. The per-function provenance (guide line numbers) is `docs/cleanroom/math_provenance.csv` in the repository.
//!
//! **Stated by the guide but not exported by AOCL 6.0.68 on Windows (Linux not measured): 20 `amd_fast*`/`amd_fasterf*` scalar functions and `amd_vrd8_fabs`/`amd_vrs16_fabsf`. They are behind the default-OFF feature `guide-only-fast`; using them fails to link on Windows.** (0.2.7 declared the 20 by default: a defect.)
//!
//! The 8 functions with `long double` / `fc32_t` / `fc64_t` are declared on **Windows only** (`decls_types.rs`), with the ABI
//! MEASURED against AOCL 6.0.68 (`docs/cleanroom/types/`); Linux is unmeasured. `amd_vrd2_sincos` is absent: its guide line is
//! too garbled to read with certainty.
//!
//! The 156 vector-API functions the guide also states take `__m128`..`__m512d` by value, which stable Rust cannot declare in
//! an `extern "C"` block (it needs the nightly `simd_ffi` compiler feature). They are declared in `decls_simd.rs` behind
//! this crate's **nightly-only** `simd_ffi` Cargo feature (never default, never enabled for docs.rs); see `README.md`.
//!
//! For a safe, idiomatic API see [`aocl-math`](https://docs.rs/aocl-math).

#![cfg_attr(feature = "simd_ffi", feature(simd_ffi))]
#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_imports,
    improper_ctypes,
    reason = "declarations mirror the guide's C names and signatures"
)]

mod decls_scalar;
pub use decls_scalar::*;

#[cfg(feature = "simd_ffi")]
mod decls_simd;
#[cfg(feature = "simd_ffi")]
pub use decls_simd::*;

// Stated by the guide, NOT exported by AOCL 6.0.68 on Windows (Linux not measured); default OFF. See docs/cleanroom/math-guide-stated-not-linkable.txt.
#[cfg(feature = "guide-only-fast")]
mod decls_guide_only;
#[cfg(feature = "guide-only-fast")]
pub use decls_guide_only::*;
#[cfg(all(feature = "guide-only-fast", feature = "simd_ffi"))]
mod decls_guide_only_simd;
#[cfg(all(feature = "guide-only-fast", feature = "simd_ffi"))]
pub use decls_guide_only_simd::*;

// Windows only: ABI measured there, Linux unmeasured. See docs/cleanroom/types/TYPES-MEASUREMENT.md.
#[cfg(windows)]
mod decls_types;
#[cfg(windows)]
pub use decls_types::*;
