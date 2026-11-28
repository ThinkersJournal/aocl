//! RED-FIRST proof for the "output length is n" rule: every generating method is given an output slice that ENDS at a committed
//! page followed by a PAGE_NOACCESS page. A library that wrote past `n` elements would fault (0xc0000005). The length-derived
//! families (multinomial, Gaussian-MV, Poisson-V) are included with their exact required sizes. Windows only; PM-run; the lane does
//! not run the library. Compiled only with OPENRNG_LIB_DIR. (A normal return is the evidence; one process, so a fault names itself.)
#![cfg(all(openrng_linked, windows))]
use aocl_openrng::*;
use core::ffi::c_void;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
    fn VirtualProtect(addr: *mut c_void, size: usize, prot: u32, old: *mut u32) -> i32;
    fn VirtualFree(addr: *mut c_void, size: usize, ty: u32) -> i32;
}
const PAGE: usize = 4096;

fn at_guard<T: Copy + Default, R>(n: usize, f: impl FnOnce(&mut [T]) -> R) -> R {
    assert!(n * core::mem::size_of::<T>() <= PAGE);
    unsafe {
        let base = VirtualAlloc(core::ptr::null_mut(), 2 * PAGE, 0x3000, 0x04);
        assert!(!base.is_null());
        let mut old = 0u32;
        assert_ne!(
            VirtualProtect(
                (base as *mut u8).add(PAGE) as *mut c_void,
                PAGE,
                0x01,
                &mut old
            ),
            0
        );
        let start = (base as *mut u8).add(PAGE - n * core::mem::size_of::<T>()) as *mut T;
        for i in 0..n {
            start.add(i).write(T::default());
        }
        let r = f(core::slice::from_raw_parts_mut(start, n));
        VirtualFree(base, 0, 0x8000);
        r
    }
}

fn st() -> Stream {
    Stream::new(Brng::Philox4x32x10, 3).unwrap()
}

/// Odd lengths on purpose: 1, 3 and 7 are not multiples of the generator's block (4 words for Philox).
const LENS: [usize; 4] = [1, 3, 7, 33];

