// GENERATED mechanically (no hand edits) from docs/cleanroom/math_decls.rs: the by-value SIMD functions (guide 6.0 ch. 8).
// Needs nightly `simd_ffi` (feature `simd_ffi`); never in the default build. `// L<n>` is the guide line.
use core::arch::x86_64::{__m128, __m128d, __m256, __m256d, __m512, __m512d};
use core::ffi::{c_double, c_float, c_int};

extern "C" {
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
    // L111094
    pub fn amd_vrd2_cdfnorm(x: __m128d) -> __m128d;
    // L111106
    pub fn amd_vrd4_cdfnorm(x: __m256d) -> __m256d;
    // L111118
    pub fn amd_vrd8_cdfnorm(x: __m512d) -> __m512d;
    // L111173
    pub fn amd_vrd2_cdfnorminv(x: __m128d) -> __m128d;
    // L111185
    pub fn amd_vrd4_cdfnorminv(x: __m256d) -> __m256d;
    // L111197
    pub fn amd_vrd8_cdfnorminv(x: __m512d) -> __m512d;
    // L111590
    pub fn amd_vrd2_linearfrac(
        x: __m128d,
        y: __m128d,
        scx: c_double,
        shx: c_double,
        scy: c_double,
        shy: c_double,
    ) -> __m128d;
    // L111606
    pub fn amd_vrs4_linearfracf(
        x: __m128,
        y: __m128,
        scx: c_float,
        shx: c_float,
        scy: c_float,
        shy: c_float,
    ) -> __m128;
    // L111623
    pub fn amd_vrd4_linearfrac(
        x: __m256d,
        y: __m256d,
        scx: c_double,
        shx: c_double,
        scy: c_double,
        shy: c_double,
    ) -> __m256d;
    // L111639
    pub fn amd_vrs8_linearfracf(
        x: __m256,
        y: __m256,
        scx: c_float,
        shx: c_float,
        scy: c_float,
        shy: c_float,
    ) -> __m256;
    // L111656
    pub fn amd_vrd8_linearfrac(
        x: __m512d,
        y: __m512d,
        scx: c_double,
        shx: c_double,
        scy: c_double,
        shy: c_double,
    ) -> __m512d;
    // L111672
    pub fn amd_vrs16_linearfracf(
        x: __m512,
        y: __m512,
        scx: c_float,
        shx: c_float,
        scy: c_float,
        shy: c_float,
    ) -> __m512;
}
