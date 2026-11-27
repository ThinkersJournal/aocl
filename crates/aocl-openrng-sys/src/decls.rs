// SPDX-FileCopyrightText: Copyright 2024-2026 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-FileCopyrightText: Copyright (c) 2026 Advanced Micro Devices, Inc. (AMD fork, AOCL-OpenRNG 6.0.0)
// SPDX-License-Identifier: MIT OR Apache-2.0 WITH LLVM-exception
//
// GENERATED mechanically (no hand edits) from include/openrng.h of github.com/amd/openrng, tag 6.0.0, commit 5c05f2eeb6effc696b0df6ce8be1fcbe855b631b,
// header sha256 ee3a48fcb50a9a3ded3300bfd342b8a7c7bf8b534f38233be93b26ced577386b, as read through the GitHub API. Licence: the header is "MIT OR Apache-2.0 WITH LLVM-exception"
// (Arm Limited); see ../NOTICE and ../LICENSE-ARM-OPENRNG / ../LICENSE-AMD-OPENRNG. Per-declaration provenance:
// docs/cleanroom/openrng/provenance.csv at https://github.com/ThinkersJournal/aocl.

use crate::{
    dUpdateFuncPtr, iUpdateFuncPtr, openrng_int_t, openrng_uint64_t, openrng_uint_t,
    sUpdateFuncPtr, VSLBRngProperties, VSLStreamStatePtr,
};

