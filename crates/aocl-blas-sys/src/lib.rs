//! Raw FFI bindings to AOCL-BLAS (BLIS).
//!
//! This crate exposes the CBLAS C interface plus BLIS's native API, generated
//! by `bindgen` against the AOCL-installed headers. For a safe, idiomatic
//! API see the [`aocl-blas`](https://docs.rs/aocl-blas) crate.

#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    improper_ctypes,
    clippy::all,
    reason = "bindgen output mirrors the C headers: names, unused items and C-only types are not ours to change"
)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Compile-time guard for the types passed by pointer-to-array or by value (docs/SYS-LAYOUT-AUDIT-20261127.md). `scomplex` /
// `dcomplex` are `{ float real, imag; }` / `{ double real, imag; }` (blis.h): 8 bytes align 4, 16 bytes align 8, no alignment
// attribute. `rntm_t` is `BLIS_ATTRIB_ALIGN` = `__attribute__((aligned(64)))` (on `_WIN32` the macro is non-empty only for clang;
// the static BLIS objects reference LLVM's `__kmpc_*` OpenMP entry points, so they are clang-built); its size 192 is the
// C layout of the same header (LP64 and ILP64) and tests/rntm_layout.rs has the library fill one (feature `static-link`).
const _: () = {
    assert!(core::mem::size_of::<scomplex>() == 8);
    assert!(core::mem::align_of::<scomplex>() == 4);
    assert!(core::mem::size_of::<dcomplex>() == 16);
    assert!(core::mem::align_of::<dcomplex>() == 8);
    assert!(core::mem::align_of::<rntm_t>() == 64);
    #[cfg(windows)]
    assert!(core::mem::size_of::<rntm_t>() == 192);
};
