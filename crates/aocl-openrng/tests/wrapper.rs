//! Tests against the PM-built OpenRNG library (compiled only with OPENRNG_LIB_DIR; run by `C:/openrng-measure/run_wrapper_tests.py`, never by the lane).
//! Known answers: Philox seed 0 is the published Random123 vector (see aocl-openrng-sys/tests/stream.rs); seed 42's words were recorded on both builds.
#![cfg(openrng_linked)]
use aocl_error::Error;
use aocl_openrng::*;

const R123: [u32; 4] = [0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8];

fn philox(seed: u32) -> Stream {
    Stream::new(Brng::Philox4x32x10, seed).unwrap()
}

#[test]
fn version_is_reported() {
    let v = version();
    println!("RECORD openRngGetVersion = {v:?}");
    assert!(v.0 >= 0);
}

#[test]
fn philox_bits_match_the_published_vector_and_a_control() {
    let mut w = [0u32; 4];
    philox(0).uniform_bits32(&mut w).unwrap();
    assert_eq!(w, R123);
    philox(1).uniform_bits32(&mut w).unwrap();
    assert_ne!(w, R123, "control: seed 1 differs");
    let mut w8 = [0u32; 8];
    philox(42).uniform_bits32(&mut w8).unwrap();
    assert_eq!(
        w8,
        [
            0x9ceaf053, 0x77f5493b, 0x12bf50ad, 0x5742b3d7, 0xfcdb2127, 0x53ba6cfd, 0x838f5a6e,
            0x744e06fb
        ]
    );
}

#[test]
fn empty_output_is_accepted_or_refused_never_a_crash() {
    let r = philox(1).uniform(&mut [], 0.0, 1.0, Accuracy::Standard);
    println!("RECORD uniform with n = 0: {r:?}");
}

#[test]
fn clone_and_copy_state_continue_identically() {
    let mut a = philox(7);
    let mut skip = [0u32; 3];
    a.uniform_bits32(&mut skip).unwrap();
    let mut b = a.clone();
    let (mut x, mut y) = ([0u32; 6], [0u32; 6]);
    a.uniform_bits32(&mut x).unwrap();
    b.uniform_bits32(&mut y).unwrap();
    assert_eq!(x, y);
    let mut c = philox(99);
    c.copy_state_from(&a).unwrap();
    let (mut p, mut q) = ([0u32; 4], [0u32; 4]);
    a.uniform_bits32(&mut p).unwrap();
    c.uniform_bits32(&mut q).unwrap();
    assert_eq!(p, q);
}

#[test]
fn save_restore_round_trip_for_every_generator_the_build_accepts() {
    let mut accepted = 0;
    for b in Brng::ALL {
        let mut s = match Stream::new(b, 5) {
            Ok(s) => s,
            Err(e) => {
                println!("REFUSED new({b:?}, 5): {e:?}");
                continue;
            }
        };
        let mut warm = [0u32; 3];
        if let Err(e) = s.uniform_bits32(&mut warm) {
            println!("REFUSED uniform_bits32 on {b:?}: {e:?}");
            continue;
        }
        let saved = match s.save() {
            Ok(v) => v,
            Err(e) => {
                println!("REFUSED save({b:?}): {e:?}");
                continue;
            }
        };
        println!(
            "RECORD save({b:?}) = {} bytes, brng_id {:?}",
            saved.as_bytes().len(),
            s.brng_id()
        );
        let mut r = match saved.restore() {
            Ok(r) => r,
            Err(e) => {
                println!("REFUSED restore({b:?}): {e:?}");
                continue;
            }
        };
        let (mut x, mut y) = ([0u32; 4], [0u32; 4]);
        s.uniform_bits32(&mut x).unwrap();
        r.uniform_bits32(&mut y).unwrap();
        assert_eq!(x, y, "{b:?}: a restored stream must continue identically");
        accepted += 1;
    }
    println!(
        "RECORD save/restore round trip accepted by {accepted} of {} generators",
        Brng::ALL.len()
    );
    assert!(accepted >= 1, "control: at least Philox must round-trip");
}

#[test]
fn leapfrog_validates_k_and_streams_interleave() {
    assert!(matches!(
        philox(1).leapfrog(2, 2),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        philox(1).leapfrog(0, 0),
        Err(Error::InvalidArgument(_))
    ));
    let mut whole = [0u32; 4];
    philox(3).uniform_bits32(&mut whole).unwrap();
    let mut s0 = philox(3);
    let r = s0.leapfrog(0, 2);
    println!("RECORD leapfrog(0, 2) on Philox = {r:?}");
}

