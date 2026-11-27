// GENERATED mechanically (no hand edits) from the 0.2.7 decls_scalar.rs / decls_simd.rs: declarations the guide (6.0, ch. 8) states
// but the installed AOCL 6.0.68 Windows libalm does NOT export (the linker reports them unresolved, dynamic and static alike).
// Linux is NOT measured. Feature `guide-only-fast` (default OFF): enabling it makes any use fail to link on Windows 6.0.68.
use core::ffi::{c_double, c_float};

extern "C" {
    // L107807
    pub fn amd_fastacos(x: c_double) -> c_double;
    // L107799
    pub fn amd_fastacosf(x: c_float) -> c_float;
    // L107788
    pub fn amd_fastasin(x: c_double) -> c_double;
    // L107780
    pub fn amd_fastasinf(x: c_float) -> c_float;
    // L107825
    pub fn amd_fastatan(x: c_double) -> c_double;
    // L107817
    pub fn amd_fastatanf(x: c_float) -> c_float;
    // L107747
    pub fn amd_fastcos(x: c_double) -> c_double;
    // L107739
    pub fn amd_fastcosf(x: c_float) -> c_float;
    // L107916
    pub fn amd_fasterf(x: c_double) -> c_double;
    // L107908
    pub fn amd_fasterff(x: c_float) -> c_float;
    // L107846
    pub fn amd_fastexp(x: c_double) -> c_double;
    // L107838
    pub fn amd_fastexpf(x: c_float) -> c_float;
    // L107867
    pub fn amd_fastlog(x: c_double) -> c_double;
    // L107859
    pub fn amd_fastlogf(x: c_float) -> c_float;
    // L107892
    pub fn amd_fastpow(x: c_double, y: c_double) -> c_double;
    // L107880
    pub fn amd_fastpowf(x: c_float, y: c_float) -> c_float;
    // L107728
    pub fn amd_fastsin(x: c_double) -> c_double;
    // L107720
    pub fn amd_fastsinf(x: c_float) -> c_float;
    // L107767
    pub fn amd_fasttan(x: c_double) -> c_double;
    // L107758
    pub fn amd_fasttanf(x: c_float) -> c_float;
}