#[test]
fn real_outputs_stop_at_n() {
    for n in LENS {
        at_guard::<f64, _>(n, |o| {
            let mut s = st();
            s.uniform(o, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.gaussian(o, 0.0, 1.0, GaussianMethod::BoxMuller).unwrap();
            s.gaussian(o, 0.0, 1.0, GaussianMethod::BoxMuller2).unwrap();
            s.gaussian(o, 0.0, 1.0, GaussianMethod::Icdf).unwrap();
            s.lognormal(o, 0.0, 1.0, 0.0, 1.0, LognormalMethod::BoxMuller2)
                .unwrap();
            s.exponential(o, 0.0, 1.0, Accuracy::Accurate).unwrap();
            s.rayleigh(o, 0.0, 1.0, Accuracy::Accurate).unwrap();
            s.weibull(o, 2.0, 0.0, 1.0, Accuracy::Accurate).unwrap();
            s.gamma(o, 2.0, 0.0, 1.0, Accuracy::Accurate).unwrap();
            s.gamma(o, 0.5, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.beta(o, 2.0, 3.0, 0.0, 1.0).unwrap();
            s.chi_square(o, 3).unwrap();
            s.cauchy(o, 0.0, 1.0).unwrap();
            s.gumbel(o, 0.0, 1.0).unwrap();
            s.laplace(o, 0.0, 1.0).unwrap();
        });
        at_guard::<f32, _>(n, |o| {
            let mut s = st();
            s.uniform_f32(o, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.gaussian_f32(o, 0.0, 1.0, GaussianMethod::BoxMuller2)
                .unwrap();
            s.lognormal_f32(o, 0.0, 1.0, 0.0, 1.0, LognormalMethod::Icdf)
                .unwrap();
            s.exponential_f32(o, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.rayleigh_f32(o, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.weibull_f32(o, 2.0, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.gamma_f32(o, 2.0, 0.0, 1.0, Accuracy::Standard).unwrap();
            s.beta_f32(o, 2.0, 3.0, 0.0, 1.0).unwrap();
            s.chi_square_f32(o, 3).unwrap();
            s.cauchy_f32(o, 0.0, 1.0).unwrap();
            s.gumbel_f32(o, 0.0, 1.0).unwrap();
            s.laplace_f32(o, 0.0, 1.0).unwrap();
        });
    }
}

#[test]
fn integer_outputs_stop_at_n() {
    for n in LENS {
        at_guard::<i32, _>(n, |o| {
            let mut s = st();
            s.uniform_int(o, 0, 10).unwrap();
            s.bernoulli(o, 0.5).unwrap();
            s.binomial(o, 10, 0.5).unwrap();
            s.geometric(o, 0.5).unwrap();
            s.hypergeometric(o, 20, 8, 5).unwrap();
            s.neg_binomial(o, 3.0, 0.5).unwrap();
            s.negbinomial(o, 3.0, 0.5).unwrap();
            s.poisson(o, 4.0, PoissonMethod::Ptpe).unwrap();
            s.poisson(o, 40.0, PoissonMethod::PoisNorm).unwrap();
            s.poisson_v(o, &vec![3.0; n]).unwrap();
        });
        at_guard::<u32, _>(n, |o| {
            let mut s = st();
            s.uniform_bits(o).unwrap();
            s.uniform_bits32(o).unwrap();
        });
        at_guard::<u64, _>(n, |o| st().uniform_bits64(o).unwrap());
    }
}

#[test]
fn derived_length_families_stop_at_their_exact_size() {
    for experiments in [1usize, 3, 5] {
        for k in [2usize, 3, 5] {
            at_guard::<i32, _>(experiments * k, |o| {
                let p = vec![1.0 / k as f64; k];
                st().multinomial(o, 7, &p).unwrap();
            });
        }
    }
    for dimen in [1usize, 2, 3] {
        for (storage, tlen) in [
            (MatrixStorage::Full, dimen * dimen),
            (MatrixStorage::Packed, dimen * (dimen + 1) / 2),
            (MatrixStorage::Diagonal, dimen),
        ] {
            let mut t = vec![0.0f64; tlen];
            // a valid Cholesky factor: ones on the diagonal, zero elsewhere (positions differ per storage)
            match storage {
                MatrixStorage::Full => (0..dimen).for_each(|i| t[i * dimen + i] = 1.0),
                MatrixStorage::Packed => {
                    let mut idx = 0;
                    for i in 0..dimen {
                        for j in 0..=i {
                            t[idx] = if i == j { 1.0 } else { 0.0 };
                            idx += 1;
                        }
                    }
                }
                MatrixStorage::Diagonal => t.fill(1.0),
            }
            let mean = vec![0.0f64; dimen];
            at_guard::<f64, _>(2 * dimen, |o| {
                st().gaussian_mv(o, dimen, storage, &mean, &t, GaussianMethod::BoxMuller)
                    .unwrap();
            });
            let (mean32, t32): (Vec<f32>, Vec<f32>) = (
                mean.iter().map(|x| *x as f32).collect(),
                t.iter().map(|x| *x as f32).collect(),
            );
            at_guard::<f32, _>(2 * dimen, |o| {
                st().gaussian_mv_f32(o, dimen, storage, &mean32, &t32, GaussianMethod::BoxMuller)
                    .unwrap();
            });
        }
    }
}

#[test]
fn the_input_arrays_are_not_read_past_their_length() {
    // mean and Cholesky factor at the end of a guard page: a library that read more than dimen / the storage count would fault.
    let dimen = 3usize;
    at_guard::<f64, _>(dimen, |mean| {
        at_guard::<f64, _>(dimen * dimen, |t| {
            for i in 0..dimen {
                t[i * dimen + i] = 1.0;
            }
            let mut out = vec![0.0f64; 2 * dimen];
            st().gaussian_mv(
                &mut out,
                dimen,
                MatrixStorage::Full,
                mean,
                t,
                GaussianMethod::BoxMuller,
            )
            .unwrap();
        });
    });
    at_guard::<f64, _>(4, |p| {
        p.fill(0.25);
        let mut out = vec![0i32; 8];
        st().multinomial(&mut out, 5, p).unwrap();
    });
    at_guard::<f64, _>(8, |lam| {
        lam.fill(2.0);
        let mut out = vec![0i32; 8];
        st().poisson_v(&mut out, lam).unwrap();
    });
}
