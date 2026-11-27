use crate::{rng_int_t, RNG_DRANDINITIALIZEUSER_UGEN, RNG_DRANDINITIALIZEUSER_UINI};

// UPPERCASE forms (all arguments by pointer except where the guide says otherwise). Comment line numbers are
// lines of guide chapter 9 as extracted (offset +111896 gives the line of guide-6.0.txt).
extern "C" {
    // guide-6.0.txt chapter-9 file line: L134
    pub fn SRANDINITIALIZE(
        genid: *mut rng_int_t,
        subid: *mut rng_int_t,
        seed: *mut rng_int_t,
        lseed: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L136
    pub fn DRANDINITIALIZE(
        genid: *mut rng_int_t,
        subid: *mut rng_int_t,
        seed: *mut rng_int_t,
        lseed: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L305
    /// **Unsafe; undocumented protocol.** Requires the `uini` callback to set `state[0] = n` and `lstate >= n + 3`; the guide does not state
    /// this. Measured on AOCL 6.0.68 Windows (see `docs/cleanroom/types/userinit/USERINIT-MEASUREMENT.md`). The library does not validate `n`
    /// against `lstate`, so a `uini` that does not follow it corrupts memory. No safe wrapper exists and no test calls this.
    pub fn DRANDINITIALIZEUSER(
        uini: RNG_DRANDINITIALIZEUSER_UINI,
        ugen: RNG_DRANDINITIALIZEUSER_UGEN,
        genid: rng_int_t,
        subid: rng_int_t,
        seed: *mut rng_int_t,
        lseed: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L332
    pub fn SRANDINITIALIZEBBS(
        nbits: *mut rng_int_t,
        lp: *mut rng_int_t,
        p: *mut rng_int_t,
        lq: *mut rng_int_t,
        q: *mut rng_int_t,
        ls: *mut rng_int_t,
        s: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L335
    pub fn DRANDINITIALIZEBBS(
        nbits: *mut rng_int_t,
        lp: *mut rng_int_t,
        p: *mut rng_int_t,
        lq: *mut rng_int_t,
        q: *mut rng_int_t,
        ls: *mut rng_int_t,
        s: *mut rng_int_t,
        state: *mut rng_int_t,
        lstate: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L448
    pub fn SRANDBLUMBLUMSHUB(
        n: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L449
    pub fn DRANDBLUMBLUMSHUB(
        n: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L512
    pub fn SRANDBETA(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        b: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L513
    pub fn DRANDBETA(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        b: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L543
    pub fn SRANDCAUCHY(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        b: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L544
    pub fn DRANDCAUCHY(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        b: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L574
    pub fn SRANDCHISQUARED(
        n: *mut rng_int_t,
        df: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L575
    pub fn DRANDCHISQUARED(
        n: *mut rng_int_t,
        df: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L602
    pub fn SRANDEXPONENTIAL(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L603
    pub fn DRANDEXPONENTIAL(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L632
    pub fn SRANDF(
        n: *mut rng_int_t,
        df1: *mut rng_int_t,
        df2: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L635
    pub fn DRANDF(
        n: *mut rng_int_t,
        df1: *mut rng_int_t,
        df2: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L671
    pub fn SRANDGAMMA(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        b: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L672
    pub fn DRANDGAMMA(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        b: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L704
    pub fn SRANDGAUSSIAN(
        n: *mut rng_int_t,
        xmu: *mut core::ffi::c_float,
        var: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L706
    pub fn DRANDGAUSSIAN(
        n: *mut rng_int_t,
        xmu: *mut core::ffi::c_double,
        var: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L741
    pub fn SRANDLOGISTIC(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        b: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L742
    pub fn DRANDLOGISTIC(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        b: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L774
    pub fn SRANDLOGNORMAL(
        n: *mut rng_int_t,
        xmu: *mut core::ffi::c_float,
        var: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L776
    pub fn DRANDLOGNORMAL(
        n: *mut rng_int_t,
        xmu: *mut core::ffi::c_double,
        var: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L809
    pub fn SRANDSTUDENTST(
        n: *mut rng_int_t,
        df: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L810
    pub fn DRANDSTUDENTST(
        n: *mut rng_int_t,
        df: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L836
    pub fn SRANDTRIANGULAR(
        n: *mut rng_int_t,
        xmin: *mut core::ffi::c_float,
        xmed: *mut core::ffi::c_float,
        xmax: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L839
    pub fn DRANDTRIANGULAR(
        n: *mut rng_int_t,
        xmin: *mut core::ffi::c_double,
        xmed: *mut core::ffi::c_double,
        xmax: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L877
    pub fn SRANDUNIFORM(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        b: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L878
    pub fn DRANDUNIFORM(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        b: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L908
    pub fn SRANDVONMISES(
        n: *mut rng_int_t,
        vk: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L909
    pub fn DRANDVONMISES(
        n: *mut rng_int_t,
        vk: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L935
    pub fn SRANDWEIBULL(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_float,
        b: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L937
    pub fn DRANDWEIBULL(
        n: *mut rng_int_t,
        a: *mut core::ffi::c_double,
        b: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L977
    pub fn SRANDMULTINORMAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        xmu: *mut core::ffi::c_float,
        c: *mut core::ffi::c_float,
        ldc: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L980
    pub fn DRANDMULTINORMAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        xmu: *mut core::ffi::c_double,
        c: *mut core::ffi::c_double,
        ldc: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1029
    pub fn SRANDMULTISTUDENTST(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        df: *mut rng_int_t,
        xmu: *mut core::ffi::c_float,
        c: *mut core::ffi::c_float,
        ldc: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1031
    pub fn DRANDMULTISTUDENTST(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        df: *mut rng_int_t,
        xmu: *mut core::ffi::c_double,
        c: *mut core::ffi::c_double,
        ldc: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1075
    pub fn SRANDMULTINORMALREFERENCE(
        m: *mut rng_int_t,
        xmu: *mut core::ffi::c_float,
        c: *mut core::ffi::c_float,
        ldc: *mut rng_int_t,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1077
    pub fn DRANDMULTINORMALREFERENCE(
        m: *mut rng_int_t,
        xmu: *mut core::ffi::c_double,
        c: *mut core::ffi::c_double,
        ldc: *mut rng_int_t,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1116
    pub fn SRANDMULTINORMALR(
        n: *mut rng_int_t,
        r#ref: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1118
    pub fn DRANDMULTINORMALR(
        n: *mut rng_int_t,
        r#ref: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1156
    pub fn SRANDMULTISTUDENTSTREFERENCE(
        m: *mut rng_int_t,
        df: *mut rng_int_t,
        xmu: *mut core::ffi::c_float,
        c: *mut core::ffi::c_float,
        ldc: *mut rng_int_t,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1159
    pub fn DRANDMULTISTUDENTSTREFERENCE(
        m: *mut rng_int_t,
        df: *mut rng_int_t,
        xmu: *mut core::ffi::c_double,
        c: *mut core::ffi::c_double,
        ldc: *mut rng_int_t,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1204
    pub fn SRANDMULTISTUDENTSTR(
        n: *mut rng_int_t,
        r#ref: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_float,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1206
    pub fn DRANDMULTISTUDENTSTR(
        n: *mut rng_int_t,
        r#ref: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut core::ffi::c_double,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1275
    pub fn SRANDBINOMIAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        p: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1277
    pub fn DRANDBINOMIAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        p: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1310
    pub fn SRANDGEOMETRIC(
        n: *mut rng_int_t,
        p: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1311
    pub fn DRANDGEOMETRIC(
        n: *mut rng_int_t,
        p: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1337
    pub fn SRANDHYPERGEOMETRIC(
        n: *mut rng_int_t,
        np: *mut rng_int_t,
        ns: *mut rng_int_t,
        m: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1339
    pub fn DRANDHYPERGEOMETRIC(
        n: *mut rng_int_t,
        np: *mut rng_int_t,
        ns: *mut rng_int_t,
        m: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1379
    pub fn SRANDNEGATIVEBINOMIAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        p: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1381
    pub fn DRANDNEGATIVEBINOMIAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        p: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1414
    pub fn SRANDPOISSON(
        n: *mut rng_int_t,
        lambda: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1415
    pub fn DRANDPOISSON(
        n: *mut rng_int_t,
        lambda: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1443
    pub fn SRANDDISCRETEUNIFORM(
        n: *mut rng_int_t,
        a: *mut rng_int_t,
        b: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1446
    pub fn DRANDDISCRETEUNIFORM(
        n: *mut rng_int_t,
        a: *mut rng_int_t,
        b: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1488
    pub fn SRANDGENERALDISCRETE(
        n: *mut rng_int_t,
        r#ref: *mut core::ffi::c_float,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1491
    pub fn DRANDGENERALDISCRETE(
        n: *mut rng_int_t,
        r#ref: *mut core::ffi::c_double,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1535
    pub fn SRANDBINOMIALREFERENCE(
        m: *mut rng_int_t,
        p: *mut core::ffi::c_float,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1538
    pub fn DRANDBINOMIALREFERENCE(
        m: *mut rng_int_t,
        p: *mut core::ffi::c_double,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1575
    pub fn SRANDGEOMETRICREFERENCE(
        p: *mut core::ffi::c_float,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1576
    pub fn DRANDGEOMETRICREFERENCE(
        p: *mut core::ffi::c_double,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1605
    pub fn SRANDHYPERGEOMETRICREFERENCE(
        np: *mut rng_int_t,
        ns: *mut rng_int_t,
        m: *mut rng_int_t,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1608
    pub fn DRANDHYPERGEOMETRICREFERENCE(
        np: *mut rng_int_t,
        ns: *mut rng_int_t,
        m: *mut rng_int_t,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1648
    pub fn SRANDNEGATIVEBINOMIALREFERENCE(
        m: *mut rng_int_t,
        p: *mut core::ffi::c_float,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1651
    pub fn DRANDNEGATIVEBINOMIALREFERENCE(
        m: *mut rng_int_t,
        p: *mut core::ffi::c_double,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1699
    pub fn SRANDPOISSONREFERENCE(
        lambda: *mut core::ffi::c_float,
        r#ref: *mut core::ffi::c_float,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1700
    pub fn DRANDPOISSONREFERENCE(
        lambda: *mut core::ffi::c_double,
        r#ref: *mut core::ffi::c_double,
        lref: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1734
    pub fn SRANDMULTINOMIAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        p: *mut core::ffi::c_float,
        k: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1736
    pub fn DRANDMULTINOMIAL(
        n: *mut rng_int_t,
        m: *mut rng_int_t,
        p: *mut core::ffi::c_double,
        k: *mut rng_int_t,
        state: *mut rng_int_t,
        x: *mut rng_int_t,
        ldx: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1786
    pub fn SRANDSKIPAHEAD(n: *mut rng_int_t, state: *mut rng_int_t, info: *mut rng_int_t);
    // guide-6.0.txt chapter-9 file line: L1787
    pub fn DRANDSKIPAHEAD(n: *mut rng_int_t, state: *mut rng_int_t, info: *mut rng_int_t);
    // guide-6.0.txt chapter-9 file line: L1817
    pub fn SRANDLEAPFROG(
        n: *mut rng_int_t,
        k: *mut rng_int_t,
        state: *mut rng_int_t,
        info: *mut rng_int_t,
    );
    // guide-6.0.txt chapter-9 file line: L1818
    pub fn DRANDLEAPFROG(
        n: *mut rng_int_t,
        k: *mut rng_int_t,
        state: *mut rng_int_t,
        info: *mut rng_int_t,
    );
}