extern "C" {
    // include/openrng.h:L2282
    pub fn vdRngBeta(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        p: core::ffi::c_double,
        q: core::ffi::c_double,
        alpha: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2314
    pub fn vdRngCauchy(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2348
    pub fn vdRngChiSquare(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        v: core::ffi::c_int,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2384
    pub fn vdRngExponential(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2426
    pub fn vdRngGamma(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        alpha: core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2461
    pub fn vdRngGaussian(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        sigma: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2503
    pub fn vdRngGaussianMV(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        dimen: openrng_int_t,
        mstorage: openrng_int_t,
        a: *const core::ffi::c_double,
        t: *const core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2538
    pub fn vdRngGumbel(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2573
    pub fn vdRngLaplace(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2617
    pub fn vdRngLognormal(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        alpha: core::ffi::c_double,
        sigma: core::ffi::c_double,
        b: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2656
    pub fn vdRngRayleigh(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2692
    pub fn vdRngUniform(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        a: core::ffi::c_double,
        b: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2733
    pub fn vdRngWeibull(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_double,
        alpha: core::ffi::c_double,
        a: core::ffi::c_double,
        beta: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2767
    pub fn viRngBernoulli(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        p: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2797
    pub fn viRngBinomial(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        ntrial: core::ffi::c_int,
        p: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2828
    pub fn viRngGeometric(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        p: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2847
    pub fn viRngHypergeometric(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        l: core::ffi::c_int,
        s: core::ffi::c_int,
        m: core::ffi::c_int,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2893
    pub fn viRngMultinomial(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        ntrial: core::ffi::c_int,
        k: core::ffi::c_int,
        p: *const core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2912
    pub fn viRngNegBinomial(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        a: core::ffi::c_double,
        p: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2931
    pub fn viRngNegbinomial(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        a: core::ffi::c_double,
        p: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2957
    pub fn viRngPoisson(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        lambda: core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L2975
    pub fn viRngPoissonV(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        lambda: *const core::ffi::c_double,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3007
    pub fn viRngUniform(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_int,
        a: core::ffi::c_int,
        b: core::ffi::c_int,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3038
    pub fn viRngUniformBits(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        buffer: *mut core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3058
    pub fn viRngUniformBits32(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        buffer: *mut core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3078
    pub fn viRngUniformBits64(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        buffer: *mut openrng_uint64_t,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3121
    pub fn vsRngBeta(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        p: core::ffi::c_float,
        q: core::ffi::c_float,
        alpha: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3153
    pub fn vsRngCauchy(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3187
    pub fn vsRngChiSquare(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        v: core::ffi::c_int,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3223
    pub fn vsRngExponential(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3263
    pub fn vsRngGamma(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        alpha: core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3298
    pub fn vsRngGaussian(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        sigma: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3339
    pub fn vsRngGaussianMV(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        dimen: openrng_int_t,
        mstorage: openrng_int_t,
        a: *const core::ffi::c_float,
        t: *const core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3373
    pub fn vsRngGumbel(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3408
    pub fn vsRngLaplace(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3451
    pub fn vsRngLognormal(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        alpha: core::ffi::c_float,
        sigma: core::ffi::c_float,
        b: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3489
    pub fn vsRngRayleigh(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3526
    pub fn vsRngUniform(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        a: core::ffi::c_float,
        b: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3567
    pub fn vsRngWeibull(
        method: openrng_int_t,
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        r: *mut core::ffi::c_float,
        alpha: core::ffi::c_float,
        a: core::ffi::c_float,
        beta: core::ffi::c_float,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3586
    pub fn vslCopyStream(
        newstream: *mut VSLStreamStatePtr,
        srcstream: VSLStreamStatePtr,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3603
    pub fn vslCopyStreamState(
        dststream: VSLStreamStatePtr,
        srcstream: VSLStreamStatePtr,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3615
    pub fn vslDeleteStream(stream: *mut VSLStreamStatePtr) -> core::ffi::c_int;
    // include/openrng.h:L3630
    pub fn vslGetBrngProperties(
        brngId: core::ffi::c_int,
        properties: *mut VSLBRngProperties,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3641
    pub fn vslGetNumRegBrngs() -> core::ffi::c_int;
    // include/openrng.h:L3656
    pub fn vslGetStreamSize(stream: VSLStreamStatePtr) -> core::ffi::c_int;
    // include/openrng.h:L3667
    pub fn vslGetStreamStateBrng(stream: VSLStreamStatePtr) -> core::ffi::c_int;
    // include/openrng.h:L3697
    pub fn vslLeapfrogStream(
        stream: VSLStreamStatePtr,
        k: openrng_int_t,
        nstreams: openrng_int_t,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3713
    pub fn vslLoadStreamF(
        stream: *mut VSLStreamStatePtr,
        fname: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3728
    pub fn vslLoadStreamM(
        stream: *mut VSLStreamStatePtr,
        memptr: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3747
    pub fn vslNewStream(
        stream: *mut VSLStreamStatePtr,
        brng: openrng_int_t,
        seed: openrng_uint_t,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3769
    pub fn vslNewStreamEx(
        stream: *mut VSLStreamStatePtr,
        brng: openrng_int_t,
        n: openrng_int_t,
        params: *const core::ffi::c_uint,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3784
    pub fn vslRegisterBrng(properties: *const VSLBRngProperties) -> core::ffi::c_int;
    // include/openrng.h:L3796
    pub fn vslSaveStreamF(
        stream: VSLStreamStatePtr,
        fname: *const core::ffi::c_char,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3813
    pub fn vslSaveStreamM(
        stream: VSLStreamStatePtr,
        memptr: *mut core::ffi::c_char,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3836
    pub fn vslSkipAheadStream(
        stream: VSLStreamStatePtr,
        nskip: core::ffi::c_longlong,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3859
    pub fn vslSkipAheadStreamEx(
        stream: VSLStreamStatePtr,
        n: openrng_int_t,
        nskip: *const openrng_uint64_t,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3877
    pub fn vsldNewAbstractStream(
        stream: *mut VSLStreamStatePtr,
        n: openrng_int_t,
        dbuf: *const core::ffi::c_double,
        a: core::ffi::c_double,
        b: core::ffi::c_double,
        dcallback: dUpdateFuncPtr,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3894
    pub fn vsliNewAbstractStream(
        stream: *mut VSLStreamStatePtr,
        n: openrng_int_t,
        ibuf: *const core::ffi::c_uint,
        a: iUpdateFuncPtr,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3912
    pub fn vslsNewAbstractStream(
        stream: *mut VSLStreamStatePtr,
        n: openrng_int_t,
        sbuf: *const core::ffi::c_float,
        a: core::ffi::c_float,
        b: core::ffi::c_float,
        scallback: sUpdateFuncPtr,
    ) -> core::ffi::c_int;
    // include/openrng.h:L3929
    pub fn openRngGetVersion(major: *mut i32, minor: *mut i32, patch: *mut i32);
}
