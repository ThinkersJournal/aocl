# aocl-math-sys

Raw FFI bindings to AOCL-LibM. For the safe API see [`aocl-math`](../aocl-math/).

Dual-licensed under MIT or Apache-2.0.

## What is declared, and what is not (AOCL-LibM 6.0)

**Declared (218 functions):** the scalar functions and the array-form functions (`amd_vrda_*`, `amd_vrsa_*`: a length plus
pointers), exactly as the AOCL API Guide 6.0 states them.

**Not declared, and why:**
- **156 by-value SIMD functions** (`amd_vrs4_*`, `amd_vrd2_*`, `amd_vrd4_*`, `amd_vrs8_*`, `amd_vrd8_*`, `amd_vrs16_*`, ...) take `__m128`..`__m512d`
  by value; stable Rust rejects SIMD types in an `extern "C"` block (it needs the nightly `simd_ffi` feature). The array forms
  cover the same functions. They remain a recorded later row.
- **8 functions with a type the guide never defines:** `amd_nexttoward`, `amd_nexttowardf` (`long double`) and `amd_cexp`,
  `amd_cexpf`, `amd_clog`, `amd_clogf`, `amd_cpow`, `amd_cpowf` (`fc32_t`, `fc64_t`): the ABI cannot be derived from the guide.
- **1 unreadable:** `amd_vrd2_sincos`: its line in the guide text is garbled; it stays UNKNOWN rather than guessed.

**Provenance:** the declarations were written from AMD's public guide 6.0 (document 68552), chapter 8, by a separate session
that was given only that text; names and signatures only. The authors of this repository have read the AOCL headers; that
session had not. Per-function guide line numbers: `docs/cleanroom/math_provenance.csv`. See `NOTICE` for AMD's legal line as the
guide states it (it grants no licence; whether names and signatures need permission is UNVERIFIED).

**AOCL is not shipped.** This crate links an AOCL-LibM that you have installed; it contains no AMD library, header or documentation.
