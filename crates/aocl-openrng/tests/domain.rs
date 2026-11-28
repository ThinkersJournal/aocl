//! RECORDS (asserts nothing about the library) what each distribution does with an out-of-domain parameter, so the crate's claim
//! "the library refuses with VSL_ERROR_BADARGS" is read from output, not assumed. Compiled only with OPENRNG_LIB_DIR; PM-run.
#![cfg(openrng_linked)]
use aocl_openrng::*;

fn s() -> Stream {
    Stream::new(Brng::Philox4x32x10, 1).unwrap()
}

macro_rules! rec {
    ($label:expr, $e:expr) => {{
        let r = $e;
        println!("DOMAIN {:55} -> {:?}", $label, r);
    }};
}

#[test]
fn record_domain_behaviour() {
    let mut d = [0f64; 8];
    let mut i = [0i32; 8];
    rec!(
        "uniform a > b",
        s().uniform(&mut d, 2.0, 1.0, Accuracy::Standard)
    );
    rec!(
        "uniform NaN",
        s().uniform(&mut d, f64::NAN, 1.0, Accuracy::Standard)
    );
    rec!(
        "gaussian sigma < 0",
        s().gaussian(&mut d, 0.0, -1.0, GaussianMethod::BoxMuller)
    );
    rec!(
        "gaussian sigma = 0",
        s().gaussian(&mut d, 0.0, 0.0, GaussianMethod::BoxMuller)
    );
    rec!(
        "exponential beta <= 0",
        s().exponential(&mut d, 0.0, 0.0, Accuracy::Standard)
    );
    rec!(
        "gamma alpha <= 0",
        s().gamma(&mut d, 0.0, 0.0, 1.0, Accuracy::Standard)
    );
    rec!(
        "weibull alpha <= 0",
        s().weibull(&mut d, 0.0, 0.0, 1.0, Accuracy::Standard)
    );
    rec!("beta p <= 0", s().beta(&mut d, 0.0, 1.0, 0.0, 1.0));
    rec!("chi_square v <= 0", s().chi_square(&mut d, 0));
    rec!("cauchy beta <= 0", s().cauchy(&mut d, 0.0, 0.0));
    rec!(
        "lognormal sigma < 0",
        s().lognormal(&mut d, 0.0, -1.0, 0.0, 1.0, LognormalMethod::Icdf)
    );
    rec!("uniform_int a > b", s().uniform_int(&mut i, 5, 1));
    rec!("bernoulli p > 1", s().bernoulli(&mut i, 1.5));
    rec!("bernoulli p < 0", s().bernoulli(&mut i, -0.5));
    rec!("binomial ntrial < 0", s().binomial(&mut i, -1, 0.5));
    rec!("binomial p > 1", s().binomial(&mut i, 5, 2.0));
    rec!("geometric p = 0", s().geometric(&mut i, 0.0));
    rec!("hypergeometric s > l", s().hypergeometric(&mut i, 5, 9, 2));
    rec!("hypergeometric m > l", s().hypergeometric(&mut i, 5, 2, 9));
    rec!("neg_binomial a <= 0", s().neg_binomial(&mut i, 0.0, 0.5));
    rec!(
        "poisson lambda < 0",
        s().poisson(&mut i, -1.0, PoissonMethod::Ptpe)
    );
    rec!(
        "poisson_v lambda < 0 element",
        s().poisson_v(&mut i, &[-1.0; 8])
    );
    rec!(
        "multinomial p not summing to 1",
        s().multinomial(&mut i, 4, &[0.9, 0.9])
    );
    rec!(
        "multinomial ntrial < 0",
        s().multinomial(&mut i, -1, &[0.5, 0.5])
    );
    rec!("neg_binomial vs negbinomial agree", {
        let (mut a, mut b) = ([0i32; 8], [0i32; 8]);
        let _ = s().neg_binomial(&mut a, 3.0, 0.5);
        let _ = s().negbinomial(&mut b, 3.0, 0.5);
        a == b
    });
    rec!(
        "leapfrog on MT19937",
        Stream::new(Brng::Mt19937, 1).and_then(|mut m| m.leapfrog(0, 2))
    );
    rec!(
        "skip_ahead on MT19937",
        Stream::new(Brng::Mt19937, 1).and_then(|mut m| m.skip_ahead(5))
    );
    rec!("copy_state_from across generators", {
        let mut a = s();
        let b = Stream::new(Brng::Mt19937, 1).unwrap();
        a.copy_state_from(&b)
    });
}
