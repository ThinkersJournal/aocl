// SPDX-FileCopyrightText: Copyright 2024-2026 Arm Limited and/or its affiliates <open-source-office@arm.com>
// SPDX-FileCopyrightText: Copyright (c) 2026 Advanced Micro Devices, Inc. (AMD fork, AOCL-OpenRNG 6.0.0)
// SPDX-License-Identifier: MIT OR Apache-2.0 WITH LLVM-exception
//
// GENERATED mechanically (no hand edits) from include/openrng.h of github.com/amd/openrng, tag 6.0.0, commit 5c05f2eeb6effc696b0df6ce8be1fcbe855b631b,
// header sha256 ee3a48fcb50a9a3ded3300bfd342b8a7c7bf8b534f38233be93b26ced577386b, as read through the GitHub API. Licence: the header is "MIT OR Apache-2.0 WITH LLVM-exception"
// (Arm Limited); see ../NOTICE and ../LICENSE-ARM-OPENRNG / ../LICENSE-AMD-OPENRNG. Per-declaration provenance:
// docs/cleanroom/openrng/provenance.csv at https://github.com/ThinkersJournal/aocl.

/// include/openrng.h:L167
pub const VSL_STATUS_OK: core::ffi::c_int = 0;
/// include/openrng.h:L174
pub const VSL_ERROR_OK: core::ffi::c_int = 0;
/// include/openrng.h:L181
pub const VSL_ERROR_FEATURE_NOT_IMPLEMENTED: core::ffi::c_int = -1;
/// include/openrng.h:L189
pub const VSL_ERROR_BADARGS: core::ffi::c_int = -2;
/// include/openrng.h:L196
pub const VSL_ERROR_MEM_FAILURE: core::ffi::c_int = -3;
/// include/openrng.h:L203
pub const VSL_ERROR_NULL_PTR: core::ffi::c_int = -4;
/// include/openrng.h:L211
pub const VSL_RNG_ERROR_INVALID_BRNG_INDEX: core::ffi::c_int = -0x1000;
/// include/openrng.h:L219
pub const VSL_RNG_ERROR_LEAPFROG_UNSUPPORTED: core::ffi::c_int = -0x1001;
/// include/openrng.h:L227
pub const VSL_RNG_ERROR_SKIPAHEAD_UNSUPPORTED: core::ffi::c_int = -0x1002;
/// include/openrng.h:L235
pub const VSL_RNG_ERROR_SKIPAHEADEX_UNSUPPORTED: core::ffi::c_int = -0x1003;
/// include/openrng.h:L242
pub const VSL_RNG_ERROR_BRNGS_INCOMPATIBLE: core::ffi::c_int = -0x1004;
/// include/openrng.h:L249
pub const VSL_RNG_ERROR_BAD_STREAM: core::ffi::c_int = -0x1005;
/// include/openrng.h:L256
pub const VSL_RNG_ERROR_BAD_MEM_FORMAT: core::ffi::c_int = -0x1006;
/// include/openrng.h:L264
pub const VSL_RNG_ERROR_BRNG_NOT_SUPPORTED: core::ffi::c_int = -0x1007;
/// include/openrng.h:L271
pub const VSL_RNG_ERROR_FILE_OPEN: core::ffi::c_int = -0x1008;
/// include/openrng.h:L278
pub const VSL_RNG_ERROR_FILE_WRITE: core::ffi::c_int = -0x1009;
/// include/openrng.h:L285
pub const VSL_RNG_ERROR_FILE_READ: core::ffi::c_int = -0x100a;
/// include/openrng.h:L292
pub const VSL_RNG_ERROR_FILE_CLOSE: core::ffi::c_int = -0x100b;
/// include/openrng.h:L300
pub const VSL_RNG_ERROR_NONDETERM_NOT_SUPPORTED: core::ffi::c_int = -0x100c;
/// include/openrng.h:L308
pub const VSL_DISTR_MULTINOMIAL_BAD_PROBABILITY_ARRAY: core::ffi::c_int = -0x100d;
/// include/openrng.h:L318
pub const VSL_USER_QRNG_INITIAL_VALUES: core::ffi::c_int = 0x1;
/// include/openrng.h:L332
pub const VSL_USER_PRIMITIVE_POLYMS: core::ffi::c_int = 0x1;
/// include/openrng.h:L349
pub const VSL_USER_INIT_DIRECTION_NUMBERS: core::ffi::c_int = 0x2;
/// include/openrng.h:L363
pub const VSL_USER_DIRECTION_NUMBERS: core::ffi::c_int = 0x4;
/// include/openrng.h:L380
pub const VSL_QRNG_OVERRIDE_1ST_DIM_INIT: core::ffi::c_int = 0x8;
/// include/openrng.h:L473
pub const VSL_BRNG_MCG31: core::ffi::c_int = 0x1000;
/// include/openrng.h:L557
pub const VSL_BRNG_R250: core::ffi::c_int = 0x1001;
/// include/openrng.h:L653
pub const VSL_BRNG_MRG32K3A: core::ffi::c_int = 0x1002;
/// include/openrng.h:L743
pub const VSL_BRNG_MCG59: core::ffi::c_int = 0x1003;
/// include/openrng.h:L752
pub const VSL_BRNG_WH: core::ffi::c_int = 0x1004;
/// include/openrng.h:L939
pub const VSL_BRNG_SOBOL: core::ffi::c_int = 0x1005;
/// include/openrng.h:L948
pub const VSL_BRNG_NIEDERR: core::ffi::c_int = 0x1006;
/// include/openrng.h:L1043
pub const VSL_BRNG_MT19937: core::ffi::c_int = 0x1007;
/// include/openrng.h:L1179
pub const VSL_BRNG_MT2203: core::ffi::c_int = 0x10000;
/// include/openrng.h:L1188
pub const VSL_BRNG_IABSTRACT: core::ffi::c_int = 0x1009;
/// include/openrng.h:L1198
pub const VSL_BRNG_DABSTRACT: core::ffi::c_int = 0x100a;
/// include/openrng.h:L1208
pub const VSL_BRNG_SABSTRACT: core::ffi::c_int = 0x100b;
/// include/openrng.h:L1267
pub const VSL_BRNG_SFMT19937: core::ffi::c_int = 0x100c;
/// include/openrng.h:L1321
pub const VSL_BRNG_NONDETERM: core::ffi::c_int = 0x100d;
/// include/openrng.h:L1445
pub const VSL_BRNG_ARS5: core::ffi::c_int = 0x100e;
/// include/openrng.h:L1571
pub const VSL_BRNG_PHILOX4X32X10: core::ffi::c_int = 0x100f;
/// include/openrng.h:L1596
pub const VSL_RNG_METHOD_UNIFORM_STD: core::ffi::c_int = 0;
/// include/openrng.h:L1613
pub const VSL_RNG_METHOD_UNIFORM_STD_ACCURATE: core::ffi::c_int = 1;
/// include/openrng.h:L1627
pub const VSL_RNG_METHOD_UNIFORMBITS_STD: core::ffi::c_int = 0;
/// include/openrng.h:L1645
pub const VSL_RNG_METHOD_UNIFORMBITS32_STD: core::ffi::c_int = 0;
/// include/openrng.h:L1663
pub const VSL_RNG_METHOD_UNIFORMBITS64_STD: core::ffi::c_int = 0;
/// include/openrng.h:L1678
pub const VSL_RNG_METHOD_GAUSSIAN_BOXMULLER: core::ffi::c_int = 0;
/// include/openrng.h:L1694
pub const VSL_RNG_METHOD_GAUSSIAN_BOXMULLER2: core::ffi::c_int = 1;
/// include/openrng.h:L1705
pub const VSL_RNG_METHOD_GAUSSIAN_ICDF: core::ffi::c_int = 2;
/// include/openrng.h:L1717
pub const VSL_RNG_METHOD_GAUSSIANMV_BOXMULLER: core::ffi::c_int = 0;
/// include/openrng.h:L1729
pub const VSL_RNG_METHOD_GAUSSIANMV_BOXMULLER2: core::ffi::c_int = 1;
/// include/openrng.h:L1741
pub const VSL_RNG_METHOD_GAUSSIANMV_ICDF: core::ffi::c_int = 2;
/// include/openrng.h:L1752
pub const VSL_RNG_METHOD_LAPLACE_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1765
pub const VSL_RNG_METHOD_LOGNORMAL_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1778
pub const VSL_RNG_METHOD_LOGNORMAL_BOXMULLER2: core::ffi::c_int = 2;
/// include/openrng.h:L1792
pub const VSL_RNG_METHOD_EXPONENTIAL_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1808
pub const VSL_RNG_METHOD_EXPONENTIAL_ICDF_ACCURATE: core::ffi::c_int = 1;
/// include/openrng.h:L1822
pub const VSL_RNG_METHOD_RAYLEIGH_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1837
pub const VSL_RNG_METHOD_RAYLEIGH_ICDF_ACCURATE: core::ffi::c_int = 1;
/// include/openrng.h:L1851
pub const VSL_RNG_METHOD_WEIBULL_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1866
pub const VSL_RNG_METHOD_WEIBULL_ICDF_ACCURATE: core::ffi::c_int = 1;
/// include/openrng.h:L1877
pub const VSL_RNG_METHOD_GUMBEL_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1913
pub const VSL_RNG_METHOD_GAMMA_GNORM: core::ffi::c_int = 0;
/// include/openrng.h:L1928
pub const VSL_RNG_METHOD_GAMMA_GNORM_ACCURATE: core::ffi::c_int = 1;
/// include/openrng.h:L1939
pub const VSL_RNG_METHOD_BERNOULLI_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1959
pub const VSL_RNG_METHOD_BINOMIAL_BTPE: core::ffi::c_int = 0;
/// include/openrng.h:L1970
pub const VSL_RNG_METHOD_GEOMETRIC_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L1981
pub const VSL_RNG_METHOD_CAUCHY_ICDF: core::ffi::c_int = 0;
/// include/openrng.h:L2006
pub const VSL_RNG_METHOD_POISSON_PTPE: core::ffi::c_int = 0;
/// include/openrng.h:L2048
pub const VSL_RNG_METHOD_POISSON_POISNORM: core::ffi::c_int = 1;
/// include/openrng.h:L2093
pub const VSL_RNG_METHOD_MULTINOMIAL_MULTPOISSON: core::ffi::c_int = 0;
/// include/openrng.h:L2101
pub const VSL_MATRIX_STORAGE_FULL: core::ffi::c_int = 0;
/// include/openrng.h:L2109
pub const VSL_MATRIX_STORAGE_PACKED: core::ffi::c_int = 1;
/// include/openrng.h:L2116
pub const VSL_MATRIX_STORAGE_DIAGONAL: core::ffi::c_int = 2;
