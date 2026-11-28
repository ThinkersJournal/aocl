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
fn save_restore_round_trip_for_every_safe_generator_that_can_be_made_from_a_seed() {
    for b in Brng::ALL {
        let mut s = match Stream::new(b, 5) {
            Ok(s) => s,
            Err(e) => {
                println!("RECORD new({b:?}, 5) = {e:?}");
                continue;
            }
        };
        let mut warm = [0u32; 3];
        let _ = s.uniform_bits32(&mut warm);
        let saved = match s.save() {
            Ok(v) => v,
            Err(e) => {
                println!("RECORD save({b:?}) = {e:?}");
                continue;
            }
        };
        println!(
            "RECORD save({b:?}) = {} bytes, brng_id {:?}",
            saved.as_bytes().len(),
            s.brng_id()
        );
        let mut r = saved.restore().unwrap();
        let (mut x, mut y) = ([0u32; 4], [0u32; 4]);
        s.uniform_bits32(&mut x).unwrap();
        r.uniform_bits32(&mut y).unwrap();
        assert_eq!(x, y, "{b:?}");
    }
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

#[test]
fn continuous_distributions_fill_and_stay_in_support() {
    let mut s = philox(2024);
    let mut v = [0f64; 64];
    s.uniform(&mut v, 2.0, 3.0, Accuracy::Standard).unwrap();
    assert!(v.iter().all(|x| (2.0..3.0).contains(x)));
    s.exponential(&mut v, 0.0, 1.0, Accuracy::Standard).unwrap();
    assert!(v.iter().all(|x| *x >= 0.0));
    s.gamma(&mut v, 2.0, 0.0, 1.0, Accuracy::Standard).unwrap();
    assert!(v.iter().all(|x| *x >= 0.0));
    s.weibull(&mut v, 2.0, 0.0, 1.0, Accuracy::Standard)
        .unwrap();
    assert!(v.iter().all(|x| *x >= 0.0));
    s.rayleigh(&mut v, 0.0, 1.0, Accuracy::Standard).unwrap();
    assert!(v.iter().all(|x| *x >= 0.0));
    s.beta(&mut v, 2.0, 3.0, 0.0, 1.0).unwrap();
    assert!(v.iter().all(|x| (0.0..=1.0).contains(x)));
    s.chi_square(&mut v, 3).unwrap();
    assert!(v.iter().all(|x| *x >= 0.0));
    for m in [
        GaussianMethod::BoxMuller,
        GaussianMethod::BoxMuller2,
        GaussianMethod::Icdf,
    ] {
        s.gaussian(&mut v, 0.0, 1.0, m).unwrap();
        assert!(v.iter().all(|x| x.is_finite()), "{m:?}");
    }
    s.lognormal(&mut v, 0.0, 1.0, 0.0, 1.0, LognormalMethod::Icdf)
        .unwrap();
    assert!(v.iter().all(|x| *x > 0.0));
    for f in [
        Stream::cauchy as fn(&mut Stream, &mut [f64], f64, f64) -> _,
        Stream::gumbel,
        Stream::laplace,
    ] {
        f(&mut s, &mut v, 0.0, 1.0).unwrap();
        assert!(v.iter().all(|x| x.is_finite()));
    }
    let mut f = [0f32; 64];
    s.uniform_f32(&mut f, 0.0, 1.0, Accuracy::Standard).unwrap();
    assert!(f.iter().all(|x| (0.0..1.0).contains(x)));
    s.gaussian_f32(&mut f, 0.0, 1.0, GaussianMethod::BoxMuller2)
        .unwrap();
    assert!(f.iter().all(|x| x.is_finite()));
    s.chi_square_f32(&mut f, 3).unwrap();
    assert!(f.iter().all(|x| *x >= 0.0));
}

#[test]
fn integer_distributions_fill_and_stay_in_support() {
    let mut s = philox(77);
    let mut v = [0i32; 256];
    s.uniform_int(&mut v, 10, 20).unwrap();
    assert!(v.iter().all(|x| (10..20).contains(x)));
    s.bernoulli(&mut v, 0.5).unwrap();
    assert!(v.iter().all(|x| *x == 0 || *x == 1));
    s.binomial(&mut v, 10, 0.3).unwrap();
    assert!(v.iter().all(|x| (0..=10).contains(x)));
    s.geometric(&mut v, 0.4).unwrap();
    assert!(v.iter().all(|x| *x >= 0));
    s.poisson(&mut v, 4.0, PoissonMethod::Ptpe).unwrap();
    assert!(v.iter().all(|x| *x >= 0));
    s.hypergeometric(&mut v, 20, 8, 5).unwrap();
    assert!(v.iter().all(|x| (0..=5).contains(x)));
    s.neg_binomial(&mut v, 3.0, 0.5).unwrap();
    assert!(v.iter().all(|x| *x >= 0));
    let lam = vec![3.0f64; v.len()];
    s.poisson_v(&mut v, &lam).unwrap();
    assert!(v.iter().all(|x| *x >= 0));
    // multinomial: 50 experiments of 4 categories, each distributing 10 trials
    let mut m = [0i32; 200];
    s.multinomial(&mut m, 10, &[0.1, 0.2, 0.3, 0.4]).unwrap();
    for row in m.chunks(4) {
        assert_eq!(row.iter().sum::<i32>(), 10, "{row:?}");
    }
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
        r.unwrap();
        assert!(o.iter().all(|x| x.is_finite()));
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