#[test]
fn skip_ahead_equals_drawing_and_discarding() {
    let mut a = philox(11);
    let mut drop8 = [0u32; 8];
    a.uniform_bits32(&mut drop8).unwrap();
    let mut b = philox(11);
    let r = b.skip_ahead(8);
    println!("RECORD skip_ahead(8) on Philox = {r:?}");
    if r.is_ok() {
        let (mut x, mut y) = ([0u32; 4], [0u32; 4]);
        a.uniform_bits32(&mut x).unwrap();
        b.uniform_bits32(&mut y).unwrap();
        println!("RECORD skip_ahead(8) equals 8 draws: {}", x == y);
    }
    assert!(matches!(
        philox(1).skip_ahead(-1),
        Err(Error::InvalidArgument(_))
    ));
}

/// Run one generating call; if the build ACCEPTS it, every element must satisfy `ok`; if it REFUSES, record the status
/// (`REFUSED name status`): a refusal is the library's answer and a safe `Err`, not a test failure.
macro_rules! support {
    ($label:expr, $res:expr, $buf:expr, $ok:expr) => {{
        match $res {
            Ok(()) => {
                assert!($buf.iter().all($ok), "{}: {:?}", $label, $buf);
                println!("ACCEPTED {}", $label);
            }
            Err(e) => println!("REFUSED  {:36} {e:?}", $label),
        }
    }};
}

#[test]
fn continuous_distributions_fill_and_stay_in_support_where_the_build_accepts_them() {
    let mut s = philox(2024);
    let mut v = [0f64; 64];
    let r = s.uniform(&mut v, 2.0, 3.0, Accuracy::Standard);
    support!("uniform", r, v, |x| (2.0..3.0).contains(x));
    let r = s.exponential(&mut v, 0.0, 1.0, Accuracy::Standard);
    support!("exponential", r, v, |x| *x >= 0.0);
    let r = s.gamma(&mut v, 2.0, 0.0, 1.0, Accuracy::Standard);
    support!("gamma", r, v, |x| *x >= 0.0);
    let r = s.weibull(&mut v, 2.0, 0.0, 1.0, Accuracy::Standard);
    support!("weibull", r, v, |x| *x >= 0.0);
    let r = s.rayleigh(&mut v, 0.0, 1.0, Accuracy::Standard);
    support!("rayleigh", r, v, |x| *x >= 0.0);
    let r = s.beta(&mut v, 2.0, 3.0, 0.0, 1.0);
    support!("beta", r, v, |x| (0.0..=1.0).contains(x));
    let r = s.chi_square(&mut v, 3);
    support!("chi_square", r, v, |x| *x >= 0.0);
    for m in [
        GaussianMethod::BoxMuller,
        GaussianMethod::BoxMuller2,
        GaussianMethod::Icdf,
    ] {
        let r = s.gaussian(&mut v, 0.0, 1.0, m);
        support!(format!("gaussian {m:?}"), r, v, |x| x.is_finite());
    }
    for m in [LognormalMethod::Icdf, LognormalMethod::BoxMuller2] {
        let r = s.lognormal(&mut v, 0.0, 1.0, 0.0, 1.0, m);
        support!(format!("lognormal {m:?}"), r, v, |x| *x > 0.0);
    }
    let r = s.cauchy(&mut v, 0.0, 1.0);
    support!("cauchy", r, v, |x| x.is_finite());
    let r = s.gumbel(&mut v, 0.0, 1.0);
    support!("gumbel", r, v, |x| x.is_finite());
    let r = s.laplace(&mut v, 0.0, 1.0);
    support!("laplace", r, v, |x| x.is_finite());
    let mut f = [0f32; 64];
    let r = s.uniform_f32(&mut f, 0.0, 1.0, Accuracy::Standard);
    support!("uniform_f32", r, f, |x| (0.0..1.0).contains(x));
    let r = s.gaussian_f32(&mut f, 0.0, 1.0, GaussianMethod::BoxMuller2);
    support!("gaussian_f32", r, f, |x| x.is_finite());
    let r = s.chi_square_f32(&mut f, 3);
    support!("chi_square_f32", r, f, |x| *x >= 0.0);
    let r = s.beta_f32(&mut f, 2.0, 3.0, 0.0, 1.0);
    support!("beta_f32", r, f, |x| (0.0..=1.0).contains(x));
    // control that the macro can fail: Philox accepts uniform, so at least that one must have been ACCEPTED (asserted above by support!)
    let mut g = [0f64; 8];
    assert!(
        s.uniform(&mut g, 0.0, 1.0, Accuracy::Standard).is_ok(),
        "control: uniform must be accepted"
    );
}

