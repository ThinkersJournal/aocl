// GENERATED from docs/cleanroom/math_decls.rs. Guide-stated, not exported by AOCL 6.0.68 Windows libalm (Linux not measured). Needs features `guide-only-fast` + `simd_ffi` (nightly).
use core::arch::x86_64::{__m512, __m512d};

extern "C" {
    // L109760
    pub fn amd_vrd8_fabs(x: __m512d) -> __m512d;
    // L109749
    pub fn amd_vrs16_fabsf(x: __m512) -> __m512;
}
