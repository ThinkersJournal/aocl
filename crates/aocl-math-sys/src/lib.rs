//! Raw FFI declarations for AOCL-LibM, written clean-room from AMD's published documentation.
//!
//! The declarations in this crate come from AMD's *AOCL API Guide 6.0* (document 68552), chapter 8 only, through a
//! writer session that was given nothing but that text. No AOCL header is read, parsed or copied by this crate or its
//! build. The per-function provenance (guide line numbers) is `docs/cleanroom/math_provenance.csv` in the repository.
//!
//! Not declared, because the guide names a type it never defines so the ABI cannot be derived: `amd_nexttoward`,
//! `amd_nexttowardf` (`long double`) and the six complex functions (`fc32_t`/`fc64_t`). `amd_vrd2_sincos` is also
//! absent: its guide line is too garbled to read with certainty.
//!
//! The 156 vector-API functions the guide also states take `__m128`..`__m512d` by value, which stable Rust cannot declare in\n//! an `extern "C"` block (it needs the nightly `simd_ffi` feature). They are in `docs/cleanroom/math_decls.rs` and are not\n//! wired here. The previous header-derived crate declared the scalar API only, so no scalar item is lost.\n//!
//! For a safe, idiomatic API see [`aocl-math`](https://docs.rs/aocl-math).

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    unused_imports,
    improper_ctypes,
    reason = "declarations mirror the guide's C names and signatures"
)]

mod decls_scalar;
pub use decls_scalar::*;
