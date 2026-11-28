//! Raw FFI bindings to AOCL-FFTW.
//!
//! For a safe, idiomatic API see [`aocl-fft`](https://docs.rs/aocl-fft).
//!
//! # Licence condition of the library you link
//!
//! The AMD `libfftw3`/`libfftw3f` in AOCL is GPL-licensed (AOCL's `NOTICE.txt` says GPL-2.0; the shipped `COPYRIGHT` says
//! version 2 or any later version: the two disagree). This crate ships no library. A program that links it and is
//! distributed must comply with the library's licence. Whether a source-only crate of `extern` declarations is itself a
//! combined work is UNKNOWN (no FSF text found). This is not legal advice.

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    improper_ctypes,
    clippy::all,
    reason = "bindgen output mirrors the C headers: names, unused items and C-only types are not ours to change"
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Compile-time guard: `fftw_iodim` is `{ int n, is, os; }` and `fftw_iodim64` is `{ ptrdiff_t n, is, os; }` (fftw3.h), passed
// as arrays (`const fftw_iodim *dims`); no alignment attribute (docs/SYS-LAYOUT-AUDIT-20261127.md).
const _: () = {
    assert!(core::mem::size_of::<fftw_iodim>() == 12);
    assert!(core::mem::align_of::<fftw_iodim>() == 4);
    #[cfg(target_pointer_width = "64")]
    {
        assert!(core::mem::size_of::<fftw_iodim64>() == 24);
        assert!(core::mem::align_of::<fftw_iodim64>() == 8);
    }
};
