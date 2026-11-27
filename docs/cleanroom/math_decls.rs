// Clean-room declarations from AOCL API Guide 6.0 (68552), ch. 8 AOCL-LibM only.
// Placeholder aliases for types the guide names but does not define (see math_notes.md).
use core::ffi::{c_char, c_double, c_float, c_int, c_long, c_longlong, c_void};
use core::arch::x86_64::{__m128, __m128d, __m256, __m256d, __m512, __m512d};
pub type long_double = c_void; // UNRESOLVED: guide says `long double`; no core type; ABI not representable
#[repr(C)] pub struct fc32_t { _opaque: [u8; 0] } // UNRESOLVED: complex single, not defined in guide
#[repr(C)] pub struct fc64_t { _opaque: [u8; 0] } // UNRESOLVED: complex double, not defined in guide

extern "C" {
    // L106231
    pub fn amd_sinf(x: c_float) -> c_float;
    // L106242
    pub fn amd_sin(x: c_double) -> c_double;
    // L106255
    pub fn amd_cosf(x: c_float) -> c_float;
    // L106265
    pub fn amd_cos(x: c_double) -> c_double;
    // L106278
    pub fn amd_tanf(x: c_float) -> c_float;
    // L106289
    pub fn amd_tan(x: c_double) -> c_double;
    // L106302
    pub fn amd_sincosf(x: c_float, s: *mut c_float, c: *mut c_float);
    // L106310
    pub fn amd_sincos(x: c_double, s: *mut c_double, c: *mut c_double);
    // L106323
    pub fn amd_asinf(x: c_float) -> c_float;
    // L106334
    pub fn amd_asin(x: c_double) -> c_double;
    // L106347
    pub fn amd_acosf(x: c_float) -> c_float;
    // L106357
    pub fn amd_acos(x: c_double) -> c_double;
    // L106370
    pub fn amd_atanf(x: c_float) -> c_float;
    // L106381
    pub fn amd_atan(x: c_double) -> c_double;
    // L106394
    pub fn amd_atan2f(x: c_float, y: c_float) -> c_float;
    // L106405
    pub fn amd_atan2(x: c_double, y: c_double) -> c_double;
    // L106421
    pub fn amd_sinhf(x: c_float) -> c_float;
    // L106432
    pub fn amd_sinh(x: c_double) -> c_double;
    // L106445
    pub fn amd_coshf(x: c_float) -> c_float;
    // L106455
    pub fn amd_cosh(x: c_double) -> c_double;
    // L106468
    pub fn amd_tanhf(x: c_float) -> c_float;
    // L106479
    pub fn amd_tanh(x: c_double) -> c_double;
    // L106494
    pub fn amd_asinhf(x: c_float) -> c_float;
    // L106504
    pub fn amd_asinh(x: c_double) -> c_double;
    // L106517
    pub fn amd_acoshf(x: c_float) -> c_float;
    // L106528
    pub fn amd_acosh(x: c_double) -> c_double;
    // L106541
    pub fn amd_atanhf(x: c_float) -> c_float;
    // L106551
    pub fn amd_atanh(x: c_double) -> c_double;
    // L106566
    pub fn amd_expf(x: c_float) -> c_float;
    // L106577
    pub fn amd_exp(x: c_double) -> c_double;
    // L106590
    pub fn amd_exp2f(x: c_float) -> c_float;
    // L106600
    pub fn amd_exp2(x: c_double) -> c_double;
    // L106613
    pub fn amd_exp10f(x: c_float) -> c_float;
    // L106624
    pub fn amd_exp10(x: c_double) -> c_double;
    // L106637
    pub fn amd_expm1f(x: c_float) -> c_float;
    // L106647
    pub fn amd_expm1(x: c_double) -> c_double;
    // L106662
    pub fn amd_logf(x: c_float) -> c_float;
    // L106673
    pub fn amd_log(x: c_double) -> c_double;
    // L106686
    pub fn amd_log2f(x: c_float) -> c_float;
    // L106696
    pub fn amd_log2(x: c_double) -> c_double;
    // L106709
    pub fn amd_log10f(x: c_float) -> c_float;
    // L106720
    pub fn amd_log10(x: c_double) -> c_double;
    // L106733
    pub fn amd_log1pf(x: c_float) -> c_float;
    // L106743
    pub fn amd_log1p(x: c_double) -> c_double;
    // L106758
    pub fn amd_powf(x: c_float, y: c_float) -> c_float;
    // L106770
    pub fn amd_pow(x: c_double, y: c_double) -> c_double;
    // L106784
    pub fn amd_sqrtf(x: c_float) -> c_float;
    // L106794
    pub fn amd_sqrt(x: c_double) -> c_double;
    // L106807
    pub fn amd_cbrtf(x: c_float) -> c_float;
    // L106818
    pub fn amd_cbrt(x: c_double) -> c_double;
    // L106831
    pub fn amd_hypotf(x: c_float, y: c_float) -> c_float;
    // L106842
    pub fn amd_hypot(x: c_double, y: c_double) -> c_double;
    // L106858
    pub fn amd_erff(x: c_float) -> c_float;
    // L106869
    pub fn amd_erf(x: c_double) -> c_double;
    // L106882
    pub fn amd_erfcf(x: c_float) -> c_float;
    // L106892
    pub fn amd_erfc(x: c_double) -> c_double;
    // L106905
    pub fn amd_erfinv(x: c_double) -> c_double;
    // L106918
    pub fn amd_erfcinv(x: c_double) -> c_double;
    // L106942
    pub fn amd_remainderf(x: c_float, y: c_float) -> c_float;
    // L106954
    pub fn amd_remainder(x: c_double, y: c_double) -> c_double;
    // L106968
    pub fn amd_remquof(x: c_float, y: c_float, quo: *mut c_int) -> c_float;
    // L106980
    pub fn amd_remquo(x: c_double, y: c_double, quo: *mut c_int) -> c_double;
    // L106995
    pub fn amd_fmodf(x: c_float, y: c_float) -> c_float;
    // L107007
    pub fn amd_fmod(x: c_double, y: c_double) -> c_double;
    // L107020
    pub fn amd_modff(x: c_float, iptr: *mut c_float) -> c_float;
    // L107032
    pub fn amd_modf(x: c_double, iptr: *mut c_double) -> c_double;
    // L107048
    pub fn amd_ceilf(x: c_float) -> c_float;
    // L107059
    pub fn amd_ceil(x: c_double) -> c_double;
    // L107071
    pub fn amd_floorf(x: c_float) -> c_float;
    // L107082
    pub fn amd_floor(x: c_double) -> c_double;
    // L107095
    pub fn amd_truncf(x: c_float) -> c_float;
    // L107106
    pub fn amd_trunc(x: c_double) -> c_double;
    // L107118
    pub fn amd_roundf(f: c_float) -> c_float;
    // L107129
    pub fn amd_round(f: c_double) -> c_double;
    // L107142
    pub fn amd_rintf(x: c_float) -> c_float;
    // L107154
    pub fn amd_rint(x: c_double) -> c_double;
    // L107167
    pub fn amd_nearbyintf(x: c_float) -> c_float;
    // L107178
    pub fn amd_nearbyint(x: c_double) -> c_double;
    // L107191
    pub fn amd_lrintf(x: c_float) -> c_long;
    // L107202
    pub fn amd_lrint(x: c_double) -> c_long;
    // L107214
    pub fn amd_llrintf(x: c_float) -> c_longlong;
    // L107225
    pub fn amd_llrint(x: c_double) -> c_longlong;
    // L107238
    pub fn amd_lroundf(f: c_float) -> c_long;
    // L107249
    pub fn amd_lround(d: c_double) -> c_long;
    // L107261
    pub fn amd_llroundf(f: c_float) -> c_longlong;
    // L107272
    pub fn amd_llround(d: c_double) -> c_longlong;
    // L107287
    pub fn amd_frexpf(value: c_float, exp: *mut c_int) -> c_float;
    // L107299
    pub fn amd_frexp(value: c_double, exp: *mut c_int) -> c_double;
    // L107312
    pub fn amd_ldexpf(x: c_float, exp: c_int) -> c_float;
    // L107324
    pub fn amd_ldexp(x: c_double, exp: c_int) -> c_double;
    // L107338
    pub fn amd_scalbnf(x: c_float, n: c_int) -> c_float;
    // L107350
    pub fn amd_scalbn(x: c_double, n: c_int) -> c_double;
    // L107363
    pub fn amd_scalblnf(x: c_float, n: c_long) -> c_float;
    // L107375
    pub fn amd_scalbln(x: c_double, n: c_long) -> c_double;
    // L107389
    pub fn amd_logbf(x: c_float) -> c_float;
    // L107400
    pub fn amd_logb(x: c_double) -> c_double;
    // L107412
    pub fn amd_ilogbf(x: c_float) -> c_int;
    // L107423
    pub fn amd_ilogb(x: c_double) -> c_int;
    // L107436
    pub fn amd_nextafterf(x: c_float, y: c_float) -> c_float;
    // L107448
    pub fn amd_nextafter(x: c_double, y: c_double) -> c_double;
    // L107461
    pub fn amd_nexttowardf(x: c_float, y: long_double) -> c_float;
    // L107473
    pub fn amd_nexttoward(x: c_double, y: long_double) -> c_double;
    // L107487
    pub fn amd_copysignf(x: c_float, y: c_float) -> c_float;
    // L107499
    pub fn amd_copysign(x: c_double, y: c_double) -> c_double;
    // L107512
    pub fn amd_nanf(tagp: *const c_char) -> c_float;
    // L107523
    pub fn amd_nan(tagp: *const c_char) -> c_double;
    // L107536
    pub fn amd_finitef(x: c_float) -> c_int;
    // L107547
    pub fn amd_finite(x: c_double) -> c_int;
    // L107563
    pub fn amd_fminf(x: c_float, y: c_float) -> c_float;
    // L107575
    pub fn amd_fmin(x: c_double, y: c_double) -> c_double;
    // L107589
    pub fn amd_fmaxf(x: c_float, y: c_float) -> c_float;
    // L107601
    pub fn amd_fmax(x: c_double, y: c_double) -> c_double;
    // L107614
    pub fn amd_fdimf(x: c_float, y: c_float) -> c_float;
    // L107626
    pub fn amd_fdim(x: c_double, y: c_double) -> c_double;
    // L107640
    pub fn amd_fabsf(x: c_float) -> c_float;
    // L107651
    pub fn amd_fabs(x: c_double) -> c_double;
    // L107720
    pub fn amd_fastsinf(x: c_float) -> c_float;
    // L107728
    pub fn amd_fastsin(x: c_double) -> c_double;
    // L107739
    pub fn amd_fastcosf(x: c_float) -> c_float;
    // L107747
    pub fn amd_fastcos(x: c_double) -> c_double;
    // L107758
    pub fn amd_fasttanf(x: c_float) -> c_float;
    // L107767
    pub fn amd_fasttan(x: c_double) -> c_double;
    // L107780
    pub fn amd_fastasinf(x: c_float) -> c_float;
    // L107788
    pub fn amd_fastasin(x: c_double) -> c_double;
    // L107799
    pub fn amd_fastacosf(x: c_float) -> c_float;
    // L107807
    pub fn amd_fastacos(x: c_double) -> c_double;
    // L107817
    pub fn amd_fastatanf(x: c_float) -> c_float;
    // L107825
    pub fn amd_fastatan(x: c_double) -> c_double;
    // L107838
    pub fn amd_fastexpf(x: c_float) -> c_float;
    // L107846
    pub fn amd_fastexp(x: c_double) -> c_double;
    // L107859
    pub fn amd_fastlogf(x: c_float) -> c_float;
    // L107867
    pub fn amd_fastlog(x: c_double) -> c_double;
    // L107880
    pub fn amd_fastpowf(x: c_float, y: c_float) -> c_float;
    // L107892
    pub fn amd_fastpow(x: c_double, y: c_double) -> c_double;
    // L107908
    pub fn amd_fasterff(x: c_float) -> c_float;
    // L107916
    pub fn amd_fasterf(x: c_double) -> c_double;
    // L107937
    pub fn amd_vrs4_sinf(x: __m128) -> __m128;
    // L107948
    pub fn amd_vrd2_sin(x: __m128d) -> __m128d;
    // L107961
    pub fn amd_vrs4_cosf(x: __m128) -> __m128;
    // L107971
    pub fn amd_vrd2_cos(x: __m128d) -> __m128d;
    // L107984
    pub fn amd_vrs4_tanf(x: __m128) -> __m128;
    // L107995
    pub fn amd_vrd2_tan(x: __m128d) -> __m128d;
    // L108008
    pub fn amd_vrs4_sincosf(x: __m128, sin: *mut __m128, cos: *mut __m128);
    // L108029
    pub fn amd_vrs4_asinf(x: __m128) -> __m128;
    // L108040
    pub fn amd_vrd2_asin(x: __m128d) -> __m128d;
    // L108053
    pub fn amd_vrs4_acosf(x: __m128) -> __m128;
    // L108063
    pub fn amd_vrd2_acos(x: __m128d) -> __m128d;
    // L108076
    pub fn amd_vrs4_atanf(x: __m128) -> __m128;
    // L108087
    pub fn amd_vrd2_atan(x: __m128d) -> __m128d;
    // L108098
    pub fn amd_vrd2_atan2(y: __m128d, x: __m128d) -> __m128d;
    // L108113
    pub fn amd_vrs4_coshf(x: __m128) -> __m128;
    // L108124
    pub fn amd_vrd2_cosh(x: __m128d) -> __m128d;
    // L108137
    pub fn amd_vrs4_tanhf(x: __m128) -> __m128;
    // L108148
    pub fn amd_vrd2_tanh(x: __m128d) -> __m128d;
    // L108164
    pub fn amd_vrs4_expf(x: __m128) -> __m128;
    // L108175
    pub fn amd_vrd2_exp(x: __m128d) -> __m128d;
    // L108188
    pub fn amd_vrs4_exp2f(x: __m128) -> __m128;
    // L108199
    pub fn amd_vrd2_exp2(x: __m128d) -> __m128d;
    // L108211
    pub fn amd_vrs4_exp10f(x: __m128) -> __m128;
    // L108222
    pub fn amd_vrd2_exp10(x: __m128d) -> __m128d;
    // L108235
    pub fn amd_vrs4_expm1f(x: __m128) -> __m128;
    // L108250
    pub fn amd_vrs4_logf(x: __m128) -> __m128;
    // L108260
    pub fn amd_vrd2_log(x: __m128d) -> __m128d;
    // L108273
    pub fn amd_vrs4_log2f(x: __m128) -> __m128;
    // L108284
    pub fn amd_vrd2_log2(x: __m128d) -> __m128d;
    // L108297
    pub fn amd_vrs4_log10f(x: __m128) -> __m128;
    // L108307
    pub fn amd_vrd2_log10(x: __m128d) -> __m128d;
    // L108320
    pub fn amd_vrs4_log1pf(x: __m128) -> __m128;
    // L108331
    pub fn amd_vrd2_log1p(x: __m128d) -> __m128d;
    // L108346
    pub fn amd_vrs4_powf(x: __m128, y: __m128) -> __m128;
    // L108357
    pub fn amd_vrd2_pow(x: __m128d, y: __m128d) -> __m128d;
    // L108371
    pub fn amd_vrs4_powxf(x: __m128, y: c_float) -> __m128;
    // L108383
    pub fn amd_vrd2_powx(x: __m128d, y: c_double) -> __m128d;
    // L108397
    pub fn amd_vrs4_sqrtf(x: __m128) -> __m128;
    // L108407
    pub fn amd_vrd2_sqrt(x: __m128d) -> __m128d;
    // L108420
    pub fn amd_vrs4_cbrtf(x: __m128) -> __m128;
    // L108431
    pub fn amd_vrd2_cbrt(x: __m128d) -> __m128d;
    // L108446
    pub fn amd_vrs4_erff(x: __m128) -> __m128;
    // L108456
    pub fn amd_vrd2_erf(x: __m128d) -> __m128d;
    // L108469
    pub fn amd_vrs4_erfcf(x: __m128) -> __m128;
    // L108480
    pub fn amd_vrd2_erfc(x: __m128d) -> __m128d;
    // L108493
    pub fn amd_vrd2_erfinv(x: __m128d) -> __m128d;
    // L108505
    pub fn amd_vrd2_erfcinv(x: __m128d) -> __m128d;
    // L108530
    pub fn amd_vrs4_roundf(x: __m128) -> __m128;
    // L108541
    pub fn amd_vrd2_round(x: __m128d) -> __m128d;
    // L108555
    pub fn amd_vrs4_fabsf(x: __m128) -> __m128;
    // L108566
    pub fn amd_vrd2_fabs(x: __m128d) -> __m128d;
    // L108597
    pub fn amd_vrs8_sinf(x: __m256) -> __m256;
    // L108608
    pub fn amd_vrd4_sin(x: __m256d) -> __m256d;
    // L108621
    pub fn amd_vrs8_cosf(x: __m256) -> __m256;
    // L108632
    pub fn amd_vrd4_cos(x: __m256d) -> __m256d;
    // L108644
    pub fn amd_vrs8_tanf(x: __m256) -> __m256;
    // L108655
    pub fn amd_vrd4_tan(x: __m256d) -> __m256d;
    // L108668
    pub fn amd_vrs8_sincosf(x: __m256, sin: *mut __m256, cos: *mut __m256);
    // L108677
    pub fn amd_vrd4_sincos(x: __m256d, sin: *mut __m256d, cos: *mut __m256d);
    // L108689
    pub fn amd_vrs8_asinf(x: __m256) -> __m256;
    // L108700
    pub fn amd_vrd4_asin(x: __m256d) -> __m256d;
    // L108713
    pub fn amd_vrs8_acosf(x: __m256) -> __m256;
    // L108724
    pub fn amd_vrd4_acos(x: __m256d) -> __m256d;
    // L108736
    pub fn amd_vrs8_atanf(x: __m256) -> __m256;
    // L108747
    pub fn amd_vrd4_atan(x: __m256d) -> __m256d;
    // L108758
    pub fn amd_vrd4_atan2(y: __m256d, x: __m256d) -> __m256d;
    // L108774
    pub fn amd_vrs8_coshf(x: __m256) -> __m256;
    // L108784
    pub fn amd_vrd4_cosh(x: __m256d) -> __m256d;
    // L108797
    pub fn amd_vrs8_tanhf(x: __m256) -> __m256;
    // L108808
    pub fn amd_vrd4_tanh(x: __m256d) -> __m256d;
    // L108825
    pub fn amd_vrs8_expf(x: __m256) -> __m256;
    // L108835
    pub fn amd_vrd4_exp(x: __m256d) -> __m256d;
    // L108848
    pub fn amd_vrs8_exp2f(x: __m256) -> __m256;
    // L108859
    pub fn amd_vrd4_exp2(x: __m256d) -> __m256d;
    // L108872
    pub fn amd_vrs8_exp10f(x: __m256) -> __m256;
    // L108882
    pub fn amd_vrd4_exp10(x: __m256d) -> __m256d;
    // L108897
    pub fn amd_vrs8_logf(x: __m256) -> __m256;
    // L108908
    pub fn amd_vrd4_log(x: __m256d) -> __m256d;
    // L108921
    pub fn amd_vrs8_log2f(x: __m256) -> __m256;
    // L108931
    pub fn amd_vrd4_log2(x: __m256d) -> __m256d;
    // L108944
    pub fn amd_vrs8_log10f(x: __m256) -> __m256;
    // L108959
    pub fn amd_vrs8_powf(x: __m256, y: __m256) -> __m256;
    // L108971
    pub fn amd_vrd4_pow(x: __m256d, y: __m256d) -> __m256d;
    // L108984
    pub fn amd_vrs8_powxf(x: __m256, y: c_float) -> __m256;
    // L108996
    pub fn amd_vrd4_powx(x: __m256d, y: c_double) -> __m256d;
    // L109010
    pub fn amd_vrs8_sqrtf(x: __m256) -> __m256;
    // L109021
    pub fn amd_vrd4_sqrt(x: __m256d) -> __m256d;
    // L109033
    pub fn amd_vrs8_cbrtf(x: __m256) -> __m256;
    // L109048
    pub fn amd_vrs8_erff(x: __m256) -> __m256;
    // L109059
    pub fn amd_vrd4_erf(x: __m256d) -> __m256d;
    // L109072
    pub fn amd_vrs8_erfcf(x: __m256) -> __m256;
    // L109082
    pub fn amd_vrd4_erfc(x: __m256d) -> __m256d;
    // L109095
    pub fn amd_vrd4_erfinv(x: __m256d) -> __m256d;
    // L109108
    pub fn amd_vrd4_erfcinv(x: __m256d) -> __m256d;
    // L109132
    pub fn amd_vrs8_roundf(x: __m256) -> __m256;
    // L109143
    pub fn amd_vrd4_round(x: __m256d) -> __m256d;
    // L109158
    pub fn amd_vrs8_fabsf(x: __m256) -> __m256;
    // L109169
    pub fn amd_vrd4_fabs(x: __m256d) -> __m256d;
    // L109201
    pub fn amd_vrs16_sinf(x: __m512) -> __m512;
    // L109212
    pub fn amd_vrd8_sin(x: __m512d) -> __m512d;
    // L109225
    pub fn amd_vrs16_cosf(x: __m512) -> __m512;
    // L109235
    pub fn amd_vrd8_cos(x: __m512d) -> __m512d;
    // L109248
    pub fn amd_vrs16_tanf(x: __m512) -> __m512;
    // L109259
    pub fn amd_vrd8_tan(x: __m512d) -> __m512d;
    // L109272
    pub fn amd_vrs16_sincosf(x: __m512, sin: *mut __m512, cos: *mut __m512);
    // L109280
    pub fn amd_vrd8_sincos(x: __m512d, sin: *mut __m512d, cos: *mut __m512d);
    // L109293
    pub fn amd_vrs16_asinf(x: __m512) -> __m512;
    // L109304
    pub fn amd_vrd8_asin(x: __m512d) -> __m512d;
    // L109317
    pub fn amd_vrs16_acosf(x: __m512) -> __m512;
    // L109327
    pub fn amd_vrd8_acos(x: __m512d) -> __m512d;
    // L109340
    pub fn amd_vrs16_atanf(x: __m512) -> __m512;
    // L109351
    pub fn amd_vrd8_atan(x: __m512d) -> __m512d;
    // L109362
    pub fn amd_vrd8_atan2(y: __m512d, x: __m512d) -> __m512d;
    // L109377
    pub fn amd_vrs16_coshf(x: __m512) -> __m512;
    // L109388
    pub fn amd_vrd8_cosh(x: __m512d) -> __m512d;
    // L109401
    pub fn amd_vrs16_tanhf(x: __m512) -> __m512;
    // L109412
    pub fn amd_vrd8_tanh(x: __m512d) -> __m512d;
    // L109428
    pub fn amd_vrs16_expf(x: __m512) -> __m512;
    // L109439
    pub fn amd_vrd8_exp(x: __m512d) -> __m512d;
    // L109452
    pub fn amd_vrs16_exp2f(x: __m512) -> __m512;
    // L109463
    pub fn amd_vrd8_exp2(x: __m512d) -> __m512d;
    // L109475
    pub fn amd_vrs16_exp10f(x: __m512) -> __m512;
    // L109486
    pub fn amd_vrd8_exp10(x: __m512d) -> __m512d;
    // L109501
    pub fn amd_vrs16_logf(x: __m512) -> __m512;
    // L109512
    pub fn amd_vrd8_log(x: __m512d) -> __m512d;
    // L109524
    pub fn amd_vrs16_log2f(x: __m512) -> __m512;
    // L109535
    pub fn amd_vrd8_log2(x: __m512d) -> __m512d;
    // L109548
    pub fn amd_vrs16_log10f(x: __m512) -> __m512;
    // L109563
    pub fn amd_vrs16_powf(x: __m512, y: __m512) -> __m512;
    // L109574
    pub fn amd_vrd8_pow(x: __m512d, y: __m512d) -> __m512d;
    // L109588
    pub fn amd_vrs16_powxf(x: __m512, y: c_float) -> __m512;
    // L109600
    pub fn amd_vrd8_powx(x: __m512d, y: c_double) -> __m512d;
    // L109614
    pub fn amd_vrs16_sqrtf(x: __m512) -> __m512;
    // L109624
    pub fn amd_vrd8_sqrt(x: __m512d) -> __m512d;
    // L109639
    pub fn amd_vrs16_erff(x: __m512) -> __m512;
    // L109650
    pub fn amd_vrd8_erf(x: __m512d) -> __m512d;
    // L109663
    pub fn amd_vrs16_erfcf(x: __m512) -> __m512;
    // L109673
    pub fn amd_vrd8_erfc(x: __m512d) -> __m512d;
    // L109686
    pub fn amd_vrd8_erfinv(x: __m512d) -> __m512d;
    // L109699
    pub fn amd_vrd8_erfcinv(x: __m512d) -> __m512d;
    // L109723
    pub fn amd_vrs16_roundf(x: __m512) -> __m512;
    // L109734
    pub fn amd_vrd8_round(x: __m512d) -> __m512d;
    // L109749
    pub fn amd_vrs16_fabsf(x: __m512) -> __m512;
    // L109760
    pub fn amd_vrd8_fabs(x: __m512d) -> __m512d;
    // L109799
    pub fn amd_vrsa_sinf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109808
    pub fn amd_vrda_sin(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109818
    pub fn amd_vrsa_cosf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109827
    pub fn amd_vrda_cos(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109838
    pub fn amd_vrsa_tanf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109847
    pub fn amd_vrda_tan(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109857
    pub fn amd_vrsa_sincosf(len: c_int, src: *const c_float, sin: *mut c_float, cos: *mut c_float);
    // L109867
    pub fn amd_vrda_sincos(len: c_int, src: *const c_double, sin: *mut c_double, cos: *mut c_double);
    // L109881
    pub fn amd_vrsa_asinf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109890
    pub fn amd_vrda_asin(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109900
    pub fn amd_vrsa_acosf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109909
    pub fn amd_vrda_acos(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109920
    pub fn amd_vrsa_atanf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109929
    pub fn amd_vrda_atan(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109941
    pub fn amd_vrsa_coshf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109950
    pub fn amd_vrda_cosh(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109961
    pub fn amd_vrsa_tanhf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109970
    pub fn amd_vrda_tanh(len: c_int, src: *const c_double, dst: *mut c_double);
    // L109982
    pub fn amd_vrsa_expf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L109991
    pub fn amd_vrda_exp(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110002
    pub fn amd_vrsa_exp2f(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110011
    pub fn amd_vrda_exp2(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110021
    pub fn amd_vrsa_exp10f(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110030
    pub fn amd_vrda_exp10(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110041
    pub fn amd_vrsa_expm1f(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110050
    pub fn amd_vrda_expm1(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110062
    pub fn amd_vrsa_logf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110071
    pub fn amd_vrda_log(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110082
    pub fn amd_vrsa_log2f(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110091
    pub fn amd_vrda_log2(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110101
    pub fn amd_vrsa_log10f(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110110
    pub fn amd_vrda_log10(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110121
    pub fn amd_vrsa_log1pf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110130
    pub fn amd_vrda_log1p(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110142
    pub fn amd_vrsa_powf(len: c_int, src1: *const c_float, src2: *const c_float, dst: *mut c_float);
    // L110152
    pub fn amd_vrda_pow(len: c_int, src1: *const c_double, src2: *const c_double, dst: *mut c_double);
    // L110164
    pub fn amd_vrsa_powxf(len: c_int, src: *const c_float, y: c_float, dst: *mut c_float);
    // L110174
    pub fn amd_vrda_powx(len: c_int, src: *const c_double, y: c_double, dst: *mut c_double);
    // L110185
    pub fn amd_vrsa_sqrtf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110194
    pub fn amd_vrda_sqrt(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110205
    pub fn amd_vrsa_cbrtf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110214
    pub fn amd_vrda_cbrt(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110226
    pub fn amd_vrsa_erff(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110235
    pub fn amd_vrda_erf(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110246
    pub fn amd_vrsa_erfcf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110255
    pub fn amd_vrda_erfc(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110265
    pub fn amd_vrda_erfinv(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110276
    pub fn amd_vrda_erfcinv(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110299
    pub fn amd_vrsa_fabsf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110307
    pub fn amd_vrda_fabs(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110320
    pub fn amd_vrsa_roundf(len: c_int, src: *const c_float, dst: *mut c_float);
    // L110329
    pub fn amd_vrda_round(len: c_int, src: *const c_double, dst: *mut c_double);
    // L110342
    pub fn amd_vrsa_fmaxf(len: c_int, lhs: *const c_float, rhs: *const c_float, dst: *mut c_float);
    // L110351
    pub fn amd_vrda_fmax(len: c_int, lhs: *const c_double, rhs: *const c_double, dst: *mut c_double);
    // L110363
    pub fn amd_vrsa_fminf(len: c_int, lhs: *const c_float, rhs: *const c_float, dst: *mut c_float);
    // L110373
    pub fn amd_vrda_fmin(len: c_int, lhs: *const c_double, rhs: *const c_double, dst: *mut c_double);
    // L110387
    pub fn amd_vrsa_fmaxfi(len: c_int, lhs: *const c_float, inc_a: c_int, rhs: *const c_float, inc_b: c_int, dst: *mut c_float, inc_res: c_int);
    // L110401
    pub fn amd_vrda_fmaxi(len: c_int, lhs: *const c_double, inc_a: c_int, rhs: *const c_double, inc_b: c_int, dst: *mut c_double, inc_res: c_int);
    // L110417
    pub fn amd_vrsa_fminfi(len: c_int, lhs: *const c_float, inc_a: c_int, rhs: *const c_float, inc_b: c_int, dst: *mut c_float, inc_res: c_int);
    // L110431
    pub fn amd_vrda_fmini(len: c_int, lhs: *const c_double, inc_a: c_int, rhs: *const c_double, inc_b: c_int, dst: *mut c_double, inc_res: c_int);
    // L111081
    pub fn amd_cdfnorm(x: c_double) -> c_double;
    // L111094
    pub fn amd_vrd2_cdfnorm(x: __m128d) -> __m128d;
    // L111106
    pub fn amd_vrd4_cdfnorm(x: __m256d) -> __m256d;
    // L111118
    pub fn amd_vrd8_cdfnorm(x: __m512d) -> __m512d;
    // L111130
    pub fn amd_vrda_cdfnorm(len: c_int, src: *const c_double, dst: *mut c_double);
    // L111159
    pub fn amd_cdfnorminv(x: c_double) -> c_double;
    // L111173
    pub fn amd_vrd2_cdfnorminv(x: __m128d) -> __m128d;
    // L111185
    pub fn amd_vrd4_cdfnorminv(x: __m256d) -> __m256d;
    // L111197
    pub fn amd_vrd8_cdfnorminv(x: __m512d) -> __m512d;
    // L111209
    pub fn amd_vrda_cdfnorminv(len: c_int, src: *const c_double, dst: *mut c_double);
    // L111383
    pub fn amd_vrda_add(len: c_int, lhs: *const c_double, rhs: *const c_double, dst: *mut c_double);
    // L111393
    pub fn amd_vrsa_addf(len: c_int, lhs: *const c_float, rhs: *const c_float, dst: *mut c_float);
    // L111404
    pub fn amd_vrda_addi(len: c_int, lhs: *const c_double, inc_a: c_int, rhs: *const c_double, inc_b: c_int, dst: *mut c_double, inc_res: c_int);
    // L111418
    pub fn amd_vrsa_addfi(len: c_int, lhs: *const c_float, inc_a: c_int, rhs: *const c_float, inc_b: c_int, dst: *mut c_float, inc_res: c_int);
    // L111435
    pub fn amd_vrda_sub(len: c_int, lhs: *const c_double, rhs: *const c_double, dst: *mut c_double);
    // L111445
    pub fn amd_vrsa_subf(len: c_int, lhs: *const c_float, rhs: *const c_float, dst: *mut c_float);
    // L111456
    pub fn amd_vrda_subi(len: c_int, lhs: *const c_double, inc_a: c_int, rhs: *const c_double, inc_b: c_int, dst: *mut c_double, inc_res: c_int);
    // L111470
    pub fn amd_vrsa_subfi(len: c_int, lhs: *const c_float, inc_a: c_int, rhs: *const c_float, inc_b: c_int, dst: *mut c_float, inc_res: c_int);
    // L111487
    pub fn amd_vrda_mul(len: c_int, lhs: *const c_double, rhs: *const c_double, dst: *mut c_double);
    // L111496
    pub fn amd_vrsa_mulf(len: c_int, lhs: *const c_float, rhs: *const c_float, dst: *mut c_float);
    // L111507
    pub fn amd_vrda_muli(len: c_int, lhs: *const c_double, inc_a: c_int, rhs: *const c_double, inc_b: c_int, dst: *mut c_double, inc_res: c_int);
    // L111521
    pub fn amd_vrsa_mulfi(len: c_int, lhs: *const c_float, inc_a: c_int, rhs: *const c_float, inc_b: c_int, dst: *mut c_float, inc_res: c_int);
    // L111538
    pub fn amd_vrda_div(len: c_int, lhs: *const c_double, rhs: *const c_double, dst: *mut c_double);
    // L111548
    pub fn amd_vrsa_divf(len: c_int, lhs: *const c_float, rhs: *const c_float, dst: *mut c_float);
    // L111559
    pub fn amd_vrda_divi(len: c_int, lhs: *const c_double, inc_a: c_int, rhs: *const c_double, inc_b: c_int, dst: *mut c_double, inc_res: c_int);
    // L111573
    pub fn amd_vrsa_divfi(len: c_int, lhs: *const c_float, inc_a: c_int, rhs: *const c_float, inc_b: c_int, dst: *mut c_float, inc_res: c_int);
    // L111590
    pub fn amd_vrd2_linearfrac(x: __m128d, y: __m128d, scx: c_double, shx: c_double, scy: c_double, shy: c_double) -> __m128d;
    // L111606
    pub fn amd_vrs4_linearfracf(x: __m128, y: __m128, scx: c_float, shx: c_float, scy: c_float, shy: c_float) -> __m128;
    // L111623
    pub fn amd_vrd4_linearfrac(x: __m256d, y: __m256d, scx: c_double, shx: c_double, scy: c_double, shy: c_double) -> __m256d;
    // L111639
    pub fn amd_vrs8_linearfracf(x: __m256, y: __m256, scx: c_float, shx: c_float, scy: c_float, shy: c_float) -> __m256;
    // L111656
    pub fn amd_vrd8_linearfrac(x: __m512d, y: __m512d, scx: c_double, shx: c_double, scy: c_double, shy: c_double) -> __m512d;
    // L111672
    pub fn amd_vrs16_linearfracf(x: __m512, y: __m512, scx: c_float, shx: c_float, scy: c_float, shy: c_float) -> __m512;
    // L111689
    pub fn amd_vrda_linearfrac(len: c_int, x: *const c_double, y: *const c_double, scx: c_double, shx: c_double, scy: c_double, shy: c_double, dst: *mut c_double);
    // L111704
    pub fn amd_vrsa_linearfracf(len: c_int, x: *const c_float, y: *const c_float, scx: c_float, shx: c_float, scy: c_float, shy: c_float, dst: *mut c_float);
    // L111727
    pub fn amd_cexpf(y: fc32_t) -> fc32_t;
    // L111738
    pub fn amd_cexp(x: fc64_t) -> fc64_t;
    // L111750
    pub fn amd_clogf(y: fc32_t) -> fc32_t;
    // L111761
    pub fn amd_clog(x: fc64_t) -> fc64_t;
    // L111774
    pub fn amd_cpowf(x: fc32_t, y: fc32_t) -> fc32_t;
    // L111786
    pub fn amd_cpow(x: fc64_t, y: fc64_t) -> fc64_t;
    // L111856
    pub fn amd_stablesort_getsize_64f(len: c_int, workspace_size: *mut c_int) -> c_int;
    // L111876
    pub fn amd_stablesort_ascend_64f(src: *const c_double, src_stride_bytes: c_int, dst_index: *mut c_int, len: c_int, workspace: *mut c_void) -> c_int;
}
