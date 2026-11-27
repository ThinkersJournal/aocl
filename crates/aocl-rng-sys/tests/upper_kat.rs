//! UPPERCASE forms: every declared symbol resolves in the installed DLL (taking the address forces the link), and the
//! uppercase init/uniform pair reproduces the lowercase known answer (all arguments by pointer).
#![allow(warnings)]
use aocl_rng_sys::*;

#[test]
fn all_uppercase_symbols_link() {
    let addrs: Vec<usize> = vec![
        SRANDINITIALIZE as usize,
        DRANDINITIALIZE as usize,
        DRANDINITIALIZEUSER as usize,
        SRANDINITIALIZEUSER as usize,
        SRANDINITIALIZEBBS as usize,
        DRANDINITIALIZEBBS as usize,
        SRANDBLUMBLUMSHUB as usize,
        DRANDBLUMBLUMSHUB as usize,
        SRANDBETA as usize,
        DRANDBETA as usize,
        SRANDCAUCHY as usize,
        DRANDCAUCHY as usize,
        SRANDCHISQUARED as usize,
        DRANDCHISQUARED as usize,
        SRANDEXPONENTIAL as usize,
        DRANDEXPONENTIAL as usize,
        SRANDF as usize,
        DRANDF as usize,
        SRANDGAMMA as usize,
        DRANDGAMMA as usize,
        SRANDGAUSSIAN as usize,
        DRANDGAUSSIAN as usize,
        SRANDLOGISTIC as usize,
        DRANDLOGISTIC as usize,
        SRANDLOGNORMAL as usize,
        DRANDLOGNORMAL as usize,
        SRANDSTUDENTST as usize,
        DRANDSTUDENTST as usize,
        SRANDTRIANGULAR as usize,
        DRANDTRIANGULAR as usize,
        SRANDUNIFORM as usize,
        DRANDUNIFORM as usize,
        SRANDVONMISES as usize,
        DRANDVONMISES as usize,
        SRANDWEIBULL as usize,
        DRANDWEIBULL as usize,
        SRANDMULTINORMAL as usize,
        DRANDMULTINORMAL as usize,
        SRANDMULTISTUDENTST as usize,
        DRANDMULTISTUDENTST as usize,
        SRANDMULTINORMALREFERENCE as usize,
        DRANDMULTINORMALREFERENCE as usize,
        SRANDMULTINORMALR as usize,
        DRANDMULTINORMALR as usize,
        SRANDMULTISTUDENTSTREFERENCE as usize,
        DRANDMULTISTUDENTSTREFERENCE as usize,
        SRANDMULTISTUDENTSTR as usize,
        DRANDMULTISTUDENTSTR as usize,
        SRANDBINOMIAL as usize,
        DRANDBINOMIAL as usize,
        SRANDGEOMETRIC as usize,
        DRANDGEOMETRIC as usize,
        SRANDHYPERGEOMETRIC as usize,
        DRANDHYPERGEOMETRIC as usize,
        SRANDNEGATIVEBINOMIAL as usize,
        DRANDNEGATIVEBINOMIAL as usize,
        SRANDPOISSON as usize,
        DRANDPOISSON as usize,
        SRANDDISCRETEUNIFORM as usize,
        DRANDDISCRETEUNIFORM as usize,
        SRANDGENERALDISCRETE as usize,
        DRANDGENERALDISCRETE as usize,
        SRANDBINOMIALREFERENCE as usize,
        DRANDBINOMIALREFERENCE as usize,
        SRANDGEOMETRICREFERENCE as usize,
        DRANDGEOMETRICREFERENCE as usize,
        SRANDHYPERGEOMETRICREFERENCE as usize,
        DRANDHYPERGEOMETRICREFERENCE as usize,
        SRANDNEGATIVEBINOMIALREFERENCE as usize,
        DRANDNEGATIVEBINOMIALREFERENCE as usize,
        SRANDPOISSONREFERENCE as usize,
        DRANDPOISSONREFERENCE as usize,
        SRANDMULTINOMIAL as usize,
        DRANDMULTINOMIAL as usize,
        SRANDSKIPAHEAD as usize,
        DRANDSKIPAHEAD as usize,
        SRANDLEAPFROG as usize,
        DRANDLEAPFROG as usize,
    ];
    assert_eq!(addrs.len(), 78);
    assert!(addrs.iter().all(|a| *a != 0));
}

#[test]
fn uppercase_known_answer() {
    let mut genid: rng_int_t = 1;
    let mut subid: rng_int_t = 0;
    let mut seed: [rng_int_t; 1] = [12345];
    let mut lseed: rng_int_t = 1;
    let mut state: [rng_int_t; 633] = [0; 633];
    let mut lstate: rng_int_t = 633;
    let mut info: rng_int_t = -99;
    let mut n: rng_int_t = 3;
    let (mut a, mut b) = (0f64, 1f64);
    let mut x = [0f64; 3];
    unsafe {
        DRANDINITIALIZE(
            &mut genid,
            &mut subid,
            seed.as_mut_ptr(),
            &mut lseed,
            state.as_mut_ptr(),
            &mut lstate,
            &mut info,
        );
        assert_eq!(info, 0, "init info");
        DRANDUNIFORM(
            &mut n,
            &mut a,
            &mut b,
            state.as_mut_ptr(),
            x.as_mut_ptr(),
            &mut info,
        );
        assert_eq!(info, 0, "uniform info");
    }
    assert_eq!(x[0].to_bits(), 0x3fef20e1d46d4343);
}
