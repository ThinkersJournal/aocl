# aocl-math-sys

Raw FFI bindings to AOCL-LibM. For the safe API see [`aocl-math`](../aocl-math/).

Dual-licensed under MIT or Apache-2.0.

## What is declared, and what is not (AOCL-LibM 6.0)

**Declared (218 functions):** the scalar functions and the array-form functions (`amd_vrda_*`, `amd_vrsa_*`: a length plus
pointers), exactly as the AOCL API Guide 6.0 states them.

**Not declared, and why:**
- **156 by-value SIMD functions on stable** (`amd_vrs4_*`, `amd_vrd2_*`, `amd_vrd4_*`, `amd_vrs8_*`, `amd_vrd8_*`, `amd_vrs16_*`, ...) take `__m128`..`__m512d`
  by value; stable Rust rejects SIMD types in an `extern "C"` block. They are declared behind the **nightly-only** Cargo feature
  `simd_ffi` (needs `rustc` `feature(simd_ffi)`; never default, not enabled for docs.rs), IN ADDITION to the array forms.
  Tested on `rustc 1.101.0-nightly (1c3a65069 2026-11-02)`, Windows x86_64, AOCL 6.0.68: `docs/cleanroom/probe/cargo-test-simd.ps1`
  checks `amd_vrs4_sinf`/`cosf`, `amd_vrd2_sin`, `amd_vrs8_sinf`, `amd_vrd4_sin`, `amd_vrs16_sinf` lane by lane against the scalar
  functions. **Coverage: 154 SIMD declarations; 6 exercised by tests (called, lanes compared); the other 148 are declared from the guide and linked only (`all_154_simd_symbols_link` takes every address, so each resolves in the installed `libalm`).** The guide states 156; `amd_vrd8_fabs` and `amd_vrs16_fabsf` are NOT provided by the installed 6.0.68 `libalm` (unresolved), so they are not declared. Scalar/array: **198 declarations; 7 scalar and 1 array function called by tests; all 198 link-checked (`tests/link_all.rs`).** The guide also states 20 `amd_fast*` scalar functions that 6.0.68 does not provide (dynamic or static): removed, listed in `docs/cleanroom/math-guide-stated-not-linkable.txt` (they were declared in the published 0.2.7).
- **8 functions with a type the guide never defines:** `amd_nexttoward`, `amd_nexttowardf` (`long double`) and `amd_cexp`,
  `amd_cexpf`, `amd_clog`, `amd_clogf`, `amd_cpow`, `amd_cpowf` (`fc32_t`, `fc64_t`): the ABI cannot be derived from the guide.
- **1 unreadable:** `amd_vrd2_sincos`: its line in the guide text is garbled; it stays UNKNOWN rather than guessed.

**Provenance:** the declarations were written from AMD's public guide 6.0 (document 68552), chapter 8, by a separate session
that was given only that text; names and signatures only. The authors of this repository have read the AOCL headers; that
session had not. Per-function guide line numbers: `docs/cleanroom/math_provenance.csv`. See `NOTICE` for AMD's legal line as the
guide states it (it grants no licence; whether names and signatures need permission is UNVERIFIED).

**AOCL is not shipped.** This crate links an AOCL-LibM that you have installed; it contains no AMD library, header or documentation.
