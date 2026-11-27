//! Raw FFI bindings to AOCL-Cryptography (ALCP).
//!
//! For a safe, idiomatic API see [`aocl-crypto`](https://docs.rs/aocl-crypto).

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

// Compile-time guard: `alc_cipher_state_t` has three `__attribute__((aligned(64))) Uint64 [64]` members (cipher.h): the struct is
// 64-aligned and 1600 bytes (3 * 512 + 4 rounded up to 64); bindgen reproduces that (C and C++ agree, measured:
// docs/SYS-LAYOUT-AUDIT-20261127.md). It is only ever passed by pointer.
const _: () = {
    assert!(core::mem::align_of::<alc_cipher_state_t>() == 64);
    assert!(core::mem::size_of::<alc_cipher_state_t>() == 1600);
};