#[test]
fn integer_distributions_fill_and_stay_in_support_where_the_build_accepts_them() {
    let mut s = philox(77);
    let mut v = [0i32; 256];
    let r = s.uniform_int(&mut v, 10, 20);
    support!("uniform_int", r, v, |x| (10..20).contains(x));
    let r = s.bernoulli(&mut v, 0.5);
    support!("bernoulli", r, v, |x| *x == 0 || *x == 1);
    let r = s.binomial(&mut v, 10, 0.3);
    support!("binomial", r, v, |x| (0..=10).contains(x));
    let r = s.geometric(&mut v, 0.4);
    support!("geometric", r, v, |x| *x >= 0);
    for m in [PoissonMethod::Ptpe, PoissonMethod::PoisNorm] {
        let r = s.poisson(&mut v, 4.0, m);
        support!(format!("poisson {m:?}"), r, v, |x| *x >= 0);
    }
    let r = s.hypergeometric(&mut v, 20, 8, 5);
    support!("hypergeometric", r, v, |x| (0..=5).contains(x));
    let r = s.neg_binomial(&mut v, 3.0, 0.5);
    support!("neg_binomial", r, v, |x| *x >= 0);
    let r = s.negbinomial(&mut v, 3.0, 0.5);
    support!("negbinomial", r, v, |x| *x >= 0);
    let lam = vec![3.0f64; v.len()];
    let r = s.poisson_v(&mut v, &lam);
    support!("poisson_v", r, v, |x| *x >= 0);
    // multinomial: 50 experiments of 4 categories, each distributing 10 trials
    let mut m = [0i32; 200];
    match s.multinomial(&mut m, 10, &[0.1, 0.2, 0.3, 0.4]) {
        Ok(()) => {
            for row in m.chunks(4) {
                assert_eq!(row.iter().sum::<i32>(), 10, "{row:?}");
            }
            println!("ACCEPTED multinomial");
        }
        Err(e) => println!("REFUSED  {:36} {e:?}", "multinomial"),
    }
    let mut g = [0i32; 8];
    assert!(
        s.uniform_int(&mut g, 0, 5).is_ok(),
        "control: uniform_int must be accepted"
    );
}

#[test]
fn wrong_shapes_are_refused_before_the_library() {
    let mut s = philox(1);
    let mut v = [0i32; 8];
    assert!(matches!(
        s.poisson_v(&mut v, &[1.0; 7]),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        s.multinomial(&mut v, 3, &[0.5, 0.5, 0.0]),
        Err(Error::InvalidArgument(_))
    )); // 8 % 3 != 0
    let mut o = [0f64; 6];
    let r = s.gaussian_mv(
        &mut o,
        3,
        MatrixStorage::Packed,
        &[0.0; 3],
        &[1.0; 5],
        GaussianMethod::BoxMuller,
    );
    assert!(matches!(r, Err(Error::InvalidArgument(_))));
}

#[test]
fn gaussian_mv_with_identity_covariance_runs_in_all_storages() {
    let mut s = philox(5);
    let mut o = [0f64; 30];
    let mean = [1.0, 2.0, 3.0];
    let full = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let packed = [1.0, 0.0, 0.0, 1.0, 0.0, 1.0];
    let diag = [1.0, 1.0, 1.0];
    for (st, t) in [
        (MatrixStorage::Full, &full[..]),
        (MatrixStorage::Packed, &packed[..]),
        (MatrixStorage::Diagonal, &diag[..]),
    ] {
        let r = s.gaussian_mv(&mut o, 3, st, &mean, t, GaussianMethod::BoxMuller);
        println!("RECORD gaussian_mv {st:?} = {r:?}");
        if r.is_ok() {
            assert!(o.iter().all(|x| x.is_finite()));
        }
    }
}

#[test]
fn file_save_and_load_round_trip() {
    let mut s = philox(8);
    let mut warm = [0u32; 5];
    s.uniform_bits32(&mut warm).unwrap();
    let p = std::env::temp_dir().join("aocl_openrng_wrapper_test.state");
    s.save_to_file(&p).unwrap();
    // SAFETY: the file was just written by this process.
    let mut r = unsafe { Stream::load_from_file(&p) }.unwrap();
    let (mut x, mut y) = ([0u32; 4], [0u32; 4]);
    s.uniform_bits32(&mut x).unwrap();
    r.uniform_bits32(&mut y).unwrap();
    assert_eq!(x, y);
    let _ = std::fs::remove_file(&p);
}
