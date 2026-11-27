extern "C" {
    // input-rng-guide6.txt:L112040
    pub fn srandinitialize(genid: rng_int_t, subid: rng_int_t, seed: *mut rng_int_t, lseed: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112043
    pub fn drandinitialize(genid: rng_int_t, subid: rng_int_t, seed: *mut rng_int_t, lseed: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112211
    pub fn srandinitializeuser(uini: RNG_SRANDINITIALIZEUSER_UINI, ugen: RNG_SRANDINITIALIZEUSER_UGEN, genid: rng_int_t, subid: rng_int_t, seed: *mut rng_int_t, lseed: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112215
    pub fn drandinitializeuser(uini: RNG_DRANDINITIALIZEUSER_UINI, ugen: RNG_DRANDINITIALIZEUSER_UGEN, genid: rng_int_t, subid: rng_int_t, seed: *mut rng_int_t, lseed: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112240
    pub fn srandinitializebbs(nbits: rng_int_t, lp: rng_int_t, p: *mut rng_int_t, lq: rng_int_t, q: *mut rng_int_t, ls: rng_int_t, s: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112243
    pub fn drandinitializebbs(nbits: rng_int_t, lp: rng_int_t, p: *mut rng_int_t, lq: rng_int_t, q: *mut rng_int_t, ls: rng_int_t, s: *mut rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112348
    pub fn srandblumblumshub(n: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112349
    pub fn drandblumblumshub(n: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112414
    pub fn srandbeta(n: rng_int_t, a: core::ffi::c_float, b: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112415
    pub fn drandbeta(n: rng_int_t, a: core::ffi::c_double, b: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112445
    pub fn srandcauchy(n: rng_int_t, a: core::ffi::c_float, b: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112446
    pub fn drandcauchy(n: rng_int_t, a: core::ffi::c_double, b: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112573
    pub fn srandgamma(n: rng_int_t, a: core::ffi::c_float, b: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112574
    pub fn drandgamma(n: rng_int_t, a: core::ffi::c_double, b: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112643
    pub fn srandlogistic(n: rng_int_t, a: core::ffi::c_float, b: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112644
    pub fn drandlogistic(n: rng_int_t, a: core::ffi::c_double, b: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112841
    pub fn srandweibull(n: rng_int_t, a: core::ffi::c_float, b: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112843
    pub fn drandweibull(n: rng_int_t, a: core::ffi::c_double, b: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112779
    pub fn sranduniform(n: rng_int_t, a: core::ffi::c_float, b: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112780
    pub fn dranduniform(n: rng_int_t, a: core::ffi::c_double, b: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112475
    pub fn srandchisquared(n: rng_int_t, df: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112476
    pub fn drandchisquared(n: rng_int_t, df: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112502
    pub fn srandexponential(n: rng_int_t, a: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112503
    pub fn drandexponential(n: rng_int_t, a: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112540
    pub fn srandf(n: rng_int_t, df1: rng_int_t, df2: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112542
    pub fn drandf(n: rng_int_t, df1: rng_int_t, df2: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112608
    pub fn srandgaussian(n: rng_int_t, xmu: core::ffi::c_float, var: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112609
    pub fn drandgaussian(n: rng_int_t, xmu: core::ffi::c_double, var: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112678
    pub fn srandlognormal(n: rng_int_t, xmu: core::ffi::c_float, var: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112679
    pub fn drandlognormal(n: rng_int_t, xmu: core::ffi::c_double, var: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112709
    pub fn srandstudentst(n: rng_int_t, df: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112710
    pub fn drandstudentst(n: rng_int_t, df: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112744
    pub fn srandtriangular(n: rng_int_t, xmin: core::ffi::c_float, xmed: core::ffi::c_float, xmax: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112747
    pub fn drandtriangular(n: rng_int_t, xmin: core::ffi::c_double, xmed: core::ffi::c_double, xmax: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112808
    pub fn srandvonmises(n: rng_int_t, vk: core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112809
    pub fn drandvonmises(n: rng_int_t, vk: core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112884
    pub fn srandmultinormal(n: rng_int_t, m: rng_int_t, xmu: *mut core::ffi::c_float, c: *mut core::ffi::c_float, ldc: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_float, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112886
    pub fn drandmultinormal(n: rng_int_t, m: rng_int_t, xmu: *mut core::ffi::c_double, c: *mut core::ffi::c_double, ldc: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_double, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112933
    pub fn srandmultistudentst(n: rng_int_t, m: rng_int_t, df: rng_int_t, xmu: *mut core::ffi::c_float, c: *mut core::ffi::c_float, ldc: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_float, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112935
    pub fn drandmultistudentst(n: rng_int_t, m: rng_int_t, df: rng_int_t, xmu: *mut core::ffi::c_double, c: *mut core::ffi::c_double, ldc: rng_int_t, state: *mut rng_int_t, x: *mut core::ffi::c_double, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112979
    pub fn srandmultinormalreference(m: rng_int_t, xmu: *mut core::ffi::c_float, c: *mut core::ffi::c_float, ldc: rng_int_t, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112981
    pub fn drandmultinormalreference(m: rng_int_t, xmu: *mut core::ffi::c_double, c: *mut core::ffi::c_double, ldc: rng_int_t, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113020
    pub fn srandmultinormalr(n: rng_int_t, r#ref: *mut core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113022
    pub fn drandmultinormalr(n: rng_int_t, r#ref: *mut core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113064
    pub fn srandmultistudentstreference(m: rng_int_t, df: rng_int_t, xmu: *mut core::ffi::c_float, c: *mut core::ffi::c_float, ldc: rng_int_t, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113067
    pub fn drandmultistudentstreference(m: rng_int_t, df: rng_int_t, xmu: *mut core::ffi::c_double, c: *mut core::ffi::c_double, ldc: rng_int_t, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113108
    pub fn srandmultistudentstr(n: rng_int_t, r#ref: *mut core::ffi::c_float, state: *mut rng_int_t, x: *mut core::ffi::c_float, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113110
    pub fn drandmultistudentstr(n: rng_int_t, r#ref: *mut core::ffi::c_double, state: *mut rng_int_t, x: *mut core::ffi::c_double, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113179
    pub fn srandbinomial(n: rng_int_t, m: rng_int_t, p: core::ffi::c_float, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113181
    pub fn drandbinomial(n: rng_int_t, m: rng_int_t, p: core::ffi::c_double, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113283
    pub fn srandnegativebinomial(n: rng_int_t, m: rng_int_t, p: core::ffi::c_float, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113285
    pub fn drandnegativebinomial(n: rng_int_t, m: rng_int_t, p: core::ffi::c_double, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113210
    pub fn srandgeometric(n: rng_int_t, p: core::ffi::c_float, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113211
    pub fn drandgeometric(n: rng_int_t, p: core::ffi::c_double, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113241
    pub fn srandhypergeometric(n: rng_int_t, np: rng_int_t, ns: rng_int_t, m: rng_int_t, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113243
    pub fn drandhypergeometric(n: rng_int_t, np: rng_int_t, ns: rng_int_t, m: rng_int_t, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113316
    pub fn srandpoisson(n: rng_int_t, lambda: core::ffi::c_float, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113317
    pub fn drandpoisson(n: rng_int_t, lambda: core::ffi::c_double, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113351
    pub fn sranddiscreteuniform(n: rng_int_t, a: rng_int_t, b: rng_int_t, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113354
    pub fn dranddiscreteuniform(n: rng_int_t, a: rng_int_t, b: rng_int_t, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113396
    pub fn srandgeneraldiscrete(n: rng_int_t, r#ref: *mut core::ffi::c_float, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113398
    pub fn drandgeneraldiscrete(n: rng_int_t, r#ref: *mut core::ffi::c_double, state: *mut rng_int_t, x: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113442
    pub fn srandbinomialreference(m: rng_int_t, p: core::ffi::c_float, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113444
    pub fn drandbinomialreference(m: rng_int_t, p: core::ffi::c_double, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113556
    pub fn srandnegativebinomialreference(m: rng_int_t, p: core::ffi::c_float, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113559
    pub fn drandnegativebinomialreference(m: rng_int_t, p: core::ffi::c_double, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113475
    pub fn srandgeometricreference(p: core::ffi::c_float, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113476
    pub fn drandgeometricreference(p: core::ffi::c_double, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113513
    pub fn srandhypergeometricreference(np: rng_int_t, ns: rng_int_t, m: rng_int_t, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113516
    pub fn drandhypergeometricreference(np: rng_int_t, ns: rng_int_t, m: rng_int_t, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113599
    pub fn srandpoissonreference(lambda: core::ffi::c_float, r#ref: *mut core::ffi::c_float, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113600
    pub fn drandpoissonreference(lambda: core::ffi::c_double, r#ref: *mut core::ffi::c_double, lref: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113638
    pub fn srandmultinomial(n: rng_int_t, m: rng_int_t, p: *mut core::ffi::c_float, k: rng_int_t, state: *mut rng_int_t, x: *mut rng_int_t, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113640
    pub fn drandmultinomial(n: rng_int_t, m: rng_int_t, p: *mut core::ffi::c_double, k: rng_int_t, state: *mut rng_int_t, x: *mut rng_int_t, ldx: rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113686
    pub fn srandskipahead(n: rng_int_t, state: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113687
    pub fn drandskipahead(n: rng_int_t, state: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113717
    pub fn srandleapfrog(n: rng_int_t, k: rng_int_t, state: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113718
    pub fn drandleapfrog(n: rng_int_t, k: rng_int_t, state: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L112152
    pub fn drandinitialize_secrng(genid: rng_int_t, subid: rng_int_t, state: *mut rng_int_t, lstate: *mut rng_int_t, info: *mut rng_int_t);
    // input-rng-guide6.txt:L113758
    pub fn get_rngversion() -> *const core::ffi::c_char;
}
